use std::collections::{HashMap, HashSet};

use crate::{
    implementation::{
        data_representation_inline_qos::{
            parameter_id_values::PID_STATUS_INFO,
            types::{StatusInfo, STATUS_INFO_DISPOSED_FLAG, STATUS_INFO_UNREGISTERED_FLAG},
        },
        dds_impl::status_condition_impl::StatusConditionImpl,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataReaderQos,
        qos_policy::{DestinationOrderQosPolicyKind, HistoryQosPolicyKind, LENGTH_UNLIMITED},
        status::StatusKind,
        time::{Duration, Time},
    },
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    endpoint::RtpsEndpoint,
    history_cache::RtpsParameter,
    messages::{submessages::DataSubmessage, types::ParameterId},
    types::{ChangeKind, Guid, GuidPrefix},
};

pub struct RtpsReaderCacheChange {
    kind: ChangeKind,
    writer_guid: Guid,
    data: Vec<u8>,
    source_timestamp: Option<Time>,
    sample_state: SampleStateKind,
    disposed_generation_count: i32,
    no_writers_generation_count: i32,
    reception_timestamp: Time,
}

struct InstanceHandleBuilder(fn(&[u8]) -> DdsResult<Vec<u8>>);

impl InstanceHandleBuilder {
    fn new<Foo>() -> Self
    where
        Foo: for<'de> DdsDeserialize<'de> + DdsType,
    {
        Self(Foo::deserialize_key)
    }

    fn build_instance_handle(
        &self,
        change_kind: ChangeKind,
        data: &[u8],
    ) -> DdsResult<InstanceHandle> {
        Ok(match change_kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => (self.0)(data)?.as_slice().into(),
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => data[4..].into(), // Ignore the first 4 bytes which are the header
        })
    }
}

struct Instance {
    view_state: ViewStateKind,
    instance_state: InstanceStateKind,
    most_recent_disposed_generation_count: i32,
    most_recent_no_writers_generation_count: i32,
}

impl Instance {
    fn new() -> Self {
        Self {
            view_state: ViewStateKind::New,
            instance_state: InstanceStateKind::Alive,
            most_recent_disposed_generation_count: 0,
            most_recent_no_writers_generation_count: 0,
        }
    }

    fn update_state(&mut self, change_kind: ChangeKind) {
        match self.instance_state {
            InstanceStateKind::Alive => {
                if change_kind == ChangeKind::NotAliveDisposed {
                    self.instance_state = InstanceStateKind::NotAliveDisposed;
                } else if change_kind == ChangeKind::NotAliveUnregistered {
                    self.instance_state = InstanceStateKind::NotAliveNoWriters;
                }
            }
            InstanceStateKind::NotAliveDisposed => {
                if change_kind == ChangeKind::Alive {
                    self.instance_state = InstanceStateKind::Alive;
                    self.most_recent_disposed_generation_count += 1;
                }
            }
            InstanceStateKind::NotAliveNoWriters => {
                if change_kind == ChangeKind::Alive {
                    self.instance_state = InstanceStateKind::Alive;
                    self.most_recent_no_writers_generation_count += 1;
                }
            }
        }

        match self.view_state {
            ViewStateKind::New => (),
            ViewStateKind::NotNew => {
                if change_kind == ChangeKind::NotAliveDisposed
                    || change_kind == ChangeKind::NotAliveUnregistered
                {
                    self.view_state = ViewStateKind::New;
                }
            }
        }
    }

    fn mark_viewed(&mut self) {
        self.view_state = ViewStateKind::NotNew;
    }
}

