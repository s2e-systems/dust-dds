use std::collections::HashSet;

use crate::{
    implementation::{
        data_representation_inline_qos::{
            parameter_id_values::PID_STATUS_INFO,
            types::{STATUS_INFO_DISPOSED_FLAG, STATUS_INFO_UNREGISTERED_FLAG},
        },
        dds_impl::message_receiver::MessageReceiver,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataReaderQos,
        qos_policy::{HistoryQosPolicyKind, LENGTH_UNLIMITED},
        time::Duration,
    },
    topic_definition::type_support::{DdsDeserialize, DdsType, LittleEndian},
};

use super::{
    endpoint::RtpsEndpoint,
    history_cache::RtpsParameter,
    messages::{submessages::DataSubmessage, types::ParameterId},
    reader_cache_change::RtpsReaderCacheChange,
    types::{ChangeKind, Guid, Locator, SequenceNumber, TopicKind},
};

fn calculate_instance_handle(serialized_key: &[u8]) -> InstanceHandle {
    if serialized_key.len() <= 16 {
        let mut h = [0; 16];
        h[..serialized_key.len()].clone_from_slice(serialized_key);
        h.into()
    } else {
        <[u8; 16]>::from(md5::compute(serialized_key)).into()
    }
}

struct ReaderHistoryCache {
    changes: Vec<RtpsReaderCacheChange>,
}

impl ReaderHistoryCache {
    fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }
}

pub struct RtpsReader {
    endpoint: RtpsEndpoint,
    heartbeat_response_delay: Duration,
    heartbeat_suppression_duration: Duration,
    reader_cache: ReaderHistoryCache,
    expects_inline_qos: bool,
    qos: DataReaderQos,
    serialized_data_to_key_func: fn(&[u8]) -> DdsResult<Vec<u8>>,
}

impl RtpsReader {
    pub fn new<T>(
        endpoint: RtpsEndpoint,
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
        qos: DataReaderQos,
    ) -> Self
    where
        T: for<'de> DdsDeserialize<'de> + DdsType,
    {
        // Create a function that deserializes the data and gets the key for the type
        // without having to store the actual type intermediatelly to avoid generics
        fn serialized_data_to_key_func<T>(mut buf: &[u8]) -> DdsResult<Vec<u8>>
        where
            T: for<'de> DdsDeserialize<'de> + DdsType,
        {
            Ok(T::deserialize(&mut buf)?.get_serialized_key::<LittleEndian>())
        }

        Self {
            endpoint,
            heartbeat_response_delay,
            heartbeat_suppression_duration,
            reader_cache: ReaderHistoryCache::new(),
            expects_inline_qos,
            qos,
            serialized_data_to_key_func: serialized_data_to_key_func::<T>,
        }
    }

    pub fn try_into_reader_cache_change(
        &self,
        message_receiver: &MessageReceiver,
        data: &DataSubmessage<'_>,
    ) -> DdsResult<RtpsReaderCacheChange> {
        let writer_guid = Guid::new(
            message_receiver.source_guid_prefix(),
            data.writer_id.value.into(),
        );

        let instance_handle = calculate_instance_handle(
            (self.serialized_data_to_key_func)(data.serialized_payload.value)?.as_ref(),
        );
        let sequence_number = data.writer_sn.value;
        let data_value = data.serialized_payload.value.to_vec();

        let inline_qos: Vec<RtpsParameter> = data
            .inline_qos
            .parameter
            .iter()
            .map(|p| RtpsParameter::new(ParameterId(p.parameter_id), p.value.to_vec()))
            .collect();

        let kind = match (data.data_flag, data.key_flag) {
            (true, false) => Ok(ChangeKind::Alive),
            (false, true) => {
                if let Some(p) = inline_qos
                    .iter()
                    .find(|&x| x.parameter_id() == ParameterId(PID_STATUS_INFO))
                {
                    let mut deserializer =
                        cdr::Deserializer::<_, _, cdr::LittleEndian>::new(p.value(), cdr::Infinite);
                    let status_info = serde::Deserialize::deserialize(&mut deserializer).unwrap();
                    match status_info {
                        STATUS_INFO_DISPOSED_FLAG => Ok(ChangeKind::NotAliveDisposed),
                        STATUS_INFO_UNREGISTERED_FLAG => Ok(ChangeKind::NotAliveUnregistered),
                        _ => Err(DdsError::PreconditionNotMet(
                            "Unknown status info value".to_string(),
                        )),
                    }
                } else {
                    Err(DdsError::PreconditionNotMet(
                        "Missing mandatory StatusInfo parameter".to_string(),
                    ))
                }
            }
            _ => Err(DdsError::PreconditionNotMet(
                "Invalid data submessage data and key flag combination".to_string(),
            )),
        }?;

        let source_timestamp = if message_receiver.have_timestamp() {
            Some(message_receiver.timestamp())
        } else {
            None
        };

        Ok(RtpsReaderCacheChange::new(
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data_value,
            inline_qos,
            source_timestamp,
        ))
    }
}

