use super::actor::MailHandler;
use crate::{
    builtin_topics::{DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC},
    dcps::{
        actor::{Actor, ActorAddress},
        channels::{
            mpsc::{MpscReceiver, MpscSender, mpsc_channel},
            oneshot::oneshot,
        },
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        domain_participant::{
            DataReaderEntity, DataWriterEntity, DcpsDomainParticipant, DomainParticipantEntity,
            PublisherEntity, SubscriberEntity, TopicDescriptionKind, TopicEntity,
            TransportReaderKind, TransportWriterKind,
        },
        domain_participant_mail::{
            DcpsDomainParticipantMail, DiscoveryServiceMail, MessageServiceMail,
            ParticipantServiceMail,
        },
        listeners::domain_participant_listener::DcpsDomainParticipantListener,
        status_condition::DcpsStatusCondition,
    },
    dds_async::configuration::DustDdsConfiguration,
    infrastructure::{
        domain::DomainId,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantFactoryQos, DomainParticipantQos,
            PublisherQos, QosKind, SubscriberQos, TopicQos,
        },
        qos_policy::{
            BUILT_IN_DATA_REPRESENTATION, DataRepresentationQosPolicy, DurabilityQosPolicy,
            DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind,
        },
        status::StatusKind,
        time::{Duration, DurationKind},
        type_support::TypeSupport,
    },
    rtps::{
        stateful_reader::RtpsStatefulReader, stateful_writer::RtpsStatefulWriter,
        stateless_reader::RtpsStatelessReader, stateless_writer::RtpsStatelessWriter,
    },
    runtime::{DdsRuntime, Spawner, Timer},
    transport::{
        interface::{HistoryCache, TransportParticipantFactory},
        types::{
            BUILT_IN_READER_GROUP, BUILT_IN_READER_WITH_KEY, BUILT_IN_TOPIC, BUILT_IN_WRITER_GROUP,
            BUILT_IN_WRITER_WITH_KEY, CacheChange, EntityId, Guid, GuidPrefix, ReliabilityKind,
        },
    },
};
use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};
use core::{future::Future, marker::PhantomData, pin::Pin, task::Poll};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x02], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x03], BUILT_IN_READER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId =
    EntityId::new([0, 0, 0x04], BUILT_IN_READER_WITH_KEY);

pub struct DcpsParticipantFactory<R: DdsRuntime, T> {
    domain_participant_list: Vec<(InstanceHandle, MpscSender<DcpsDomainParticipantMail>)>,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
    configuration: DustDdsConfiguration,
    transport: T,
    entity_counter: u32,
    app_id: [u8; 4],
    host_id: [u8; 4],
    runtime: PhantomData<R>,
}

impl<R: DdsRuntime, T: TransportParticipantFactory> DcpsParticipantFactory<R, T> {
    pub fn new(app_id: [u8; 4], host_id: [u8; 4], transport: T) -> Self {
        Self {
            domain_participant_list: Default::default(),
            qos: Default::default(),
            default_participant_qos: Default::default(),
            configuration: Default::default(),
            transport,
            entity_counter: 0,
            app_id,
            host_id,
            runtime: PhantomData,
        }
    }

    fn get_unique_participant_id(&mut self) -> u32 {
        let id = self.entity_counter;
        self.entity_counter += 1;
        id
    }