pub struct RtpsReader {
    endpoint: RtpsEndpoint,
    _heartbeat_response_delay: Duration,
    _heartbeat_suppression_duration: Duration,
    changes: Vec<RtpsReaderCacheChange>,
    _expects_inline_qos: bool,
    qos: DataReaderQos,
    status_condition: StatusConditionImpl,
    instance_handle_builder: InstanceHandleBuilder,
    instances: HashMap<InstanceHandle, Instance>,
    instance_reception_time: HashMap<InstanceHandle, Time>,
    data_available: bool,
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
        Foo: DdsType + for<'de> DdsDeserialize<'de>,
    {
        let instance_handle_builder = InstanceHandleBuilder::new::<Foo>();
        Self {
            endpoint,
            _heartbeat_response_delay: heartbeat_response_delay,
            _heartbeat_suppression_duration: heartbeat_suppression_duration,
            changes: Vec::new(),
            _expects_inline_qos: expects_inline_qos,
            qos,
            status_condition: StatusConditionImpl::default(),
            instance_handle_builder,
            instances: HashMap::new(),
            instance_reception_time: HashMap::new(),
            data_available: false,
        }
    }

    pub fn guid(&self) -> Guid {
        self.endpoint.guid()
    }

    pub fn add_change(
        &mut self,
        data_submessage: &DataSubmessage,
        source_timestamp: Option<Time>,
        source_guid_prefix: GuidPrefix,
        reception_timestamp: Time,
    ) -> DdsResult<()> {
        let writer_guid = Guid::new(source_guid_prefix, data_submessage.writer_id.value.into());

        let data = data_submessage.serialized_payload.value.to_vec();

        let inline_qos: Vec<RtpsParameter> = data_submessage
            .inline_qos
            .parameter
            .iter()
            .map(|p| RtpsParameter::new(ParameterId(p.parameter_id), p.value.to_vec()))
            .collect();

        let change_kind = match (data_submessage.data_flag, data_submessage.key_flag) {
            (true, false) => Ok(ChangeKind::Alive),
            (false, true) => {
                if let Some(p) = inline_qos
                    .iter()
                    .find(|&x| x.parameter_id() == ParameterId(PID_STATUS_INFO))
                {
                    let mut deserializer =
                        cdr::Deserializer::<_, _, cdr::LittleEndian>::new(p.value(), cdr::Infinite);
                    let status_info: StatusInfo =
                        serde::Deserialize::deserialize(&mut deserializer).unwrap();
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
        let change_instance_handle = self
            .instance_handle_builder
            .build_instance_handle(change_kind, data.as_slice())?;

        if self.qos.history.kind == HistoryQosPolicyKind::KeepLast
            && change_kind == ChangeKind::Alive
        {
            let num_instance_samples = self
                .changes
                .iter()
                .filter(|cc| {
                    self.instance_handle_builder
                        .build_instance_handle(change_kind, cc.data.as_slice())
                        .unwrap()
                        == change_instance_handle
                        && cc.kind == ChangeKind::Alive
                })
                .count() as i32;

            if num_instance_samples >= self.qos.history.depth {
                // Remove the lowest sequence number for the instance handle of the cache change
                // Only one sample is to be removed since cache changes come one by one
                let oldest_sample_index = self
                    .changes
                    .iter()
                    .position(|cc| {
                        self.instance_handle_builder
                            .build_instance_handle(change_kind, cc.data.as_slice())
                            .unwrap()
                            == change_instance_handle
                            && cc.kind == ChangeKind::Alive
                    })
                    .expect("If there are samples there must be a min sequence number");
                self.changes.remove(oldest_sample_index);
            }
        }

        let instance_handle_list: HashSet<_> = self
            .changes
            .iter()
            .map(|cc| {
                self.instance_handle_builder
                    .build_instance_handle(cc.kind, cc.data.as_slice())
                    .unwrap()
            })
            .collect();

        let max_samples_limit_not_reached = self.qos.resource_limits.max_samples
            == LENGTH_UNLIMITED
            || (self.changes.len() as i32) < self.qos.resource_limits.max_samples;

        let max_instances_limit_not_reached = instance_handle_list
            .contains(&change_instance_handle)
            || self.qos.resource_limits.max_instances == LENGTH_UNLIMITED
            || (instance_handle_list.len() as i32) < self.qos.resource_limits.max_instances;

        let max_samples_per_instance_limit_not_reached =
            self.qos.resource_limits.max_samples_per_instance == LENGTH_UNLIMITED
                || (self
                    .changes
                    .as_slice()
                    .iter()
                    .filter(|cc| {
                        self.instance_handle_builder
                            .build_instance_handle(change_kind, cc.data.as_slice())
                            .unwrap()
                            == change_instance_handle
                    })
                    .count() as i32)
                    < self.qos.resource_limits.max_samples_per_instance;

        if max_samples_limit_not_reached
            && max_instances_limit_not_reached
            && max_samples_per_instance_limit_not_reached
        {
            let instance_entry = self
                .instances
                .entry(change_instance_handle)
                .or_insert_with(Instance::new);

            instance_entry.update_state(change_kind);

            let change = RtpsReaderCacheChange {
                kind: change_kind,
                writer_guid,
                data,
                source_timestamp,
                sample_state: SampleStateKind::NotRead,
                disposed_generation_count: instance_entry.most_recent_disposed_generation_count,
                no_writers_generation_count: instance_entry.most_recent_no_writers_generation_count,
                reception_timestamp,
            };

            self.changes.push(change);

            self.instance_reception_time
                .insert(change_instance_handle, reception_timestamp);

            match self.qos.destination_order.kind {
                DestinationOrderQosPolicyKind::BySourceTimestamp => {
                    self.changes.sort_by(|a, b| {
                        a.source_timestamp
                            .as_ref()
                            .expect("Missing source timestamp")
                            .cmp(
                                b.source_timestamp
                                    .as_ref()
                                    .expect("Missing source timestamp"),
                            )
                    });
                }
                DestinationOrderQosPolicyKind::ByReceptionTimestamp => self
                    .changes
                    .sort_by(|a, b| a.reception_timestamp.cmp(&b.reception_timestamp)),
            }

            self.data_available = true;

            Ok(())
        } else {
            Err(DdsError::OutOfResources)
        }
    }

    pub fn get_qos(&self) -> &DataReaderQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: DataReaderQos) -> DdsResult<()> {
        qos.is_consistent()?;
        self.qos = qos;
        Ok(())
    }

    fn create_indexed_sample_collection<Foo>(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<(usize, Sample<Foo>)>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        let mut indexed_samples = Vec::new();

        let instance_handle_build = &self.instance_handle_builder;
        let instances = &self.instances;
        let mut instances_in_collection = HashMap::new();
        for (index, cache_change) in self
            .changes
            .iter_mut()
            .filter(|cc| {
                let sample_instance_handle = instance_handle_build
                    .build_instance_handle(cc.kind, cc.data.as_slice())
                    .unwrap();

                sample_states.contains(&cc.sample_state)
                    && view_states.contains(&instances[&sample_instance_handle].view_state)
                    && instance_states.contains(&instances[&sample_instance_handle].instance_state)
            })
            .enumerate()
            .take(max_samples as usize)
        {
            let sample_instance_handle = self
                .instance_handle_builder
                .build_instance_handle(cache_change.kind, cache_change.data.as_slice())
                .unwrap();
            instances_in_collection
                .entry(sample_instance_handle)
                .or_insert_with(Instance::new);

            instances_in_collection
                .get_mut(&sample_instance_handle)
                .unwrap()
                .update_state(cache_change.kind);
            let sample_state = cache_change.sample_state;
            let view_state = self.instances[&sample_instance_handle].view_state;
            let instance_state = self.instances[&sample_instance_handle].instance_state;

            let absolute_generation_rank = (self.instances[&sample_instance_handle]
                .most_recent_disposed_generation_count
                + self.instances[&sample_instance_handle].most_recent_no_writers_generation_count)
                - (instances_in_collection[&sample_instance_handle]
                    .most_recent_disposed_generation_count
                    + instances_in_collection[&sample_instance_handle]
                        .most_recent_no_writers_generation_count);

            let (data, valid_data) = match cache_change.kind {
                ChangeKind::Alive | ChangeKind::AliveFiltered => (
                    Some(DdsDeserialize::deserialize(
                        &mut cache_change.data.as_slice(),
                    )?),
                    true,
                ),
                ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => (None, false),
            };

            let sample_info = SampleInfo {
                sample_state,
                view_state,
                instance_state,
                disposed_generation_count: cache_change.disposed_generation_count,
                no_writers_generation_count: cache_change.no_writers_generation_count,
                sample_rank: 0,     // To be filled up after collection is created
                generation_rank: 0, // To be filled up after collection is created
                absolute_generation_rank,
                source_timestamp: cache_change.source_timestamp,
                instance_handle: sample_instance_handle,
                publication_handle: cache_change.writer_guid.into(),
                valid_data,
            };

            let sample = Sample { data, sample_info };

            indexed_samples.push((index, sample))
        }

        // After the collection is created, update the relative generation rank values and mark the read instances as viewed
        for handle in instances_in_collection.into_keys() {
            let most_recent_sample_absolute_generation_rank = indexed_samples
                .iter()
                .filter(|(_, s)| s.sample_info.instance_handle == handle)
                .map(|(_, s)| s.sample_info.absolute_generation_rank)
                .last()
                .expect("Instance handle must exist on collection");

            let mut total_instance_samples_in_collection = indexed_samples
                .iter()
                .filter(|(_, s)| s.sample_info.instance_handle == handle)
                .count();

            for (_, sample) in indexed_samples
                .iter_mut()
                .filter(|(_, s)| s.sample_info.instance_handle == handle)
            {
                sample.sample_info.generation_rank = sample.sample_info.absolute_generation_rank
                    - most_recent_sample_absolute_generation_rank;

                total_instance_samples_in_collection -= 1;
                sample.sample_info.sample_rank = total_instance_samples_in_collection as i32;
            }

            self.instances
                .get_mut(&handle)
                .expect("Sample must exist on hash map")
                .mark_viewed()
        }

        self.data_available = false;

        if indexed_samples.is_empty() {
            Err(DdsError::NoData)
        } else {
            Ok(indexed_samples)
        }
    }

    pub fn read<Foo>(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.status_condition
            .remove_communication_state(StatusKind::DataAvailable);

        let indexed_sample_list = self.create_indexed_sample_collection::<Foo>(
            max_samples,
            sample_states,
            view_states,
            instance_states,
        )?;

        let change_index_list: Vec<usize>;
        let samples;

        (change_index_list, samples) = indexed_sample_list.into_iter().map(|(i, s)| (i, s)).unzip();

        for index in change_index_list {
            self.changes[index].sample_state = SampleStateKind::Read;
        }

        Ok(samples)
    }

    pub fn take<Foo>(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.status_condition
            .remove_communication_state(StatusKind::DataAvailable);

        let indexed_sample_list = self.create_indexed_sample_collection::<Foo>(
            max_samples,
            sample_states,
            view_states,
            instance_states,
        )?;

        let mut change_index_list: Vec<usize>;
        let samples;

        (change_index_list, samples) = indexed_sample_list.into_iter().map(|(i, s)| (i, s)).unzip();

        while let Some(index) = change_index_list.pop() {
            self.changes.remove(index);
        }

        Ok(samples)
    }

    pub fn take_data_available(&mut self) -> bool {
        let data_available = self.data_available;
        self.data_available = false;
        data_available
    }

    pub fn get_deadline_missed_instances(&mut self, now: Time) -> Vec<InstanceHandle> {
        let (missed_deadline_instances, instance_reception_time) = self
            .instance_reception_time
            .iter()
            .partition(|&(_, received_time)| now - *received_time > self.qos.deadline.period);

        self.instance_reception_time = instance_reception_time;

        missed_deadline_instances.iter().map(|(i, _)| *i).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        implementation::rtps::{
            messages::submessage_elements::{
                EntityIdSubmessageElement, Parameter, ParameterListSubmessageElement,
                SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
            },
            types::{
                SequenceNumber, TopicKind, ENTITYID_UNKNOWN, GUIDPREFIX_UNKNOWN, GUID_UNKNOWN,
            },
        },
        infrastructure::{
            error::DdsError,
            qos_policy::{HistoryQosPolicy, ResourceLimitsQosPolicy},
            time::{DURATION_ZERO, TIME_INVALID},
        },
        subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        topic_definition::type_support::{DdsSerde, DdsSerialize, DdsType, LittleEndian},
    };

    use super::*;

    fn to_bytes_le<S: DdsSerialize>(value: &S) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        value.serialize::<_, LittleEndian>(&mut writer).unwrap();
        writer
    }

    fn create_data_submessage_for_alive_change(
        data: &[u8],
        sequence_number: SequenceNumber,
    ) -> DataSubmessage {
        DataSubmessage {
            endianness_flag: false,
            inline_qos_flag: false,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN.into(),
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN.into(),
            },
            writer_sn: SequenceNumberSubmessageElement {
                value: sequence_number,
            },
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement { value: data },
        }
    }

