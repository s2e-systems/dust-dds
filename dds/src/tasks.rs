use std::{
    ops::Deref,
    sync::{
        atomic::{self, AtomicBool},
        mpsc::{Receiver, SyncSender},
        Arc,
    },
};

use async_std::prelude::StreamExt;
use rust_dds_api::{subscription::{data_reader::DataReader}};
use rust_dds_rtps_implementation::{
    dds_impl::{
        data_reader_proxy::{RtpsReader, Samples},
        subscriber_proxy::SubscriberAttributes,
    },
    rtps_impl::rtps_writer_proxy_impl::RtpsWriterProxyImpl,
    utils::shared_object::RtpsShared, data_representation_builtin_endpoints::{spdp_discovered_participant_data::SpdpDiscoveredParticipantData, sedp_discovered_writer_data::SedpDiscoveredWriterData},
};
use rust_rtps_pim::{
    behavior::{
        reader::{
            stateful_reader::RtpsStatefulReaderOperations, writer_proxy::RtpsWriterProxyConstructor,
        },
        writer::{
            reader_proxy::RtpsReaderProxyConstructor, stateful_writer::RtpsStatefulWriterOperations,
        },
    },
    discovery::participant_discovery::ParticipantDiscovery,
};

use crate::domain_participant_factory::{RtpsStructureImpl};

pub struct Executor {
    pub receiver: Receiver<EnabledPeriodicTask>,
}

impl Executor {
    pub fn run(&self) {
        while let Ok(mut enabled_periodic_task) = self.receiver.try_recv() {
            async_std::task::spawn(async move {
                let mut interval = async_std::stream::interval(enabled_periodic_task.period);
                loop {
                    if enabled_periodic_task.enabled.load(atomic::Ordering::SeqCst) {
                        (enabled_periodic_task.task)();
                    } else {
                        println!("Task not enabled: {}", enabled_periodic_task.name);
                    }
                    interval.next().await;
                }
            });
        }
    }
}

#[derive(Clone)]
pub struct Spawner {
    pub task_sender: SyncSender<EnabledPeriodicTask>,
    pub enabled: Arc<AtomicBool>,
}