    fn create_new_guid_prefix(&mut self) -> GuidPrefix {
        let instance_id = self.get_unique_participant_id().to_ne_bytes();

        [
            self.host_id[0],
            self.host_id[1],
            self.host_id[2],
            self.host_id[3], // Host ID
            self.app_id[0],
            self.app_id[1],
            self.app_id[2],
            self.app_id[3], // App ID
            instance_id[0],
            instance_id[1],
            instance_id[2],
            instance_id[3], // Instance ID
        ]
    }

    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    pub async fn create_participant(
        &mut self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        status_kind: Vec<StatusKind>,
        clock_handle: R::ClockHandle,
        mut timer_handle: R::TimerHandle,
        spawner_handle: R::SpawnerHandle,
    ) -> DdsResult<(
        MpscSender<DcpsDomainParticipantMail>,
        InstanceHandle,
        ActorAddress<DcpsStatusCondition>,
    )> {
        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let guid_prefix = self.create_new_guid_prefix();
        let (participant_sender, participant_receiver) = mpsc_channel();

        let (data_channel_sender, data_channel_receiver) = mpsc_channel();

        let mut transport = self
            .transport
            .create_participant(guid_prefix, domain_id, data_channel_sender)
            .await;
        let participant_instance_handle = InstanceHandle::new(transport.guid().into());

        fn sedp_data_reader_qos() -> DataReaderQos {
            DataReaderQos {
                durability: DurabilityQosPolicy {
                    kind: DurabilityQosPolicyKind::TransientLocal,
                },
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepLast(1),
                },
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::Reliable,
                    max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
                },
                ..Default::default()
            }
        }

        fn sedp_data_writer_qos() -> DataWriterQos {
            DataWriterQos {
                durability: DurabilityQosPolicy {
                    kind: DurabilityQosPolicyKind::TransientLocal,
                },
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepLast(1),
                },
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::Reliable,
                    max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
                },
                representation: DataRepresentationQosPolicy {
                    value: vec![BUILT_IN_DATA_REPRESENTATION],
                },
                ..Default::default()
            }
        }

        let mut topic_list = Vec::new();

        let spdp_topic_participant_handle = [
            participant_instance_handle[0],
            participant_instance_handle[1],
            participant_instance_handle[2],
            participant_instance_handle[3],
            participant_instance_handle[4],
            participant_instance_handle[5],
            participant_instance_handle[6],
            participant_instance_handle[7],
            participant_instance_handle[8],
            participant_instance_handle[9],
            participant_instance_handle[10],
            participant_instance_handle[11],
            0,
            0,
            0,
            BUILT_IN_TOPIC,
        ];

        let spdp_topic_participant = TopicEntity::new(
            TopicQos::default(),
            "SpdpDiscoveredParticipantData".to_string(),
            String::from(DCPS_PARTICIPANT),
            InstanceHandle::new(spdp_topic_participant_handle),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
        );

        topic_list.push(TopicDescriptionKind::Topic(spdp_topic_participant));

        let sedp_topic_topics_handle = [
            participant_instance_handle[0],
            participant_instance_handle[1],
            participant_instance_handle[2],
            participant_instance_handle[3],
            participant_instance_handle[4],
            participant_instance_handle[5],
            participant_instance_handle[6],
            participant_instance_handle[7],
            participant_instance_handle[8],
            participant_instance_handle[9],
            participant_instance_handle[10],
            participant_instance_handle[11],
            0,
            0,
            1,
            BUILT_IN_TOPIC,
        ];
        let sedp_topic_topics = TopicEntity::new(
            TopicQos::default(),
            "DiscoveredTopicData".to_string(),
            String::from(DCPS_TOPIC),
            InstanceHandle::new(sedp_topic_topics_handle),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            Arc::new(DiscoveredTopicData::get_type()),
        );

        topic_list.push(TopicDescriptionKind::Topic(sedp_topic_topics));

        let sedp_topic_publications_handle = [
            participant_instance_handle[0],
            participant_instance_handle[1],
            participant_instance_handle[2],
            participant_instance_handle[3],
            participant_instance_handle[4],
            participant_instance_handle[5],
            participant_instance_handle[6],
            participant_instance_handle[7],
            participant_instance_handle[8],
            participant_instance_handle[9],
            participant_instance_handle[10],
            participant_instance_handle[11],
            0,
            0,
            2,
            BUILT_IN_TOPIC,
        ];
        let sedp_topic_publications = TopicEntity::new(
            TopicQos::default(),
            "DiscoveredWriterData".to_string(),
            String::from(DCPS_PUBLICATION),
            InstanceHandle::new(sedp_topic_publications_handle),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            Arc::new(DiscoveredWriterData::get_type()),
        );
        topic_list.push(TopicDescriptionKind::Topic(sedp_topic_publications));

        let sedp_topic_subscriptions_handle = [
            participant_instance_handle[0],
            participant_instance_handle[1],
            participant_instance_handle[2],
            participant_instance_handle[3],
            participant_instance_handle[4],
            participant_instance_handle[5],
            participant_instance_handle[6],
            participant_instance_handle[7],
            participant_instance_handle[8],
            participant_instance_handle[9],
            participant_instance_handle[10],
            participant_instance_handle[11],
            0,
            0,
            3,
            BUILT_IN_TOPIC,
        ];
        let sedp_topic_subscriptions = TopicEntity::new(
            TopicQos::default(),
            "DiscoveredReaderData".to_string(),
            String::from(DCPS_SUBSCRIPTION),
            InstanceHandle::new(sedp_topic_subscriptions_handle),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            Arc::new(DiscoveredReaderData::get_type()),
        );
        topic_list.push(TopicDescriptionKind::Topic(sedp_topic_subscriptions));

        let spdp_writer_qos = DataWriterQos {
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(1),
            },
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
            },
            representation: DataRepresentationQosPolicy {
                value: vec![BUILT_IN_DATA_REPRESENTATION],
            },
            ..Default::default()
        };
        let spdp_reader_qos = DataReaderQos {
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(1),
            },
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
            },
            ..Default::default()
        };

        let rtps_stateless_reader = RtpsStatelessReader::new(
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER),
            Box::new(DcpsParticipantReaderHistoryCache {
                participant_address: participant_sender.clone(),
            }),
        );

        let dcps_participant_reader = DataReaderEntity::new(
            InstanceHandle::new(rtps_stateless_reader.guid().into()),
            spdp_reader_qos,
            String::from(DCPS_PARTICIPANT),
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            Vec::new(),
            TransportReaderKind::Stateless(rtps_stateless_reader),
        );

        let dcps_topic_transport_reader = RtpsStatefulReader::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR),
            Box::new(DcpsTopicsReaderHistoryCache {
                participant_address: participant_sender.clone(),
            }),
            ReliabilityKind::Reliable,
        );

        let dcps_topic_reader = DataReaderEntity::new(
            InstanceHandle::new(dcps_topic_transport_reader.guid().into()),
            sedp_data_reader_qos(),
            String::from(DCPS_TOPIC),
            Arc::new(DiscoveredTopicData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            Vec::new(),
            TransportReaderKind::Stateful(dcps_topic_transport_reader),
        );

        let dcps_publication_transport_reader = RtpsStatefulReader::new(
            Guid::new(
                transport.guid().prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ),
            Box::new(DcpsPublicationsReaderHistoryCache {
                participant_address: participant_sender.clone(),
            }),
            ReliabilityKind::Reliable,
        );

        let dcps_publication_reader = DataReaderEntity::new(
            InstanceHandle::new(dcps_publication_transport_reader.guid().into()),
            sedp_data_reader_qos(),
            String::from(DCPS_PUBLICATION),
            Arc::new(DiscoveredWriterData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            Vec::new(),
            TransportReaderKind::Stateful(dcps_publication_transport_reader),
        );

        let dcps_subscription_transport_reader = RtpsStatefulReader::new(
            Guid::new(
                transport.guid().prefix(),
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            ),
            Box::new(DcpsSubscriptionsReaderHistoryCache {
                participant_address: participant_sender.clone(),
            }),
            ReliabilityKind::Reliable,
        );

        let dcps_subscription_reader = DataReaderEntity::new(
            InstanceHandle::new(dcps_subscription_transport_reader.guid().into()),
            sedp_data_reader_qos(),
            String::from(DCPS_SUBSCRIPTION),
            Arc::new(DiscoveredReaderData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            Vec::new(),
            TransportReaderKind::Stateful(dcps_subscription_transport_reader),
        );

        let data_reader_list = vec![
            dcps_participant_reader,
            dcps_topic_reader,
            dcps_publication_reader,
            dcps_subscription_reader,
        ];
        let builtin_subscriber_handle = [
            participant_instance_handle[0],
            participant_instance_handle[1],
            participant_instance_handle[2],
            participant_instance_handle[3],
            participant_instance_handle[4],
            participant_instance_handle[5],
            participant_instance_handle[6],
            participant_instance_handle[7],
            participant_instance_handle[8],
            participant_instance_handle[9],
            participant_instance_handle[10],
            participant_instance_handle[11],
            0,
            0,
            0,
            BUILT_IN_READER_GROUP,
        ];
        let builtin_subscriber = SubscriberEntity::new(
            InstanceHandle::new(builtin_subscriber_handle),
            SubscriberQos::default(),
            data_reader_list,
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
        );

        let mut dcps_participant_transport_writer = RtpsStatelessWriter::new(
            Guid::new(
                transport.guid.prefix(),
                ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
            ),
        );
        for &discovery_locator in transport.metatraffic_multicast_locator_list() {
            dcps_participant_transport_writer.reader_locator_add(discovery_locator);
        }
        let dcps_participant_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_participant_transport_writer.guid().into()),
            TransportWriterKind::Stateless(dcps_participant_transport_writer),
            String::from(DCPS_PARTICIPANT),
            "SpdpDiscoveredParticipantData".to_string(),
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            spdp_writer_qos,
        );

        let dcps_topics_transport_writer = RtpsStatefulWriter::new(
            Guid::new(
                transport.guid.prefix(),
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ),
            transport.fragment_size,
        );

        let dcps_topics_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_topics_transport_writer.guid().into()),
            TransportWriterKind::Stateful(dcps_topics_transport_writer),
            String::from(DCPS_TOPIC),
            "DiscoveredTopicData".to_string(),
            Arc::new(DiscoveredTopicData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        let dcps_publications_transport_writer = RtpsStatefulWriter::new(
            Guid::new(
                transport.guid().prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ),
            transport.fragment_size,
        );

        let dcps_publications_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_publications_transport_writer.guid().into()),
            TransportWriterKind::Stateful(dcps_publications_transport_writer),
            String::from(DCPS_PUBLICATION),
            "DiscoveredWriterData".to_string(),
            Arc::new(DiscoveredWriterData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            sedp_data_writer_qos(),
        );

        let dcps_subscriptions_transport_writer = RtpsStatefulWriter::new(
            Guid::new(
                transport.guid().prefix(),
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ),
            transport.fragment_size,
        );
        let dcps_subscriptions_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_subscriptions_transport_writer.guid().into()),
            TransportWriterKind::Stateful(dcps_subscriptions_transport_writer),
            String::from(DCPS_SUBSCRIPTION),
            "DiscoveredReaderData".to_string(),
            Arc::new(DiscoveredReaderData::get_type()),
            Actor::spawn::<R>(DcpsStatusCondition::default(), &spawner_handle),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        let builtin_data_writer_list = vec![
            dcps_participant_writer,
            dcps_topics_writer,
            dcps_publications_writer,
            dcps_subscriptions_writer,
        ];
        let builtin_publisher_handle = [
            participant_instance_handle[0],
            participant_instance_handle[1],
            participant_instance_handle[2],
            participant_instance_handle[3],
            participant_instance_handle[4],
            participant_instance_handle[5],
            participant_instance_handle[6],
            participant_instance_handle[7],
            participant_instance_handle[8],
            participant_instance_handle[9],
            participant_instance_handle[10],
            participant_instance_handle[11],
            0,
            0,
            0,
            BUILT_IN_WRITER_GROUP,
        ];
        let builtin_publisher = PublisherEntity::new(
            PublisherQos::default(),
            InstanceHandle::new(builtin_publisher_handle),
            builtin_data_writer_list,
            None,
            vec![],
        );
        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&spawner_handle));
        let domain_participant = DomainParticipantEntity::new(
            domain_id,
            domain_participant_qos,
            listener_sender,
            status_kind,
            participant_instance_handle,
            builtin_publisher,
            builtin_subscriber,
            topic_list,
            String::from(self.configuration.domain_tag()),
        );

        let mut dcps_participant: DcpsDomainParticipant<R> = DcpsDomainParticipant::new(
            domain_participant,
            transport,
            clock_handle,
            timer_handle.clone(),
            spawner_handle.clone(),
        );
        let builtin_subscriber_status_condition_address = dcps_participant
            .get_builtin_subscriber()
            .status_condition()
            .address();

        enum Either {
            A(Option<DcpsDomainParticipantMail>),
            B(Option<Arc<[u8]>>),
        }
        struct Select<A, B> {
            a: A,
            b: B,
        }
        impl<'a, A, B> Future for Select<A, B>
        where
            A: Future<Output = Option<DcpsDomainParticipantMail>> + Unpin,
            B: Future<Output = Option<Arc<[u8]>>> + Unpin,
        {
            type Output = Either;

            fn poll(
                mut self: Pin<&mut Self>,
                cx: &mut core::task::Context<'_>,
            ) -> core::task::Poll<Self::Output> {
                if let Poll::Ready(a) = Pin::new(&mut self.a).poll(cx) {
                    return Poll::Ready(Either::A(a));
                }
                if let Poll::Ready(b) = Pin::new(&mut self.b).poll(cx) {
                    return Poll::Ready(Either::B(b));
                }
                Poll::Pending
            }
        }

        spawner_handle.spawn(async move {
            loop {
                let select = Select {
                    a: core::pin::pin!(participant_receiver.receive()),
                    b: core::pin::pin!(data_channel_receiver.receive()),
                };
                match select.await {
                    Either::A(dcps_domain_participant_mail) => {
                        if let Some(dcps_domain_participant_mail) = dcps_domain_participant_mail {
                            dcps_participant.handle(dcps_domain_participant_mail).await
                        }
                    }
                    Either::B(data_message) => {
                        if let Some(data_message) = data_message {
                            dcps_participant.handle_data(data_message).await
                        }
                    }
                }
            }
        });

        //****** Spawn the participant actor and tasks **********//

        // Start the regular participant announcement task
        let participant_address = participant_sender.clone();
        let participant_announcement_interval =
            self.configuration.participant_announcement_interval();

        spawner_handle.spawn(async move {
            while participant_address
                .send(DcpsDomainParticipantMail::Discovery(
                    DiscoveryServiceMail::AnnounceParticipant,
                ))
                .await
                .is_ok()
            {
                timer_handle.delay(participant_announcement_interval).await;
            }
        });

        if self.qos.entity_factory.autoenable_created_entities {
            let (reply_sender, _reply_receiver) = oneshot();
            participant_sender
                .send(DcpsDomainParticipantMail::Participant(
                    ParticipantServiceMail::Enable { reply_sender },
                ))
                .await
                .ok();
        }

        let participant_address = participant_sender.clone();
        self.domain_participant_list
            .push((participant_instance_handle, participant_sender));

        Ok((
            participant_address,
            participant_instance_handle,
            builtin_subscriber_status_condition_address,
        ))
    }

    pub fn delete_participant(
        &mut self,
        handle: InstanceHandle,
    ) -> DdsResult<MpscSender<DcpsDomainParticipantMail>> {
        let index = self
            .domain_participant_list
            .iter()
            .position(|(h, _)| h == &handle)
            .ok_or(DdsError::PreconditionNotMet(
                "Participant can only be deleted from its parent domain participant factory"
                    .to_string(),
            ))?;

        let (_, participant) = self.domain_participant_list.remove(index);
        Ok(participant)
    }

    pub fn set_default_participant_qos(
        &mut self,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_participant_qos = qos;

        Ok(())
    }

    pub fn get_default_participant_qos(&mut self) -> DomainParticipantQos {
        self.default_participant_qos.clone()
    }

    pub fn set_qos(&mut self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;
        Ok(())
    }

    pub fn get_qos(&mut self) -> DomainParticipantFactoryQos {
        self.qos.clone()
    }

    pub fn set_configuration(&mut self, configuration: DustDdsConfiguration) {
        self.configuration = configuration;
    }

    pub fn get_configuration(&mut self) -> DustDdsConfiguration {
        self.configuration.clone()
    }
}