    fn create_data_submessage_for_disposed_change(
        data: &[u8],
        sequence_number: SequenceNumber,
    ) -> DataSubmessage {
        DataSubmessage {
            endianness_flag: false,
            inline_qos_flag: false,
            data_flag: false,
            key_flag: true,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN.into(),
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN.into(),
            },
            writer_sn: SequenceNumberSubmessageElement {
                value: sequence_number,
            },
            inline_qos: ParameterListSubmessageElement {
                parameter: vec![Parameter {
                    parameter_id: 0x71,
                    length: 4,
                    value: &[0, 0, 0, 1],
                }],
            },
            serialized_payload: SerializedDataSubmessageElement { value: data },
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct KeyedType {
        key: u8,
        data: [u8; 5],
    }

    impl DdsType for KeyedType {
        fn type_name() -> &'static str {
            "KeyedType"
        }

        fn has_key() -> bool {
            true
        }

        fn get_serialized_key<E: crate::topic_definition::type_support::Endianness>(
            &self,
        ) -> Vec<u8> {
            vec![self.key]
        }

        fn set_key_fields_from_serialized_key<
            E: crate::topic_definition::type_support::Endianness,
        >(
            &mut self,
            key: &[u8],
        ) -> DdsResult<()> {
            self.key = key[0];
            Ok(())
        }
    }