impl Spawner {
    pub fn new(task_sender: SyncSender<EnabledPeriodicTask>) -> Self {
        Self {
            task_sender,
            enabled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn spawn_enabled_periodic_task(
        &self,
        name: &'static str,
        task: impl FnMut() -> () + Send + Sync + 'static,
        period: std::time::Duration,
    ) {
        self.task_sender
            .send(EnabledPeriodicTask {
                name,
                task: Box::new(task),
                period,
                enabled: self.enabled.clone(),
            })
            .unwrap();
    }

    pub fn enable_tasks(&self) {
        self.enabled.store(true, atomic::Ordering::SeqCst);
    }

    pub fn _disable_tasks(&self) {
        self.enabled.store(false, atomic::Ordering::SeqCst);
    }
}

pub struct EnabledPeriodicTask {
    pub name: &'static str,
    pub task: Box<dyn FnMut() -> () + Send + Sync>,
    pub period: std::time::Duration,
    pub enabled: Arc<AtomicBool>,
}

pub fn task_spdp_discovery<T>(
    spdp_builtin_participant_data_reader:
        &mut impl DataReader<SpdpDiscoveredParticipantData, Samples = T>,
    domain_id: u32,
    domain_tag: &str,
    sedp_builtin_publications_writer: &mut impl RtpsStatefulWriterOperations<
        ReaderProxyType = impl RtpsReaderProxyConstructor,
    >,
    sedp_builtin_publication_reader: &mut impl RtpsStatefulReaderOperations<
        WriterProxyType = impl RtpsWriterProxyConstructor,
    >,
    sedp_builtin_subscriptions_writer: &mut impl RtpsStatefulWriterOperations<
        ReaderProxyType = impl RtpsReaderProxyConstructor,
    >,
    sedp_builtin_subscriptions_reader: &mut impl RtpsStatefulReaderOperations<
        WriterProxyType = impl RtpsWriterProxyConstructor,
    >,
    sedp_builtin_topics_writer: &mut impl RtpsStatefulWriterOperations<
        ReaderProxyType = impl RtpsReaderProxyConstructor,
    >,
    sedp_builtin_topics_reader: &mut impl RtpsStatefulReaderOperations<
        WriterProxyType = impl RtpsWriterProxyConstructor,
    >,
) where
    T: Deref<Target = [SpdpDiscoveredParticipantData]>,
{

    if let Ok(samples) = spdp_builtin_participant_data_reader.read(1, &[], &[], &[]) {
        for discovered_participant in samples.into_iter() {
            if let Ok(participant_discovery) = ParticipantDiscovery::new(
                &discovered_participant.participant_proxy,
                &(domain_id as u32),
                domain_tag,
            ) {
                participant_discovery.discovered_participant_add_publications_writer(
                    sedp_builtin_publications_writer,
                );

                participant_discovery.discovered_participant_add_publications_reader(
                    sedp_builtin_publication_reader,
                );

                participant_discovery.discovered_participant_add_subscriptions_writer(
                    sedp_builtin_subscriptions_writer,
                );

                participant_discovery.discovered_participant_add_subscriptions_reader(
                    sedp_builtin_subscriptions_reader,
                );

                participant_discovery.discovered_participant_add_topics_writer(
                    sedp_builtin_topics_writer
                );

                participant_discovery.discovered_participant_add_topics_reader(
                    sedp_builtin_topics_reader
                );
            }
        }
    }
}

pub fn task_sedp_discovery(
    sedp_builtin_publications_data_reader:
        &mut impl DataReader<SedpDiscoveredWriterData, Samples = Samples<SedpDiscoveredWriterData>>,
    subscriber_list: &Vec<RtpsShared<SubscriberAttributes<RtpsStructureImpl>>>,
) {
    if let Ok(samples) = sedp_builtin_publications_data_reader.read(1, &[], &[], &[]) {
        if let Some(sample) = samples.into_iter().next() {
            let topic_name = &sample.publication_builtin_topic_data.topic_name;
            let type_name = &sample.publication_builtin_topic_data.type_name;
            for subscriber in subscriber_list {
                let subscriber_lock = subscriber.read_lock();
                for data_reader in subscriber_lock.data_reader_list.iter() {
                    let mut data_reader_lock = data_reader.write_lock();
                    let reader_topic_name = &data_reader_lock.topic.read_lock().topic_name.clone();
                    let reader_type_name = data_reader_lock.topic.read_lock().type_name;
                    if topic_name == reader_topic_name && type_name == reader_type_name {
                        let writer_proxy = RtpsWriterProxyImpl::new(
                            sample.writer_proxy.remote_writer_guid,
                            sample.writer_proxy.unicast_locator_list.as_ref(),
                            sample.writer_proxy.multicast_locator_list.as_ref(),
                            sample.writer_proxy.data_max_size_serialized,
                            sample.writer_proxy.remote_group_entity_id,
                        );
                        match &mut data_reader_lock.rtps_reader {
                            RtpsReader::Stateless(_) => (),
                            RtpsReader::Stateful(rtps_stateful_reader) => {
                                rtps_stateful_reader.matched_writer_add(writer_proxy)
                            }
                        };
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::{mock, predicate};
    use rust_dds_api::{
        subscription::{data_reader::DataReader, query_condition::QueryCondition, subscriber::SubscriberDataReaderFactory},
        dcps_psm::{
            SampleStateKind, ViewStateKind, InstanceStateKind, InstanceHandle,
            LivelinessChangedStatus, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleLostStatus,
            SampleRejectedStatus, SubscriptionMatchedStatus, BuiltInTopicKey,
            Duration, DomainId
        },
        return_type::DDSResult,
        infrastructure::{
            sample_info::SampleInfo,
            read_condition::ReadCondition,
            qos_policy::{
                DurabilityQosPolicy, DurabilityServiceQosPolicy,
                DeadlineQosPolicy, LatencyBudgetQosPolicy,
                LivelinessQosPolicy, ReliabilityQosPolicy,
                ReliabilityQosPolicyKind, LifespanQosPolicy, UserDataQosPolicy,
                OwnershipQosPolicy, OwnershipStrengthQosPolicy,
                DestinationOrderQosPolicy, PresentationQosPolicy,
                PartitionQosPolicy, TopicDataQosPolicy, GroupDataQosPolicy
            },
            qos::{SubscriberQos, TopicQos, DomainParticipantQos, PublisherQos, DataWriterQos}
        },
        builtin_topics::{PublicationBuiltinTopicData, ParticipantBuiltinTopicData}
    };
    use rust_dds_rtps_implementation::{
        rtps_impl::{
            rtps_writer_proxy_impl::RtpsWriterProxyImpl,
            rtps_reader_proxy_impl::RtpsReaderProxyAttributesImpl,
            rtps_group_impl::RtpsGroupImpl
        },
        dds_impl::{data_reader_proxy::Samples, subscriber_proxy::{SubscriberAttributes, SubscriberProxy}, topic_proxy::{TopicAttributes, TopicProxy}, domain_participant_proxy::{DomainParticipantProxy, DomainParticipantAttributes}, publisher_proxy::PublisherAttributes, data_writer_proxy::{DataWriterAttributes, RtpsWriter}},
        utils::shared_object::{RtpsWeak, RtpsShared}, dds_type::{DdsType, DdsDeserialize}, data_representation_builtin_endpoints::{spdp_discovered_participant_data::{SpdpDiscoveredParticipantData, ParticipantProxy}, sedp_discovered_writer_data::{SedpDiscoveredWriterData, RtpsWriterProxy}, sedp_discovered_reader_data::{SedpDiscoveredReaderData, DCPS_SUBSCRIPTION}}
    };
    use rust_rtps_pim::{
        structure::{
            types::{
                Guid, PROTOCOLVERSION, GuidPrefix, VENDOR_ID_S2E,
                ENTITYID_UNKNOWN, EntityId, BUILT_IN_READER_GROUP, GUID_UNKNOWN
            },
            group::RtpsGroupConstructor
        },
        behavior::{
            reader::{
                stateful_reader::RtpsStatefulReaderOperations,
                writer_proxy::RtpsWriterProxyConstructor
            },
            writer::{
                stateful_writer::RtpsStatefulWriterOperations,
                reader_proxy::RtpsReaderProxyConstructor
            }
        },
        discovery::{
            types::{BuiltinEndpointSet, BuiltinEndpointQos},
            sedp::builtin_endpoints::{
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER, SedpBuiltinSubscriptionsWriter
            }
        },
        messages::types::Count
    };

    use crate::domain_participant_factory::RtpsStructureImpl;

    use super::{task_spdp_discovery, task_sedp_discovery};

    mock! {
        DdsDataReader<Foo: 'static>{}

        impl<Foo> DataReader<Foo> for DdsDataReader<Foo>{
            type Samples = Samples<Foo>;
            type Subscriber = ();
            type TopicDescription = ();
            fn read(
                &mut self,
                max_samples: i32,
                sample_states: &[SampleStateKind],
                view_states: &[ViewStateKind],
                instance_states: &[InstanceStateKind],
            ) -> DDSResult<Samples<Foo>>;

            fn take(
                &mut self,
                max_samples: i32,
                sample_states: &[SampleStateKind],
                view_states: &[ViewStateKind],
                instance_states: &[InstanceStateKind],
            ) -> DDSResult<Samples<Foo>>;

            fn read_w_condition(
                &self,
                data_values: &mut [Foo],
                sample_infos: &mut [SampleInfo],
                max_samples: i32,
                a_condition: ReadCondition,
            ) -> DDSResult<()>;

            fn take_w_condition(
                &self,
                data_values: &mut [Foo],
                sample_infos: &mut [SampleInfo],
                max_samples: i32,
                a_condition: ReadCondition,
            ) -> DDSResult<()>;

            fn read_next_sample(
                &self,
                data_value: &mut [Foo],
                sample_info: &mut [SampleInfo],
            ) -> DDSResult<()>;

            fn take_next_sample(
                &self,
                data_value: &mut [Foo],
                sample_info: &mut [SampleInfo],
            ) -> DDSResult<()>;

            fn read_instance(
                &self,
                data_values: &mut [Foo],
                sample_infos: &mut [SampleInfo],
                max_samples: i32,
                a_handle: InstanceHandle,
                sample_states: &[SampleStateKind],
                view_states: &[ViewStateKind],
                instance_states: &[InstanceStateKind],
            ) -> DDSResult<()>;

            fn take_instance(
                &self,
                data_values: &mut [Foo],
                sample_infos: &mut [SampleInfo],
                max_samples: i32,
                a_handle: InstanceHandle,
                sample_states: &[SampleStateKind],
                view_states: &[ViewStateKind],
                instance_states: &[InstanceStateKind],
            ) -> DDSResult<()>;

            fn read_next_instance(
                &self,
                data_values: &mut [Foo],
                sample_infos: &mut [SampleInfo],
                max_samples: i32,
                previous_handle: InstanceHandle,
                sample_states: &[SampleStateKind],
                view_states: &[ViewStateKind],
                instance_states: &[InstanceStateKind],
            ) -> DDSResult<()>;

            fn take_next_instance(
                &self,
                data_values: &mut [Foo],
                sample_infos: &mut [SampleInfo],
                max_samples: i32,
                previous_handle: InstanceHandle,
                sample_states: &[SampleStateKind],
                view_states: &[ViewStateKind],
                instance_states: &[InstanceStateKind],
            ) -> DDSResult<()>;

            fn read_next_instance_w_condition(
                &self,
                data_values: &mut [Foo],
                sample_infos: &mut [SampleInfo],
                max_samples: i32,
                previous_handle: InstanceHandle,
                a_condition: ReadCondition,
            ) -> DDSResult<()>;


            fn take_next_instance_w_condition(
                &self,
                data_values: &mut [Foo],
                sample_infos: &mut [SampleInfo],
                max_samples: i32,
                previous_handle: InstanceHandle,
                a_condition: ReadCondition,
            ) -> DDSResult<()>;


            fn return_loan(
                &self,
                data_values: &mut [Foo],
                sample_infos: &mut [SampleInfo],
            ) -> DDSResult<()>;

            fn get_key_value(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DDSResult<()>;


            fn lookup_instance(&self, instance: &Foo) -> InstanceHandle;


            fn create_readcondition(
                &self,
                sample_states: &[SampleStateKind],
                view_states: &[ViewStateKind],
                instance_states: &[InstanceStateKind],
            ) -> ReadCondition;


            fn create_querycondition(
                &self,
                sample_states: &[SampleStateKind],
                view_states: &[ViewStateKind],
                instance_states: &[InstanceStateKind],
                query_expression: &'static str,
                query_parameters: &[&'static str],
            ) -> QueryCondition;


            fn delete_readcondition(&self, a_condition: ReadCondition) -> DDSResult<()>;

            fn get_liveliness_changed_status(&self, status: &mut LivelinessChangedStatus) -> DDSResult<()>;


            fn get_requested_deadline_missed_status(
                &self,
                status: &mut RequestedDeadlineMissedStatus,
            ) -> DDSResult<()>;


            fn get_requested_incompatible_qos_status(
                &self,
                status: &mut RequestedIncompatibleQosStatus,
            ) -> DDSResult<()>;

            fn get_sample_lost_status(&self, status: &mut SampleLostStatus) -> DDSResult<()>;


            fn get_sample_rejected_status(&self, status: &mut SampleRejectedStatus) -> DDSResult<()>;


            fn get_subscription_matched_status(
                &self,
                status: &mut SubscriptionMatchedStatus,
            ) -> DDSResult<()>;


            fn get_topicdescription(&self) -> DDSResult<()>;

            fn get_subscriber(&self) -> DDSResult<()>;


            fn delete_contained_entities(&self) -> DDSResult<()>;

            fn wait_for_historical_data(&self) -> DDSResult<()>;


            fn get_matched_publication_data(
                &self,
                publication_data: &mut PublicationBuiltinTopicData,
                publication_handle: InstanceHandle,
            ) -> DDSResult<()>;

            fn get_match_publication(&self, publication_handles: &mut [InstanceHandle]) -> DDSResult<()>;
        }

    }

    mock! {
        StatefulReader {}

        impl RtpsStatefulReaderOperations for StatefulReader {
            type WriterProxyType = RtpsWriterProxyImpl;
            fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxyImpl);
            fn matched_writer_remove(&mut self, writer_proxy_guid: &Guid);
            fn matched_writer_lookup(&self, a_writer_guid: &Guid) -> Option<&'static RtpsWriterProxyImpl>;
        }
    }

    mock! {
        StatefulWriter {}

        impl RtpsStatefulWriterOperations for StatefulWriter {
            type ReaderProxyType = RtpsReaderProxyAttributesImpl;
            fn matched_reader_add(&mut self, a_reader_proxy: RtpsReaderProxyAttributesImpl);
            fn matched_reader_remove(&mut self, reader_proxy_guid: &Guid);
            fn matched_reader_lookup(&self, a_reader_guid: &Guid) -> Option<&'static RtpsReaderProxyAttributesImpl>;
            fn is_acked_by_all(&self) -> bool;
        }
    }

    fn make_participant() -> RtpsShared<DomainParticipantAttributes<RtpsStructureImpl>>
    {
        let domain_participant = RtpsShared::new(DomainParticipantAttributes::new(
            GuidPrefix([0; 12]),
            DomainId::default(),
            "".to_string(),
            DomainParticipantQos::default(),
            vec![],
            vec![],
            vec![],
            vec![],
        ));

        domain_participant.write_lock().builtin_publisher =
            Some(RtpsShared::new(PublisherAttributes::new(
                PublisherQos::default(),
                RtpsGroupImpl::new(GUID_UNKNOWN),
                domain_participant.downgrade(),
            )));

        let sedp_topic_subscription = RtpsShared::new(TopicAttributes::new(
            TopicQos::default(),
            SedpDiscoveredReaderData::type_name(),
            DCPS_SUBSCRIPTION,
            RtpsWeak::new(),
        ));

        domain_participant
            .write_lock()
            .topic_list
            .push(sedp_topic_subscription.clone());


        let sedp_builtin_subscriptions_rtps_writer =
            SedpBuiltinSubscriptionsWriter::create(GuidPrefix([0;12]), &[], &[]);
        let sedp_builtin_subscriptions_data_writer = RtpsShared::new(DataWriterAttributes::new(
            DataWriterQos::default(),
            RtpsWriter::Stateful(sedp_builtin_subscriptions_rtps_writer),
            sedp_topic_subscription.clone(),
            domain_participant.read_lock().builtin_publisher.as_ref().unwrap().downgrade(),
        ));
        domain_participant.read_lock().builtin_publisher.as_ref().unwrap()
            .write_lock()
            .data_writer_list
            .push(sedp_builtin_subscriptions_data_writer.clone());

        domain_participant
    }

    struct MyType;

    impl DdsType for MyType {
        fn type_name() -> &'static str {
            "MyType"
        }

        fn has_key() -> bool {
            false
        }
    }

    impl<'de> DdsDeserialize<'de> for MyType {
        fn deserialize(_buf: &mut &'de [u8]) -> DDSResult<Self> {
            Ok(MyType{})
        }
    }

    #[test]
    fn discovery_task_all_sedp_endpoints() {
        let mut mock_spdp_data_reader = MockDdsDataReader::new();
        mock_spdp_data_reader.expect_read().returning(|_, _, _, _| {
            Ok(Samples {
                samples: vec![SpdpDiscoveredParticipantData {
                    dds_participant_data: ParticipantBuiltinTopicData {
                        key: BuiltInTopicKey { value: [5; 16] },
                        user_data: rust_dds_api::infrastructure::qos_policy::UserDataQosPolicy {
                            value: vec![],
                        },
                    },
                    participant_proxy: ParticipantProxy {
                        domain_id: 1,
                        domain_tag: String::new(),
                        protocol_version: PROTOCOLVERSION,
                        guid_prefix: GuidPrefix([5; 12]),
                        vendor_id: VENDOR_ID_S2E,
                        expects_inline_qos: false,
                        metatraffic_unicast_locator_list: vec![],
                        metatraffic_multicast_locator_list: vec![],
                        default_unicast_locator_list: vec![],
                        default_multicast_locator_list: vec![],
                        available_builtin_endpoints: BuiltinEndpointSet(
                            BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER
                                | BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
                                | BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER
                                | BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR
                                | BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER
                                | BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR
                                | BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER
                                | BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR,
                        ),
                        manual_liveliness_count: Count(1),
                        builtin_endpoint_qos: BuiltinEndpointQos(0),
                    },
                    lease_duration: rust_rtps_pim::behavior::types::Duration {
                        seconds: 100,
                        fraction: 0,
                    },
                }],
            })
        });

        let mut mock_builtin_publications_writer = MockStatefulWriter::new();
        mock_builtin_publications_writer
            .expect_matched_reader_add()
            .with(predicate::eq(RtpsReaderProxyAttributesImpl::new(
                Guid::new(
                    GuidPrefix([5; 12]),
                    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                ),
                ENTITYID_UNKNOWN,
                &[],
                &[],
                false,
                true,
            )))
            .once()
            .return_const(());

        let mut mock_builtin_publications_reader = MockStatefulReader::new();
        mock_builtin_publications_reader
            .expect_matched_writer_add()
            .with(predicate::eq(RtpsWriterProxyImpl::new(
                Guid::new(
                    GuidPrefix([5; 12]),
                    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                ),
                &[],
                &[],
                None,
                ENTITYID_UNKNOWN,
            )))
            .once()
            .return_const(());

        let mut mock_builtin_subscriptions_writer = MockStatefulWriter::new();
        mock_builtin_subscriptions_writer
            .expect_matched_reader_add()
            .with(predicate::eq(RtpsReaderProxyAttributesImpl::new(
                Guid::new(
                    GuidPrefix([5; 12]),
                    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                ),
                ENTITYID_UNKNOWN,
                &[],
                &[],
                false,
                true,
            )))
            .once()
            .return_const(());

        let mut mock_builtin_subscriptions_reader = MockStatefulReader::new();
        mock_builtin_subscriptions_reader
            .expect_matched_writer_add()
            .with(predicate::eq(RtpsWriterProxyImpl::new(
                Guid::new(
                    GuidPrefix([5; 12]),
                    ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                ),
                &[],
                &[],
                None,
                ENTITYID_UNKNOWN,
            )))
            .once()
            .return_const(());

        let mut mock_builtin_topics_writer = MockStatefulWriter::new();
        mock_builtin_topics_writer
            .expect_matched_reader_add()
            .with(predicate::eq(RtpsReaderProxyAttributesImpl::new(
                Guid::new(GuidPrefix([5; 12]), ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR),
                ENTITYID_UNKNOWN,
                &[],
                &[],
                false,
                true,
            )))
            .once()
            .return_const(());

        let mut mock_builtin_topics_reader = MockStatefulReader::new();
        mock_builtin_topics_reader
            .expect_matched_writer_add()
            .with(predicate::eq(RtpsWriterProxyImpl::new(
                Guid::new(GuidPrefix([5; 12]), ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER),
                &[],
                &[],
                None,
                ENTITYID_UNKNOWN,
            )))
            .once()
            .return_const(());

        task_spdp_discovery(
            &mut mock_spdp_data_reader,
            1,
            "",
            &mut mock_builtin_publications_writer,
            &mut mock_builtin_publications_reader,
            &mut mock_builtin_subscriptions_writer,
            &mut mock_builtin_subscriptions_reader,
            &mut mock_builtin_topics_writer,
            &mut mock_builtin_topics_reader,
        );
    }

    #[test]
    fn task_sedp_discovery_() {
        let topic = RtpsShared::new(TopicAttributes {
            _qos: TopicQos::default(),
            type_name: MyType::type_name(),
            topic_name: "MyTopic".to_string(),
            parent_participant: RtpsWeak::new(),
        });

        let mut mock_sedp_discovered_writer_data_reader = MockDdsDataReader::new();
        mock_sedp_discovered_writer_data_reader
            .expect_read()
            .returning(|_, _, _, _| {
                Ok(Samples {
                    samples: vec![SedpDiscoveredWriterData {
                        writer_proxy: RtpsWriterProxy {
                            remote_writer_guid: Guid::new(
                                GuidPrefix([1; 12]),
                                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                            ),
                            unicast_locator_list: vec![],
                            multicast_locator_list: vec![],
                            data_max_size_serialized: None,
                            remote_group_entity_id: EntityId::new([0; 3], 0),
                        },
                        publication_builtin_topic_data: PublicationBuiltinTopicData {
                            key: BuiltInTopicKey { value: [1; 16] },
                            participant_key: BuiltInTopicKey { value: [1; 16] },
                            topic_name: "MyTopic".to_string(),
                            type_name: MyType::type_name().to_string(),
                            durability: DurabilityQosPolicy::default(),
                            durability_service: DurabilityServiceQosPolicy::default(),
                            deadline: DeadlineQosPolicy::default(),
                            latency_budget: LatencyBudgetQosPolicy::default(),
                            liveliness: LivelinessQosPolicy::default(),
                            reliability: ReliabilityQosPolicy {
                                kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
                                max_blocking_time: Duration::new(3, 0),
                            },
                            lifespan: LifespanQosPolicy::default(),
                            user_data: UserDataQosPolicy::default(),
                            ownership: OwnershipQosPolicy::default(),
                            ownership_strength: OwnershipStrengthQosPolicy::default(),
                            destination_order: DestinationOrderQosPolicy::default(),
                            presentation: PresentationQosPolicy::default(),
                            partition: PartitionQosPolicy::default(),
                            topic_data: TopicDataQosPolicy::default(),
                            group_data: GroupDataQosPolicy::default(),
                        },
                    }],
                })
            });

        let participant = make_participant();

        let subscriber = RtpsShared::new(SubscriberAttributes::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(Guid::new(
                GuidPrefix([0; 12]),
                EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
            )),
            participant.downgrade(),
        ));
        let subscriber_proxy = SubscriberProxy::new(
            DomainParticipantProxy::new(RtpsWeak::new()),
            subscriber.downgrade()
        );

        let reader = subscriber_proxy.datareader_factory_create_datareader(
            &TopicProxy::<MyType, _>::new(topic.downgrade()), None, None, 0
        ).unwrap();


        let subscriber_list = vec![subscriber];

        task_sedp_discovery(
            &mut mock_sedp_discovered_writer_data_reader,
            &subscriber_list,
        );

        let reader_shared = reader.as_ref().upgrade().unwrap();
        let mut reader_lock = reader_shared.write_lock();
        let stateful_reader = reader_lock.rtps_reader.try_as_stateful_reader().unwrap();

        assert!(stateful_reader.matched_writer_lookup(
                &Guid::new(
                    GuidPrefix([1; 12]),
                    ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                )
            ).is_some()
        );
    }
}