struct DcpsParticipantReaderHistoryCache {
    participant_address: MpscSender<DcpsDomainParticipantMail>,
}

impl HistoryCache for DcpsParticipantReaderHistoryCache {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DcpsDomainParticipantMail::Message(
                    MessageServiceMail::AddBuiltinParticipantsDetectorCacheChange { cache_change },
                ))
                .await
                .ok();
        })
    }

    fn remove_change(&mut self, _sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }
}

struct DcpsTopicsReaderHistoryCache {
    pub participant_address: MpscSender<DcpsDomainParticipantMail>,
}

impl HistoryCache for DcpsTopicsReaderHistoryCache {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DcpsDomainParticipantMail::Message(
                    MessageServiceMail::AddBuiltinTopicsDetectorCacheChange { cache_change },
                ))
                .await
                .ok();
        })
    }

    fn remove_change(&mut self, _sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }
}

struct DcpsSubscriptionsReaderHistoryCache {
    pub participant_address: MpscSender<DcpsDomainParticipantMail>,
}

impl HistoryCache for DcpsSubscriptionsReaderHistoryCache {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DcpsDomainParticipantMail::Message(
                    MessageServiceMail::AddBuiltinSubscriptionsDetectorCacheChange {
                        cache_change,
                        participant_address: self.participant_address.clone(),
                    },
                ))
                .await
                .ok();
        })
    }

    fn remove_change(&mut self, _sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }
}

struct DcpsPublicationsReaderHistoryCache {
    pub participant_address: MpscSender<DcpsDomainParticipantMail>,
}

impl HistoryCache for DcpsPublicationsReaderHistoryCache {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DcpsDomainParticipantMail::Message(
                    MessageServiceMail::AddBuiltinPublicationsDetectorCacheChange {
                        cache_change,
                        participant_address: self.participant_address.clone(),
                    },
                ))
                .await
                .ok();
        })
    }

    fn remove_change(&mut self, _sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }
}
