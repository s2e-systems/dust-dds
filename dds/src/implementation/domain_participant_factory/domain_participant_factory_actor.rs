use crate::{
    builtin_topics::{DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC},
    configuration::DustDdsConfiguration,
    dcps::{
        data_reader::{DataReaderEntity, TransportReaderKind},
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        data_writer::{DataWriterEntity, TransportWriterKind},
        publisher::PublisherEntity,
        subscriber::SubscriberEntity,
        topic::TopicEntity,
    },
    implementation::{
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            domain_participant_actor_mail::{
                DiscoveryServiceMail, DomainParticipantMail, MessageServiceMail,
                ParticipantServiceMail,
            },
            entities::domain_participant::DomainParticipantEntity,
            handle::InstanceHandleCounter,
        },
        listeners::{
            data_reader_listener::DataReaderListenerActor,
            data_writer_listener::DataWriterListenerActor,
            domain_participant_listener::ListenerMail, publisher_listener::PublisherListenerActor,
            subscriber_listener::SubscriberListenerActor, topic_listener::TopicListenerActor,
        },
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        domain::DomainId,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantFactoryQos, DomainParticipantQos,
            PublisherQos, QosKind, SubscriberQos, TopicQos,
        },
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::StatusKind,
        time::{Duration, DurationKind},
        type_support::TypeSupport,
    },
    listener::NoOpListener,
    rtps_udp_transport::udp_transport::RtpsUdpTransportParticipantFactory,
    runtime::{
        actor::{Actor, ActorAddress, ActorBuilder, MailHandler},
        executor::Executor,
        mpsc::MpscSender,
        oneshot::{oneshot, OneshotSender},
        timer::TimerDriver,
    },
    transport::{
        factory::TransportParticipantFactory,
        history_cache::{CacheChange, HistoryCache},
        participant::TransportParticipant,
        reader::{TransportStatefulReader, TransportStatelessReader},
        types::{
            EntityId, GuidPrefix, ReliabilityKind, BUILT_IN_READER_WITH_KEY,
            BUILT_IN_WRITER_WITH_KEY,
        },
        writer::{TransportStatefulWriter, TransportStatelessWriter},
    },
};
use alloc::sync::Arc;

pub type DdsTransportParticipantFactory =
    Box<dyn TransportParticipantFactory<TransportParticipant = DdsTransportParticipant>>;
pub type DdsTransportParticipant = Box<
    dyn TransportParticipant<
        HistoryCache = Box<dyn HistoryCache>,
        StatelessReader = Box<dyn TransportStatelessReader>,
        StatefulReader = Box<dyn TransportStatefulReader>,
        StatelessWriter = Box<dyn TransportStatelessWriter>,
        StatefulWriter = Box<dyn TransportStatefulWriter>,
    >,
>;

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

pub struct DomainParticipantFactoryActor {
    domain_participant_list: Vec<(InstanceHandle, Actor<DomainParticipantActor>)>,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
    configuration: DustDdsConfiguration,
    transport: DdsTransportParticipantFactory,
    entity_counter: u32,
    app_id: [u8; 4],
    host_id: [u8; 4],
}

