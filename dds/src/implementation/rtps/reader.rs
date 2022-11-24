use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::SyncSender,
};

use crate::{
    implementation::dds_impl::status_condition_impl::StatusConditionImpl,
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
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    endpoint::RtpsEndpoint,
    reader_cache_change::RtpsReaderCacheChange,
    types::{ChangeKind, Guid},
};

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

struct InstanceHandleBuilder(fn(&[u8]) -> DdsResult<Vec<u8>>);

impl InstanceHandleBuilder {
    fn new<Foo>() -> Self
    where
        Foo: for<'de> DdsDeserialize<'de> + DdsType,
    {
        Self(Foo::deserialize_key)
    }

    fn build_instance_handle(&self, data: &[u8]) -> DdsResult<InstanceHandle> {
        Ok((self.0)(data)?.as_slice().into())
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
    status_condition: StatusConditionImpl,
    instance_handle_builder: InstanceHandleBuilder,
    instances: HashMap<InstanceHandle, Instance>,
    notifications_sender: SyncSender<(Guid, StatusKind)>,
}

impl RtpsReader {
    pub fn new<Foo>(
        endpoint: RtpsEndpoint,
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
        qos: DataReaderQos,
        notifications_sender: SyncSender<(Guid, StatusKind)>,
    ) -> Self
    where
        Foo: DdsType + for<'de> DdsDeserialize<'de>,
    {
        let instance_handle_builder = InstanceHandleBuilder::new::<Foo>();
        Self {
            endpoint,
            _heartbeat_response_delay: heartbeat_response_delay,
            _heartbeat_suppression_duration: heartbeat_suppression_duration,
            reader_cache: ReaderHistoryCache::new(),
            _expects_inline_qos: expects_inline_qos,
            qos,
            status_condition: StatusConditionImpl::default(),
            instance_handle_builder,
            instances: HashMap::new(),
            notifications_sender,
        }
    }

    pub fn guid(&self) -> Guid {
        self.endpoint.guid()
    }

    pub fn add_change(&mut self, change: RtpsReaderCacheChange) -> DdsResult<()> {
        let change_instance_handle = self
            .instance_handle_builder
            .build_instance_handle(change.data_value())?;

        if self.qos.history.kind == HistoryQosPolicyKind::KeepLast
            && change.kind() == ChangeKind::Alive
        {
            let num_instance_samples = self
                .reader_cache
                .changes
                .iter()
                .filter(|cc| {
                    self.instance_handle_builder
                        .build_instance_handle(cc.data_value())
                        .unwrap()
                        == change_instance_handle
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
                        self.instance_handle_builder
                            .build_instance_handle(cc.data_value())
                            .unwrap()
                            == change_instance_handle
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
                self.instance_handle_builder
                    .build_instance_handle(cc.data_value())
                    .unwrap()
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
                    .reader_cache
                    .changes
                    .as_slice()
                    .iter()
                    .filter(|cc| {
                        self.instance_handle_builder
                            .build_instance_handle(cc.data_value())
                            .unwrap()
                            == change_instance_handle
                    })
                    .count() as i32)
                    < self.qos.resource_limits.max_samples_per_instance;

        if max_samples_limit_not_reached
            && max_instances_limit_not_reached
            && max_samples_per_instance_limit_not_reached
        {
            self.reader_cache.changes.push(change);

            self.notifications_sender
                .send((self.endpoint.guid(), StatusKind::DataAvailable))
                .ok();

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

        let instance_handle_builder = &self.instance_handle_builder;
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
                        instance_handle: instance_handle_builder
                            .build_instance_handle(cache_change.data_value())
                            .unwrap(),
                        publication_handle: cache_change.writer_guid().into(),
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

        let instance_handle_builder = &self.instance_handle_builder;
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
                        instance_handle: instance_handle_builder
                            .build_instance_handle(cache_change.data_value())
                            .unwrap(),
                        publication_handle: cache_change.writer_guid().into(),
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

#[cfg(test)]
mod tests {
    use std::sync::mpsc::sync_channel;

    use crate::{
        implementation::rtps::types::{ChangeKind, TopicKind, GUID_UNKNOWN},
        infrastructure::{
            error::DdsError,
            instance::HANDLE_NIL,
            qos_policy::{HistoryQosPolicy, ResourceLimitsQosPolicy},
            time::DURATION_ZERO,
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

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct KeyedType {
        key: u8,
        data: [u8; 5],
    }

    impl DdsType for KeyedType {
        fn type_name() -> &'static str {
            "MockType"
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

    // #[test]
    // fn reader_no_key_add_change_keep_last_1() {
    //     let qos = DataReaderQos {
    //         history: HistoryQosPolicy {
    //             kind: HistoryQosPolicyKind::KeepLast,
    //             depth: 1,
    //         },
    //         ..Default::default()
    //     };
    //     let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
    //     let mut reader =
    //         RtpsReader::new::<KeyedType>(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

    //     let change1 = RtpsReaderCacheChange::new(
    //         ChangeKind::Alive,
    //         GUID_UNKNOWN,
    //         1,
    //         vec![1],
    //         vec![],
    //         None,
    //         ViewStateKind::New,
    //     );
    //     let change2 = RtpsReaderCacheChange::new(
    //         ChangeKind::Alive,
    //         GUID_UNKNOWN,
    //         2,
    //         vec![1],
    //         vec![],
    //         None,
    //         ViewStateKind::New,
    //     );
    //     reader.add_change(change1).unwrap();
    //     reader.add_change(change2.clone()).unwrap();

    //     let samples = reader
    //         .read::<KeyedType>(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
    //         .unwrap();
    //     assert_eq!(samples.len(), 1);
    //     assert_eq!(reader.changes()[0], change2);
    // }

    #[test]
    fn reader_with_key_add_change_keep_last_1() {
        let (sender, _receiver) = sync_channel(10);

        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast,
                depth: 1,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]);
        let mut reader = RtpsReader::new::<KeyedType>(
            endpoint,
            DURATION_ZERO,
            DURATION_ZERO,
            false,
            qos,
            sender,
        );

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

        let change1_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            1,
            to_bytes_le(&data1_instance1),
            vec![],
            None,
            ViewStateKind::New,
        );
        let change2_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            2,
            to_bytes_le(&data2_instance1),
            vec![],
            None,
            ViewStateKind::New,
        );
        reader.add_change(change1_instance1).unwrap();
        reader.add_change(change2_instance1.clone()).unwrap();

        let change1_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            1,
            to_bytes_le(&data1_instance2),
            vec![],
            None,
            ViewStateKind::New,
        );
        let change2_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            2,
            to_bytes_le(&data2_instance2),
            vec![],
            None,
            ViewStateKind::New,
        );
        reader.add_change(change1_instance2).unwrap();
        reader.add_change(change2_instance2.clone()).unwrap();

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
}