impl RtpsReader {
    pub fn guid(&self) -> Guid {
        self.endpoint.guid()
    }
}

impl RtpsReader {
    pub fn topic_kind(&self) -> TopicKind {
        self.endpoint.topic_kind()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.endpoint.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.endpoint.multicast_locator_list()
    }
}

impl RtpsReader {
    pub fn heartbeat_response_delay(&self) -> Duration {
        self.heartbeat_response_delay
    }

    pub fn heartbeat_suppression_duration(&self) -> Duration {
        self.heartbeat_suppression_duration
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }
}

impl RtpsReader {
    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) {
        let a_change = match self.try_into_reader_cache_change(message_receiver, data_submessage) {
            Ok(a_change) => a_change,
            Err(_) => return,
        };

        self.add_change(a_change).ok();
    }

    pub fn changes(&self) -> &[RtpsReaderCacheChange] {
        self.reader_cache.changes.as_ref()
    }

    pub fn add_change(&mut self, change: RtpsReaderCacheChange) -> DdsResult<()> {
        if self.qos.history.kind == HistoryQosPolicyKind::KeepLastHistoryQoS
            && change.kind() == ChangeKind::Alive
        {
            let num_instance_samples = self
                .reader_cache
                .changes
                .iter()
                .filter(|cc| {
                    cc.instance_handle() == change.instance_handle()
                        && cc.kind() == ChangeKind::Alive
                })
                .count() as i32;

            if num_instance_samples >= self.qos.history.depth {
                // Remove the lowest sequence number for the instance handle of the cache change
                // Only one sample is to be removed since cache changes come one by one
                let min_seq_num = self
                    .reader_cache
                    .changes
                    .iter()
                    .filter(|cc| {
                        cc.instance_handle() == change.instance_handle()
                            && cc.kind() == ChangeKind::Alive
                    })
                    .map(|cc| cc.sequence_number())
                    .min()
                    .expect("If there are samples there must be a min sequence number");

                self.remove_change(|c| c.sequence_number() == min_seq_num);
            }
        }

        let instance_handle_list: HashSet<_> = self
            .reader_cache
            .changes
            .iter()
            .map(|cc| cc.instance_handle())
            .collect();

        let max_samples_limit_not_reached = self.qos.resource_limits.max_samples
            == LENGTH_UNLIMITED
            || (self.reader_cache.changes.len() as i32) < self.qos.resource_limits.max_samples;

        let max_instances_limit_not_reached = instance_handle_list
            .contains(&change.instance_handle())
            || self.qos.resource_limits.max_instances == LENGTH_UNLIMITED
            || (instance_handle_list.len() as i32) < self.qos.resource_limits.max_instances;

        let max_samples_per_instance_limit_not_reached =
            self.qos.resource_limits.max_samples_per_instance == LENGTH_UNLIMITED
                || (self
                    .changes()
                    .iter()
                    .filter(|cc| cc.instance_handle() == change.instance_handle())
                    .count() as i32)
                    < self.qos.resource_limits.max_samples_per_instance;

        if max_samples_limit_not_reached
            && max_instances_limit_not_reached
            && max_samples_per_instance_limit_not_reached
        {
            self.reader_cache.changes.push(change);
            Ok(())
        } else {
            Err(DdsError::OutOfResources)
        }
    }

    pub fn remove_change<F>(&mut self, mut f: F)
    where
        F: FnMut(&RtpsReaderCacheChange) -> bool,
    {
        self.reader_cache.changes.retain(|cc| !f(cc));
    }

    pub fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.reader_cache
            .changes
            .iter()
            .map(|cc| cc.sequence_number())
            .min()
    }

    pub fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.reader_cache
            .changes
            .iter()
            .map(|cc| cc.sequence_number())
            .max()
    }
}