impl DomainParticipantFactoryActor {
    pub fn new(app_id: [u8; 4], host_id: [u8; 4]) -> Self {
        Self {
            domain_participant_list: Default::default(),
            qos: Default::default(),
            default_participant_qos: Default::default(),
            configuration: Default::default(),
            transport: Box::new(RtpsUdpTransportParticipantFactory::default()),
            entity_counter: 0,
            app_id,
            host_id,
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

    #[allow(clippy::type_complexity)]
    pub fn create_participant(
        &mut self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        listener_sender: MpscSender<ListenerMail>,
        status_kind: Vec<StatusKind>,
        executor: Executor,
        timer_driver: TimerDriver,
    ) -> DdsResult<(
        ActorAddress<DomainParticipantActor>,
        InstanceHandle,
        ActorAddress<StatusConditionActor>,
        ActorAddress<StatusConditionActor>,
    )> {
        let executor_handle = executor.handle();
        let timer_handle = timer_driver.handle();

        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let guid_prefix = self.create_new_guid_prefix();
        let participant_actor_builder = ActorBuilder::new();

        let mut transport = self.transport.create_participant(guid_prefix, domain_id);

        let noop_topic_listener_sender =
            TopicListenerActor::spawn(NoOpListener, &executor.handle());
        let noop_publisher_listener_sender =
            PublisherListenerActor::spawn(NoOpListener, &executor.handle());
        let noop_writer_listener_sender =
            DataWriterListenerActor::spawn::<()>(NoOpListener, &executor.handle());
        let noop_subscriber_listener_sender =
            SubscriberListenerActor::spawn(NoOpListener, &executor.handle());
        let noop_reader_listener_sender =
            DataReaderListenerActor::spawn::<()>(NoOpListener, &executor.handle());

        let mut instance_handle_counter = InstanceHandleCounter::default();
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
                ..Default::default()
            }
        }

        let mut topic_list = Vec::new();
        let spdp_topic_participant_handle = instance_handle_counter.generate_new_instance_handle();

        let mut spdp_topic_participant = TopicEntity::new(
            TopicQos::default(),
            "SpdpDiscoveredParticipantData".to_string(),
            DCPS_PARTICIPANT.to_owned(),
            spdp_topic_participant_handle,
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
            noop_topic_listener_sender.clone(),
            vec![],
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
        );
        spdp_topic_participant.enable();

        topic_list.push(spdp_topic_participant);

        let sedp_topic_topics_handle = instance_handle_counter.generate_new_instance_handle();
        let mut sedp_topic_topics = TopicEntity::new(
            TopicQos::default(),
            "DiscoveredTopicData".to_string(),
            DCPS_TOPIC.to_owned(),
            sedp_topic_topics_handle,
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
            noop_topic_listener_sender.clone(),
            vec![],
            Arc::new(DiscoveredTopicData::get_type()),
        );
        sedp_topic_topics.enable();

        topic_list.push(sedp_topic_topics);

        let sedp_topic_publications_handle = instance_handle_counter.generate_new_instance_handle();
        let mut sedp_topic_publications = TopicEntity::new(
            TopicQos::default(),
            "DiscoveredWriterData".to_string(),
            DCPS_PUBLICATION.to_owned(),
            sedp_topic_publications_handle,
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
            noop_topic_listener_sender.clone(),
            vec![],
            Arc::new(DiscoveredWriterData::get_type()),
        );
        sedp_topic_publications.enable();
        topic_list.push(sedp_topic_publications);

        let sedp_topic_subscriptions_handle =
            instance_handle_counter.generate_new_instance_handle();
        let mut sedp_topic_subscriptions = TopicEntity::new(
            TopicQos::default(),
            "DiscoveredReaderData".to_string(),
            DCPS_SUBSCRIPTION.to_owned(),
            sedp_topic_subscriptions_handle,
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
            noop_topic_listener_sender,
            vec![],
            Arc::new(DiscoveredReaderData::get_type()),
        );
        sedp_topic_subscriptions.enable();
        topic_list.push(sedp_topic_subscriptions);

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

        let dcps_participant_transport_reader = transport.create_stateless_reader(
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
            Box::new(DcpsParticipantReaderHistoryCache {
                participant_address: participant_actor_builder.address(),
            }),
        );
        let mut dcps_participant_reader = DataReaderEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            spdp_reader_qos,
            DCPS_PARTICIPANT.to_owned(),
            "SpdpDiscoveredParticipantData".to_string(),
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
            noop_reader_listener_sender.clone(),
            Vec::new(),
            TransportReaderKind::Stateless(dcps_participant_transport_reader),
        );
        dcps_participant_reader.enable();
        let dcps_topic_transport_reader = transport.create_stateful_reader(
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            ReliabilityKind::Reliable,
            Box::new(DcpsTopicsReaderHistoryCache {
                participant_address: participant_actor_builder.address(),
            }),
        );
        let mut dcps_topic_reader = DataReaderEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            sedp_data_reader_qos(),
            DCPS_TOPIC.to_owned(),
            "DiscoveredTopicData".to_string(),
            Arc::new(DiscoveredTopicData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
            noop_reader_listener_sender.clone(),
            Vec::new(),
            TransportReaderKind::Stateful(dcps_topic_transport_reader),
        );
        dcps_topic_reader.enable();
        let dcps_publication_transport_reader = transport.create_stateful_reader(
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ReliabilityKind::Reliable,
            Box::new(DcpsPublicationsReaderHistoryCache {
                participant_address: participant_actor_builder.address(),
            }),
        );
        let mut dcps_publication_reader = DataReaderEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            sedp_data_reader_qos(),
            DCPS_PUBLICATION.to_owned(),
            "DiscoveredWriterData".to_string(),
            Arc::new(DiscoveredWriterData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
            noop_reader_listener_sender.clone(),
            Vec::new(),
            TransportReaderKind::Stateful(dcps_publication_transport_reader),
        );
        dcps_publication_reader.enable();
        let dcps_subscription_transport_reader = transport.create_stateful_reader(
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            ReliabilityKind::Reliable,
            Box::new(DcpsSubscriptionsReaderHistoryCache {
                participant_address: participant_actor_builder.address(),
            }),
        );
        let mut dcps_subscription_reader = DataReaderEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            sedp_data_reader_qos(),
            DCPS_SUBSCRIPTION.to_owned(),
            "DiscoveredReaderData".to_string(),
            Arc::new(DiscoveredReaderData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
            noop_reader_listener_sender,
            Vec::new(),
            TransportReaderKind::Stateful(dcps_subscription_transport_reader),
        );
        dcps_subscription_reader.enable();

