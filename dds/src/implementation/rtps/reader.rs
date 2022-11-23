use std::collections::{HashMap, HashSet};

use crate::{
    implementation::{
        data_representation_inline_qos::{
            parameter_id_values::PID_STATUS_INFO,
            types::{STATUS_INFO_DISPOSED_FLAG, STATUS_INFO_UNREGISTERED_FLAG},
        },
        dds_impl::{message_receiver::MessageReceiver, status_condition_impl::StatusConditionImpl},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataReaderQos,
        qos_policy::{DestinationOrderQosPolicyKind, HistoryQosPolicyKind, LENGTH_UNLIMITED},
        status::StatusKind,
        time::Duration,
    },
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::{DdsDeserialize, DdsType, LittleEndian},
};

use super::{
    endpoint::RtpsEndpoint,
    history_cache::RtpsParameter,
    messages::{submessages::DataSubmessage, types::ParameterId},
    reader_cache_change::RtpsReaderCacheChange,
    types::{ChangeKind, Guid},
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

struct Instance {
    view_state: ViewStateKind,
    instance_state: InstanceStateKind,
    most_recent_disposed_generation_count: i32,
    most_recent_no_writers_generation_count: i32,
}

pub struct RtpsReader {
    endpoint: RtpsEndpoint,
    _heartbeat_response_delay: Duration,
    _heartbeat_suppression_duration: Duration,
    reader_cache: ReaderHistoryCache,
    _expects_inline_qos: bool,
    qos: DataReaderQos,
    serialized_data_to_key_func: fn(&[u8]) -> DdsResult<Vec<u8>>,
    status_condition: StatusConditionImpl,
    instances: HashMap<InstanceHandle, Instance>,
}

impl RtpsReader {
    pub fn new<Foo>(
        endpoint: RtpsEndpoint,
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
        qos: DataReaderQos,
    ) -> Self
    where
        Foo: for<'de> DdsDeserialize<'de> + DdsType,
    {
        // Create a function that deserializes the data and gets the key for the type
        // without having to store the actual type intermediatelly to avoid generics
        fn serialized_data_to_key_func<Foo>(mut buf: &[u8]) -> DdsResult<Vec<u8>>
        where
            Foo: for<'de> DdsDeserialize<'de> + DdsType,
        {
            Ok(Foo::deserialize(&mut buf)?.get_serialized_key::<LittleEndian>())
        }

        Self {
            endpoint,
            _heartbeat_response_delay: heartbeat_response_delay,
            _heartbeat_suppression_duration: heartbeat_suppression_duration,
            reader_cache: ReaderHistoryCache::new(),
            _expects_inline_qos: expects_inline_qos,
            qos,
            serialized_data_to_key_func: serialized_data_to_key_func::<Foo>,
            status_condition: StatusConditionImpl::default(),
            instances: HashMap::new(),
        }
    }

    pub fn guid(&self) -> Guid {
        self.endpoint.guid()
    }

    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) {
        let a_change = match RtpsReaderCacheChange::try_from_data_submessage(
            data_submessage,
            Some(message_receiver.timestamp()),
            message_receiver.source_guid_prefix(),
        ) {
            Ok(a_change) => a_change,
            Err(_) => return,
        };

        self.add_change(a_change).ok();
    }

    pub fn changes(&self) -> &[RtpsReaderCacheChange] {
        self.reader_cache.changes.as_ref()
    }

    pub fn add_change(&mut self, change: RtpsReaderCacheChange) -> DdsResult<()> {
        let change_instance_handle = calculate_instance_handle(&(self
            .serialized_data_to_key_func)(
            &mut change.data_value()
        )?);

        if self.qos.history.kind == HistoryQosPolicyKind::KeepLast
            && change.kind() == ChangeKind::Alive
        {
            let num_instance_samples = self
                .reader_cache
                .changes
                .iter()
                .filter(|cc| {
                    calculate_instance_handle(
                        &(self.serialized_data_to_key_func)(&mut cc.data_value()).unwrap(),
                    ) == change_instance_handle
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
                        calculate_instance_handle(
                            &(self.serialized_data_to_key_func)(&mut cc.data_value()).unwrap(),
                        ) == change_instance_handle
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
            .map(|cc| {
                calculate_instance_handle(
                    &(self.serialized_data_to_key_func)(&mut cc.data_value()).unwrap(),
                )
            })
            .collect();

        let max_samples_limit_not_reached = self.qos.resource_limits.max_samples
            == LENGTH_UNLIMITED
            || (self.reader_cache.changes.len() as i32) < self.qos.resource_limits.max_samples;

        let max_instances_limit_not_reached = instance_handle_list
            .contains(&change_instance_handle)
            || self.qos.resource_limits.max_instances == LENGTH_UNLIMITED
            || (instance_handle_list.len() as i32) < self.qos.resource_limits.max_instances;

        let max_samples_per_instance_limit_not_reached =
            self.qos.resource_limits.max_samples_per_instance == LENGTH_UNLIMITED
                || (self
                    .changes()
                    .iter()
                    .filter(|cc| {
                        calculate_instance_handle(
                            &(self.serialized_data_to_key_func)(&mut cc.data_value()).unwrap(),
                        ) == change_instance_handle
                    })
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

    pub fn get_qos(&self) -> &DataReaderQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: DataReaderQos) -> DdsResult<()> {
        qos.is_consistent()?;
        self.qos = qos;
        Ok(())
    }

    pub fn read<Foo>(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.status_condition
            .remove_communication_state(StatusKind::DataAvailable);

        let serialized_data_to_key_func = &self.serialized_data_to_key_func;
        let mut samples = {
            self.reader_cache
                .changes
                .iter_mut()
                .map(|cache_change| {
                    let sample_state = cache_change.sample_state();
                    let view_state = cache_change.view_state();
                    cache_change.mark_read();

                    let (instance_state, valid_data) = match cache_change.kind() {
                        ChangeKind::Alive => (InstanceStateKind::Alive, true),
                        ChangeKind::NotAliveDisposed => {
                            (InstanceStateKind::NotAliveDisposed, false)
                        }
                        _ => unimplemented!(),
                    };

                    let sample_info = SampleInfo {
                        sample_state,
                        view_state,
                        instance_state,
                        disposed_generation_count: 0,
                        no_writers_generation_count: 0,
                        sample_rank: 0,
                        generation_rank: 0,
                        absolute_generation_rank: 0,
                        source_timestamp: *cache_change.source_timestamp(),
                        instance_handle: calculate_instance_handle(&(serialized_data_to_key_func)(
                            &mut cache_change.data_value(),
                        )?),
                        publication_handle: <[u8; 16]>::from(cache_change.writer_guid()).into(),
                        valid_data,
                    };

                    let value = DdsDeserialize::deserialize(&mut cache_change.data_value())?;
                    Ok(Sample {
                        data: Some(value),
                        sample_info,
                    })
                })
                .filter(|result| {
                    if let Ok(sample) = result {
                        sample_states.contains(&sample.sample_info.sample_state)
                    } else {
                        true
                    }
                })
                .take(max_samples as usize)
                .collect::<DdsResult<Vec<_>>>()
        }?;

        if self.qos.destination_order.kind == DestinationOrderQosPolicyKind::BySourceTimestamp {
            samples.sort_by(|a, b| {
                a.sample_info
                    .source_timestamp
                    .as_ref()
                    .unwrap()
                    .cmp(b.sample_info.source_timestamp.as_ref().unwrap())
            });
        }

        if samples.is_empty() {
            Err(DdsError::NoData)
        } else {
            Ok(samples)
        }
    }

    pub fn take<Foo>(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.status_condition
            .remove_communication_state(StatusKind::DataAvailable);

        let serialized_data_to_key_func = &self.serialized_data_to_key_func;
        let mut samples = {
            self.reader_cache
                .changes
                .drain(..)
                .map(|cache_change| {
                    let sample_state = cache_change.sample_state();
                    let view_state = ViewStateKind::New;

                    let (instance_state, valid_data) = match cache_change.kind() {
                        ChangeKind::Alive => (InstanceStateKind::Alive, true),
                        ChangeKind::NotAliveDisposed => {
                            (InstanceStateKind::NotAliveDisposed, false)
                        }
                        _ => unimplemented!(),
                    };

                    let sample_info = SampleInfo {
                        sample_state,
                        view_state,
                        instance_state,
                        disposed_generation_count: 0,
                        no_writers_generation_count: 0,
                        sample_rank: 0,
                        generation_rank: 0,
                        absolute_generation_rank: 0,
                        source_timestamp: *cache_change.source_timestamp(),
                        instance_handle: calculate_instance_handle(&(serialized_data_to_key_func)(
                            &mut cache_change.data_value(),
                        )?),
                        publication_handle: <[u8; 16]>::from(cache_change.writer_guid()).into(),
                        valid_data,
                    };

                    let value = DdsDeserialize::deserialize(&mut cache_change.data_value())?;
                    Ok(Sample {
                        data: Some(value),
                        sample_info,
                    })
                })
                .filter(|result| {
                    if let Ok(sample) = result {
                        sample_states.contains(&sample.sample_info.sample_state)
                    } else {
                        true
                    }
                })
                .take(max_samples as usize)
                .collect::<DdsResult<Vec<_>>>()
        }?;

        if self.qos.destination_order.kind == DestinationOrderQosPolicyKind::BySourceTimestamp {
            samples.sort_by(|a, b| {
                a.sample_info
                    .source_timestamp
                    .as_ref()
                    .unwrap()
                    .cmp(b.sample_info.source_timestamp.as_ref().unwrap())
            });
        }

        if samples.is_empty() {
            Err(DdsError::NoData)
        } else {
            Ok(samples)
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         implementation::rtps::types::{ChangeKind, TopicKind, GUID_UNKNOWN},
//         infrastructure::{
//             instance::HANDLE_NIL,
//             qos_policy::{HistoryQosPolicy, ResourceLimitsQosPolicy},
//             time::DURATION_ZERO,
//         },
//     };

//     use super::*;

//     struct MockType;

//     impl DdsType for MockType {
//         fn type_name() -> &'static str {
//             todo!()
//         }
//     }

//     impl<'de> DdsDeserialize<'de> for MockType {
//         fn deserialize(_buf: &mut &'de [u8]) -> DdsResult<Self> {
//             todo!()
//         }
//     }

//     #[test]
//     fn reader_no_key_add_change_keep_last_1() {
//         let qos = DataReaderQos {
//             history: HistoryQosPolicy {
//                 kind: HistoryQosPolicyKind::KeepLast,
//                 depth: 1,
//             },
//             ..Default::default()
//         };
//         let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
//         let mut reader =
//             RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

//         let change1 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             HANDLE_NIL,
//             1,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change2 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             HANDLE_NIL,
//             2,
//             vec![1],
//             vec![],
//             None,
//         );
//         reader.add_change(change1).unwrap();
//         reader.add_change(change2.clone()).unwrap();

//         assert_eq!(reader.changes().len(), 1);
//         assert_eq!(reader.changes()[0], change2);
//     }

//     #[test]
//     fn reader_with_key_add_change_keep_last_1() {
//         let qos = DataReaderQos {
//             history: HistoryQosPolicy {
//                 kind: HistoryQosPolicyKind::KeepLast,
//                 depth: 1,
//             },
//             ..Default::default()
//         };
//         let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]);
//         let mut reader =
//             RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

//         let change1_instance1 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [1; 16].into(),
//             1,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change2_instance1 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [1; 16].into(),
//             2,
//             vec![1],
//             vec![],
//             None,
//         );
//         reader.add_change(change1_instance1).unwrap();
//         reader.add_change(change2_instance1.clone()).unwrap();

//         let change1_instance2 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [2; 16].into(),
//             1,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change2_instance2 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [2; 16].into(),
//             2,
//             vec![1],
//             vec![],
//             None,
//         );
//         reader.add_change(change1_instance2).unwrap();
//         reader.add_change(change2_instance2.clone()).unwrap();

//         assert_eq!(reader.changes().len(), 2);
//         assert!(reader.changes().contains(&change2_instance1));
//         assert!(reader.changes().contains(&change2_instance2));
//     }

//     #[test]
//     fn reader_no_key_add_change_keep_last_3() {
//         let qos = DataReaderQos {
//             history: HistoryQosPolicy {
//                 kind: HistoryQosPolicyKind::KeepLast,
//                 depth: 3,
//             },
//             ..Default::default()
//         };
//         let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
//         let mut reader =
//             RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

//         let change1 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [0; 16].into(),
//             1,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change2 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [0; 16].into(),
//             2,
//             vec![2],
//             vec![],
//             None,
//         );
//         let change3 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [0; 16].into(),
//             3,
//             vec![3],
//             vec![],
//             None,
//         );
//         let change4 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [0; 16].into(),
//             4,
//             vec![4],
//             vec![],
//             None,
//         );
//         reader.add_change(change1).unwrap();
//         reader.add_change(change2.clone()).unwrap();
//         reader.add_change(change3.clone()).unwrap();
//         reader.add_change(change4.clone()).unwrap();

//         assert_eq!(reader.changes().len(), 3);
//         assert!(reader.changes().contains(&change2));
//         assert!(reader.changes().contains(&change3));
//         assert!(reader.changes().contains(&change4));
//     }

//     #[test]
//     fn reader_with_key_add_change_keep_last_3() {
//         let qos = DataReaderQos {
//             history: HistoryQosPolicy {
//                 kind: HistoryQosPolicyKind::KeepLast,
//                 depth: 3,
//             },
//             ..Default::default()
//         };
//         let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]);
//         let mut reader =
//             RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

//         let change1_instance1 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [1; 16].into(),
//             1,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change2_instance1 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [1; 16].into(),
//             2,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change3_instance1 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [1; 16].into(),
//             3,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change4_instance1 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [1; 16].into(),
//             4,
//             vec![1],
//             vec![],
//             None,
//         );
//         reader.add_change(change1_instance1).unwrap();
//         reader.add_change(change2_instance1.clone()).unwrap();
//         reader.add_change(change3_instance1.clone()).unwrap();
//         reader.add_change(change4_instance1.clone()).unwrap();

//         let change1_instance2 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [2; 16].into(),
//             1,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change2_instance2 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [2; 16].into(),
//             2,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change3_instance2 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [2; 16].into(),
//             3,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change4_instance2 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [2; 16].into(),
//             4,
//             vec![1],
//             vec![],
//             None,
//         );
//         reader.add_change(change1_instance2).unwrap();
//         reader.add_change(change2_instance2.clone()).unwrap();
//         reader.add_change(change3_instance2.clone()).unwrap();
//         reader.add_change(change4_instance2.clone()).unwrap();

//         assert_eq!(reader.changes().len(), 6);
//         assert!(reader.changes().contains(&change2_instance1));
//         assert!(reader.changes().contains(&change3_instance1));
//         assert!(reader.changes().contains(&change4_instance1));
//         assert!(reader.changes().contains(&change2_instance2));
//         assert!(reader.changes().contains(&change3_instance2));
//         assert!(reader.changes().contains(&change4_instance2));
//     }

//     #[test]
//     fn reader_max_samples() {
//         let qos = DataReaderQos {
//             history: HistoryQosPolicy {
//                 kind: HistoryQosPolicyKind::KeepAll,
//                 depth: LENGTH_UNLIMITED,
//             },
//             resource_limits: ResourceLimitsQosPolicy {
//                 max_samples: 1,
//                 max_instances: LENGTH_UNLIMITED,
//                 max_samples_per_instance: LENGTH_UNLIMITED,
//             },
//             ..Default::default()
//         };
//         let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
//         let mut reader =
//             RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

//         let change1 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [0; 16].into(),
//             1,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change2 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [0; 16].into(),
//             2,
//             vec![1],
//             vec![],
//             None,
//         );
//         reader.add_change(change1).unwrap();

//         assert_eq!(reader.add_change(change2), Err(DdsError::OutOfResources));
//     }

//     #[test]
//     fn reader_max_instances() {
//         let qos = DataReaderQos {
//             history: HistoryQosPolicy {
//                 kind: HistoryQosPolicyKind::KeepAll,
//                 depth: LENGTH_UNLIMITED,
//             },
//             resource_limits: ResourceLimitsQosPolicy {
//                 max_samples: LENGTH_UNLIMITED,
//                 max_instances: 1,
//                 max_samples_per_instance: LENGTH_UNLIMITED,
//             },
//             ..Default::default()
//         };
//         let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
//         let mut reader =
//             RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

//         let change1_instance1 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [0; 16].into(),
//             1,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change1_instance2 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [1; 16].into(),
//             2,
//             vec![1],
//             vec![],
//             None,
//         );
//         reader.add_change(change1_instance1).unwrap();

//         assert_eq!(
//             reader.add_change(change1_instance2),
//             Err(DdsError::OutOfResources)
//         );
//     }

//     #[test]
//     fn reader_max_samples_per_instance() {
//         let qos = DataReaderQos {
//             history: HistoryQosPolicy {
//                 kind: HistoryQosPolicyKind::KeepAll,
//                 depth: LENGTH_UNLIMITED,
//             },
//             resource_limits: ResourceLimitsQosPolicy {
//                 max_samples: LENGTH_UNLIMITED,
//                 max_instances: LENGTH_UNLIMITED,
//                 max_samples_per_instance: 1,
//             },
//             ..Default::default()
//         };
//         let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
//         let mut reader =
//             RtpsReader::new::<MockType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

//         let change1_instance1 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [1; 16].into(),
//             1,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change1_instance2 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [2; 16].into(),
//             2,
//             vec![1],
//             vec![],
//             None,
//         );
//         let change2_instance2 = RtpsReaderCacheChange::new(
//             ChangeKind::Alive,
//             GUID_UNKNOWN,
//             [2; 16].into(),
//             3,
//             vec![1],
//             vec![],
//             None,
//         );
//         reader.add_change(change1_instance1).unwrap();
//         reader.add_change(change1_instance2).unwrap();

//         assert_eq!(
//             reader.add_change(change2_instance2),
//             Err(DdsError::OutOfResources)
//         );
//     }
// }