impl RtpsReader {
    pub fn get_qos(&self) -> &DataReaderQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: DataReaderQos) -> DdsResult<()> {
        qos.is_consistent()?;
        self.qos = qos;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        implementation::rtps::types::{ChangeKind, GUID_UNKNOWN},
        infrastructure::{
            instance::HANDLE_NIL,
            qos_policy::{HistoryQosPolicy, ResourceLimitsQosPolicy},
            time::DURATION_ZERO,
        },
    };

    use super::*;

    struct MockType;

    impl DdsType for MockType {
        fn type_name() -> &'static str {
            todo!()
        }
    }

    impl<'de> DdsDeserialize<'de> for MockType {
        fn deserialize(_buf: &mut &'de [u8]) -> DdsResult<Self> {
            todo!()
        }
    }

    #[test]
    fn reader_no_key_add_change_keep_last_1() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLastHistoryQoS,
                depth: 1,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            1,
            vec![1],
            vec![],
            None,
        );
        let change2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            HANDLE_NIL,
            2,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1).unwrap();
        reader.add_change(change2.clone()).unwrap();

        assert_eq!(reader.changes().len(), 1);
        assert_eq!(reader.changes()[0], change2);
    }

    #[test]
    fn reader_with_key_add_change_keep_last_1() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLastHistoryQoS,
                depth: 1,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16].into(),
            1,
            vec![1],
            vec![],
            None,
        );
        let change2_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16].into(),
            2,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1_instance1).unwrap();
        reader.add_change(change2_instance1.clone()).unwrap();

        let change1_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16].into(),
            1,
            vec![1],
            vec![],
            None,
        );
        let change2_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16].into(),
            2,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1_instance2).unwrap();
        reader.add_change(change2_instance2.clone()).unwrap();

        assert_eq!(reader.changes().len(), 2);
        assert!(reader.changes().contains(&change2_instance1));
        assert!(reader.changes().contains(&change2_instance2));
    }

    #[test]
    fn reader_no_key_add_change_keep_last_3() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLastHistoryQoS,
                depth: 3,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16].into(),
            1,
            vec![1],
            vec![],
            None,
        );
        let change2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16].into(),
            2,
            vec![2],
            vec![],
            None,
        );
        let change3 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16].into(),
            3,
            vec![3],
            vec![],
            None,
        );
        let change4 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16].into(),
            4,
            vec![4],
            vec![],
            None,
        );
        reader.add_change(change1).unwrap();
        reader.add_change(change2.clone()).unwrap();
        reader.add_change(change3.clone()).unwrap();
        reader.add_change(change4.clone()).unwrap();

        assert_eq!(reader.changes().len(), 3);
        assert!(reader.changes().contains(&change2));
        assert!(reader.changes().contains(&change3));
        assert!(reader.changes().contains(&change4));
    }

    #[test]
    fn reader_with_key_add_change_keep_last_3() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLastHistoryQoS,
                depth: 3,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16].into(),
            1,
            vec![1],
            vec![],
            None,
        );
        let change2_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16].into(),
            2,
            vec![1],
            vec![],
            None,
        );
        let change3_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16].into(),
            3,
            vec![1],
            vec![],
            None,
        );
        let change4_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16].into(),
            4,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1_instance1).unwrap();
        reader.add_change(change2_instance1.clone()).unwrap();
        reader.add_change(change3_instance1.clone()).unwrap();
        reader.add_change(change4_instance1.clone()).unwrap();

        let change1_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16].into(),
            1,
            vec![1],
            vec![],
            None,
        );
        let change2_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16].into(),
            2,
            vec![1],
            vec![],
            None,
        );
        let change3_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16].into(),
            3,
            vec![1],
            vec![],
            None,
        );
        let change4_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16].into(),
            4,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1_instance2).unwrap();
        reader.add_change(change2_instance2.clone()).unwrap();
        reader.add_change(change3_instance2.clone()).unwrap();
        reader.add_change(change4_instance2.clone()).unwrap();

        assert_eq!(reader.changes().len(), 6);
        assert!(reader.changes().contains(&change2_instance1));
        assert!(reader.changes().contains(&change3_instance1));
        assert!(reader.changes().contains(&change4_instance1));
        assert!(reader.changes().contains(&change2_instance2));
        assert!(reader.changes().contains(&change3_instance2));
        assert!(reader.changes().contains(&change4_instance2));
    }

    #[test]
    fn reader_max_samples() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAllHistoryQos,
                depth: LENGTH_UNLIMITED,
            },
            resource_limits: ResourceLimitsQosPolicy {
                max_samples: 1,
                max_instances: LENGTH_UNLIMITED,
                max_samples_per_instance: LENGTH_UNLIMITED,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16].into(),
            1,
            vec![1],
            vec![],
            None,
        );
        let change2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16].into(),
            2,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1).unwrap();

        assert_eq!(reader.add_change(change2), Err(DdsError::OutOfResources));
    }

    #[test]
    fn reader_max_instances() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAllHistoryQos,
                depth: LENGTH_UNLIMITED,
            },
            resource_limits: ResourceLimitsQosPolicy {
                max_samples: LENGTH_UNLIMITED,
                max_instances: 1,
                max_samples_per_instance: LENGTH_UNLIMITED,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16].into(),
            1,
            vec![1],
            vec![],
            None,
        );
        let change1_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16].into(),
            2,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1_instance1).unwrap();

        assert_eq!(
            reader.add_change(change1_instance2),
            Err(DdsError::OutOfResources)
        );
    }

    #[test]
    fn reader_max_samples_per_instance() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAllHistoryQos,
                depth: LENGTH_UNLIMITED,
            },
            resource_limits: ResourceLimitsQosPolicy {
                max_samples: LENGTH_UNLIMITED,
                max_instances: LENGTH_UNLIMITED,
                max_samples_per_instance: 1,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16].into(),
            1,
            vec![1],
            vec![],
            None,
        );
        let change1_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16].into(),
            2,
            vec![1],
            vec![],
            None,
        );
        let change2_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16].into(),
            3,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1_instance1).unwrap();
        reader.add_change(change1_instance2).unwrap();

        assert_eq!(
            reader.add_change(change2_instance2),
            Err(DdsError::OutOfResources)
        );
    }
}