        let mut builtin_subscriber = SubscriberEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            SubscriberQos::default(),
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
            noop_subscriber_listener_sender,
            vec![],
        );
        builtin_subscriber.enable();
        builtin_subscriber.insert_data_reader(dcps_participant_reader);
        builtin_subscriber.insert_data_reader(dcps_topic_reader);
        builtin_subscriber.insert_data_reader(dcps_publication_reader);
        builtin_subscriber.insert_data_reader(dcps_subscription_reader);

        let mut dcps_participant_transport_writer =
            transport.create_stateless_writer(ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER);
        for &discovery_locator in transport.metatraffic_multicast_locator_list() {
            dcps_participant_transport_writer.add_reader_locator(discovery_locator);
        }
        let mut dcps_participant_writer = DataWriterEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            TransportWriterKind::Stateless(dcps_participant_transport_writer),
            DCPS_PARTICIPANT.to_owned(),
            "SpdpDiscoveredParticipantData".to_string(),
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
            noop_writer_listener_sender.clone(),
            vec![],
            spdp_writer_qos,
        );
        dcps_participant_writer.enable();

        let dcps_topics_transport_writer = transport.create_stateful_writer(
            ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ReliabilityKind::Reliable,
        );
        let mut dcps_topics_writer = DataWriterEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            TransportWriterKind::Stateful(dcps_topics_transport_writer),
            DCPS_TOPIC.to_owned(),
            "DiscoveredTopicData".to_string(),
            Arc::new(DiscoveredTopicData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
            noop_writer_listener_sender.clone(),
            vec![],
            sedp_data_writer_qos(),
        );
        dcps_topics_writer.enable();
        let dcps_publications_transport_writer = transport.create_stateful_writer(
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ReliabilityKind::Reliable,
        );
        let mut dcps_publications_writer = DataWriterEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            TransportWriterKind::Stateful(dcps_publications_transport_writer),
            DCPS_PUBLICATION.to_owned(),
            "DiscoveredWriterData".to_string(),
            Arc::new(DiscoveredWriterData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
            noop_writer_listener_sender.clone(),
            vec![],
            sedp_data_writer_qos(),
        );
        dcps_publications_writer.enable();

        let dcps_subscriptions_transport_writer = transport.create_stateful_writer(
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ReliabilityKind::Reliable,
        );
        let mut dcps_subscriptions_writer = DataWriterEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            TransportWriterKind::Stateful(dcps_subscriptions_transport_writer),
            DCPS_SUBSCRIPTION.to_owned(),
            "DiscoveredReaderData".to_string(),
            Arc::new(DiscoveredReaderData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
            noop_writer_listener_sender,
            vec![],
            sedp_data_writer_qos(),
        );
        dcps_subscriptions_writer.enable();
        let mut builtin_publisher = PublisherEntity::new(
            PublisherQos::default(),
            instance_handle_counter.generate_new_instance_handle(),
            noop_publisher_listener_sender,
            vec![],
            Actor::spawn(StatusConditionActor::default(), &executor_handle),
        );
        builtin_publisher.enable();
        builtin_publisher.insert_data_writer(dcps_participant_writer);
        builtin_publisher.insert_data_writer(dcps_topics_writer);
        builtin_publisher.insert_data_writer(dcps_publications_writer);
        builtin_publisher.insert_data_writer(dcps_subscriptions_writer);
        let instance_handle = InstanceHandle::new(transport.guid().into());

        let status_condition = Actor::spawn(StatusConditionActor::default(), &executor_handle);

        let domain_participant = DomainParticipantEntity::new(
            domain_id,
            domain_participant_qos,
            listener_sender,
            status_kind,
            status_condition,
            instance_handle,
            builtin_publisher,
            builtin_subscriber,
            topic_list,
            self.configuration.domain_tag().to_owned(),
        );

        let domain_participant_actor = DomainParticipantActor::new(
            domain_participant,
            transport,
            executor,
            timer_driver,
            instance_handle_counter,
        );
        let participant_handle = domain_participant_actor
            .domain_participant
            .instance_handle();

        let participant_status_condition_address = domain_participant_actor
            .domain_participant
            .status_condition()
            .address();
        let builtin_subscriber_status_condition_address = domain_participant_actor
            .domain_participant
            .builtin_subscriber()
            .status_condition()
            .address();

        let participant_actor =
            participant_actor_builder.build(domain_participant_actor, &executor_handle);

        //****** Spawn the participant actor and tasks **********//

        // Start the regular participant announcement task
        let participant_address = participant_actor.address();
        let participant_announcement_interval =
            self.configuration.participant_announcement_interval();

        executor_handle.spawn(async move {
            while participant_address
                .send_actor_mail(DomainParticipantMail::Discovery(
                    DiscoveryServiceMail::AnnounceParticipant,
                ))
                .is_ok()
            {
                timer_handle.sleep(participant_announcement_interval).await;
            }
        });

        if self.qos.entity_factory.autoenable_created_entities {
            let (reply_sender, _reply_receiver) = oneshot();
            participant_actor.send_actor_mail(DomainParticipantMail::Participant(
                ParticipantServiceMail::Enable { reply_sender },
            ));
        }

        let participant_address = participant_actor.address();
        self.domain_participant_list
            .push((participant_handle, participant_actor));

        Ok((
            participant_address,
            participant_handle,
            participant_status_condition_address,
            builtin_subscriber_status_condition_address,
        ))
    }

    pub fn delete_participant(
        &mut self,
        handle: InstanceHandle,
    ) -> DdsResult<Actor<DomainParticipantActor>> {
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

    pub fn set_transport(&mut self, transport: DdsTransportParticipantFactory) {
        self.transport = transport;
    }
}