    impl DdsSerde for KeyedType {}

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct UnkeyedType {
        data: [u8; 5],
    }

    impl DdsType for UnkeyedType {
        fn type_name() -> &'static str {
            "UnkeyedType"
        }
    }

    impl DdsSerde for UnkeyedType {}

    #[test]
    fn reader_no_key_add_change_keep_last_1() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast,
                depth: 1,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<UnkeyedType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);
        let data1 = UnkeyedType { data: [1; 5] };
        let data2 = UnkeyedType { data: [2; 5] };

        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data1), 1),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data2), 2),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();

        let samples = reader
            .read::<UnkeyedType>(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].data.as_ref(), Some(&data2));
    }

    #[test]
    fn reader_with_key_add_change_keep_last_1() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast,
                depth: 1,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<KeyedType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let data1_instance1 = KeyedType {
            key: 1,
            data: [1; 5],
        };
        let data2_instance1 = KeyedType {
            key: 1,
            data: [2; 5],
        };

        let data1_instance2 = KeyedType {
            key: 2,
            data: [1; 5],
        };

        let data2_instance2 = KeyedType {
            key: 2,
            data: [2; 5],
        };

        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data1_instance1), 1),
                Some(Time { sec: 1, nanosec: 0 }),
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data2_instance1), 2),
                Some(Time { sec: 1, nanosec: 0 }),
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();

        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data1_instance2), 3),
                Some(Time { sec: 1, nanosec: 0 }),
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data2_instance2), 4),
                Some(Time { sec: 1, nanosec: 0 }),
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();

        let samples = reader
            .read::<KeyedType>(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();

        assert_eq!(samples.len(), 2);

        // Last sample of each instance exists
        assert!(samples
            .iter()
            .any(|s| s.data.as_ref() == Some(&data2_instance1)));
        assert!(samples
            .iter()
            .any(|s| s.data.as_ref() == Some(&data2_instance2)));

        // First sample of each instance does not exist
        assert!(!samples
            .iter()
            .any(|s| s.data.as_ref() == Some(&data1_instance1)));
        assert!(!samples
            .iter()
            .any(|s| s.data.as_ref() == Some(&data1_instance2)));
    }

    #[test]
    fn reader_no_key_add_change_keep_last_3() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast,
                depth: 3,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<UnkeyedType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let data1 = UnkeyedType { data: [1; 5] };
        let data2 = UnkeyedType { data: [2; 5] };
        let data3 = UnkeyedType { data: [3; 5] };
        let data4 = UnkeyedType { data: [4; 5] };

        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data1), 1),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data2), 2),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data3), 3),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data4), 4),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();

        let samples = reader
            .read::<UnkeyedType>(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();

        assert_eq!(samples.len(), 3);

        assert!(samples.iter().any(|s| s.data.as_ref() == Some(&data2)));
        assert!(samples.iter().any(|s| s.data.as_ref() == Some(&data3)));
        assert!(samples.iter().any(|s| s.data.as_ref() == Some(&data4)));
    }

    #[test]
    fn reader_with_key_add_change_keep_last_3() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast,
                depth: 3,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<KeyedType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let data1_instance1 = KeyedType {
            key: 1,
            data: [1; 5],
        };
        let data2_instance1 = KeyedType {
            key: 1,
            data: [2; 5],
        };
        let data3_instance1 = KeyedType {
            key: 1,
            data: [3; 5],
        };
        let data4_instance1 = KeyedType {
            key: 1,
            data: [4; 5],
        };

        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data1_instance1), 1),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data2_instance1), 1),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data3_instance1), 1),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data4_instance1), 1),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();

        let data1_instance2 = KeyedType {
            key: 2,
            data: [1; 5],
        };
        let data2_instance2 = KeyedType {
            key: 2,
            data: [2; 5],
        };
        let data3_instance2 = KeyedType {
            key: 2,
            data: [3; 5],
        };
        let data4_instance2 = KeyedType {
            key: 2,
            data: [4; 5],
        };

        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data1_instance2), 1),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data2_instance2), 1),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data3_instance2), 1),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(&to_bytes_le(&data4_instance2), 1),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();

        let samples = reader
            .read::<KeyedType>(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();

        assert_eq!(samples.len(), 6);
        assert!(samples
            .iter()
            .any(|s| s.data.as_ref() == Some(&data2_instance1)));
        assert!(samples
            .iter()
            .any(|s| s.data.as_ref() == Some(&data3_instance1)));
        assert!(samples
            .iter()
            .any(|s| s.data.as_ref() == Some(&data4_instance1)));
        assert!(samples
            .iter()
            .any(|s| s.data.as_ref() == Some(&data2_instance2)));
        assert!(samples
            .iter()
            .any(|s| s.data.as_ref() == Some(&data3_instance2)));
        assert!(samples
            .iter()
            .any(|s| s.data.as_ref() == Some(&data4_instance2)));
    }

    #[test]
    fn reader_max_samples() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAll,
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
            RtpsReader::new::<UnkeyedType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&UnkeyedType { data: [1; 5] }),
                    1,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();

        assert_eq!(
            reader.add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&UnkeyedType { data: [1; 5] }),
                    1,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            ),
            Err(DdsError::OutOfResources)
        );
    }

    #[test]
    fn reader_max_instances() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAll,
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
            RtpsReader::new::<KeyedType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [1; 5],
                    }),
                    1,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();

        assert_eq!(
            reader.add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 2,
                        data: [1; 5],
                    }),
                    1
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            ),
            Err(DdsError::OutOfResources)
        );
    }

    #[test]
    fn reader_max_samples_per_instance() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAll,
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
            RtpsReader::new::<KeyedType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [1; 5],
                    }),
                    1,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 2,
                        data: [1; 5],
                    }),
                    1,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();

        assert_eq!(
            reader.add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 2,
                        data: [2; 5],
                    }),
                    1
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            ),
            Err(DdsError::OutOfResources)
        );
    }

    #[test]
    fn reader_sample_info_absolute_generation_rank() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAll,
                depth: LENGTH_UNLIMITED,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<KeyedType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [1; 5],
                    }),
                    1,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [2; 5],
                    }),
                    2,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_disposed_change(
                    &KeyedType {
                        key: 1,
                        data: [0; 5],
                    }
                    .get_serialized_key::<LittleEndian>(),
                    3,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [4; 5],
                    }),
                    4,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_disposed_change(
                    &KeyedType {
                        key: 1,
                        data: [0; 5],
                    }
                    .get_serialized_key::<LittleEndian>(),
                    5,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [6; 5],
                    }),
                    6,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();

        let samples = reader
            .read::<KeyedType>(10, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();

        assert_eq!(samples.len(), 6);
        assert_eq!(samples[0].sample_info.absolute_generation_rank, 2);
        assert_eq!(samples[1].sample_info.absolute_generation_rank, 2);
        assert_eq!(samples[2].sample_info.absolute_generation_rank, 2);
        assert_eq!(samples[3].sample_info.absolute_generation_rank, 1);
        assert_eq!(samples[4].sample_info.absolute_generation_rank, 1);
        assert_eq!(samples[5].sample_info.absolute_generation_rank, 0);
    }

    #[test]
    fn reader_sample_info_generation_rank_and_count() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAll,
                depth: LENGTH_UNLIMITED,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<KeyedType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [1; 5],
                    }),
                    1,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [2; 5],
                    }),
                    2,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_disposed_change(
                    &KeyedType {
                        key: 1,
                        data: [0; 5],
                    }
                    .get_serialized_key::<LittleEndian>(),
                    2,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [4; 5],
                    }),
                    2,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_disposed_change(
                    &KeyedType {
                        key: 1,
                        data: [0; 5],
                    }
                    .get_serialized_key::<LittleEndian>(),
                    2,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [6; 5],
                    }),
                    2,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();

        let samples = reader
            .read::<KeyedType>(4, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();

        assert_eq!(samples.len(), 4);
        assert_eq!(samples[0].sample_info.absolute_generation_rank, 2);
        assert_eq!(samples[0].sample_info.generation_rank, 1);
        assert_eq!(samples[0].sample_info.disposed_generation_count, 0);
        assert_eq!(samples[0].sample_info.no_writers_generation_count, 0);

        assert_eq!(samples[1].sample_info.absolute_generation_rank, 2);
        assert_eq!(samples[1].sample_info.generation_rank, 1);
        assert_eq!(samples[1].sample_info.disposed_generation_count, 0);
        assert_eq!(samples[1].sample_info.no_writers_generation_count, 0);

        assert_eq!(samples[2].sample_info.absolute_generation_rank, 2);
        assert_eq!(samples[2].sample_info.generation_rank, 1);
        assert_eq!(samples[2].sample_info.disposed_generation_count, 0);
        assert_eq!(samples[2].sample_info.no_writers_generation_count, 0);

        assert_eq!(samples[3].sample_info.absolute_generation_rank, 1);
        assert_eq!(samples[3].sample_info.generation_rank, 0);
        assert_eq!(samples[3].sample_info.disposed_generation_count, 1);
        assert_eq!(samples[3].sample_info.no_writers_generation_count, 0);
    }

    #[test]
    fn reader_sample_info_sample_rank() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAll,
                depth: LENGTH_UNLIMITED,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]);
        let mut reader =
            RtpsReader::new::<KeyedType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [1; 5],
                    }),
                    1,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [2; 5],
                    }),
                    2,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [3; 5],
                    }),
                    2,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();
        reader
            .add_change(
                &create_data_submessage_for_alive_change(
                    &to_bytes_le(&KeyedType {
                        key: 1,
                        data: [4; 5],
                    }),
                    2,
                ),
                None,
                GUIDPREFIX_UNKNOWN,
                TIME_INVALID,
            )
            .unwrap();

        let samples = reader
            .read::<KeyedType>(3, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();

        assert_eq!(samples.len(), 3);
        assert_eq!(samples[0].sample_info.sample_rank, 2);
        assert_eq!(samples[1].sample_info.sample_rank, 1);
        assert_eq!(samples[2].sample_info.sample_rank, 0);
    }
}