pub enum DomainParticipantFactoryMail {
    CreateParticipant {
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        listener_sender: MpscSender<ListenerMail>,
        status_kind: Vec<StatusKind>,
        executor: Executor,
        timer_driver: TimerDriver,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<
            DdsResult<(
                ActorAddress<DomainParticipantActor>,
                InstanceHandle,
                ActorAddress<StatusConditionActor>,
                ActorAddress<StatusConditionActor>,
            )>,
        >,
    },
    DeleteParticipant {
        handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<Actor<DomainParticipantActor>>>,
    },
    SetDefaultParticipantQos {
        qos: QosKind<DomainParticipantQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDefaultParticipantQos {
        reply_sender: OneshotSender<DomainParticipantQos>,
    },
    SetQos {
        qos: QosKind<DomainParticipantFactoryQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetQos {
        reply_sender: OneshotSender<DomainParticipantFactoryQos>,
    },
    SetConfiguration {
        configuration: DustDdsConfiguration,
    },
    GetConfiguration {
        reply_sender: OneshotSender<DustDdsConfiguration>,
    },
    SetTransport {
        transport: DdsTransportParticipantFactory,
    },
}

impl MailHandler for DomainParticipantFactoryActor {
    type Mail = DomainParticipantFactoryMail;

    async fn handle(&mut self, message: Self::Mail) {
        match message {
            DomainParticipantFactoryMail::CreateParticipant {
                domain_id,
                qos,
                listener_sender,
                status_kind,
                executor,
                timer_driver,
                reply_sender,
            } => reply_sender.send(self.create_participant(
                domain_id,
                qos,
                listener_sender,
                status_kind,
                executor,
                timer_driver,
            )),
            DomainParticipantFactoryMail::DeleteParticipant {
                handle,
                reply_sender,
            } => reply_sender.send(self.delete_participant(handle)),
            DomainParticipantFactoryMail::SetDefaultParticipantQos { qos, reply_sender } => {
                reply_sender.send(self.set_default_participant_qos(qos))
            }
            DomainParticipantFactoryMail::GetDefaultParticipantQos { reply_sender } => {
                reply_sender.send(self.get_default_participant_qos())
            }
            DomainParticipantFactoryMail::SetQos { qos, reply_sender } => {
                reply_sender.send(self.set_qos(qos))
            }
            DomainParticipantFactoryMail::GetQos { reply_sender } => {
                reply_sender.send(self.get_qos())
            }
            DomainParticipantFactoryMail::SetConfiguration { configuration } => {
                self.set_configuration(configuration)
            }
            DomainParticipantFactoryMail::GetConfiguration { reply_sender } => {
                reply_sender.send(self.get_configuration())
            }
            DomainParticipantFactoryMail::SetTransport { transport } => {
                self.set_transport(transport)
            }
        }
    }
}

struct DcpsParticipantReaderHistoryCache {
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl HistoryCache for DcpsParticipantReaderHistoryCache {
    fn add_change(&mut self, cache_change: CacheChange) {
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Message(
                MessageServiceMail::AddBuiltinParticipantsDetectorCacheChange { cache_change },
            ))
            .ok();
    }

    fn remove_change(&mut self, _sequence_number: i64) {
        todo!()
    }
}

struct DcpsTopicsReaderHistoryCache {
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl HistoryCache for DcpsTopicsReaderHistoryCache {
    fn add_change(&mut self, cache_change: CacheChange) {
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Message(
                MessageServiceMail::AddBuiltinTopicsDetectorCacheChange { cache_change },
            ))
            .ok();
    }

    fn remove_change(&mut self, _sequence_number: i64) {
        todo!()
    }
}

struct DcpsSubscriptionsReaderHistoryCache {
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl HistoryCache for DcpsSubscriptionsReaderHistoryCache {
    fn add_change(&mut self, cache_change: CacheChange) {
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Message(
                MessageServiceMail::AddBuiltinSubscriptionsDetectorCacheChange {
                    cache_change,
                    participant_address: self.participant_address.clone(),
                },
            ))
            .ok();
    }

    fn remove_change(&mut self, _sequence_number: i64) {
        todo!()
    }
}

struct DcpsPublicationsReaderHistoryCache {
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl HistoryCache for DcpsPublicationsReaderHistoryCache {
    fn add_change(&mut self, cache_change: CacheChange) {
        self.participant_address
            .send_actor_mail(DomainParticipantMail::Message(
                MessageServiceMail::AddBuiltinPublicationsDetectorCacheChange {
                    cache_change,
                    participant_address: self.participant_address.clone(),
                },
            ))
            .ok();
    }

    fn remove_change(&mut self, _sequence_number: i64) {
        todo!()
    }
}
