use crate::{
    builtin_topics::{DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC},
    configuration::DustDdsConfiguration,
    dds_async::domain_participant_listener::DomainParticipantListenerAsync,
    domain::domain_participant_factory::DomainId,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            domain_participant_actor_mail::{
                DiscoveryServiceMail, DomainParticipantMail, MessageServiceMail,
                ParticipantServiceMail,
            },
            entities::{
                data_reader::{DataReaderEntity, TransportReaderKind},
                data_writer::{DataWriterEntity, TransportWriterKind},
                domain_participant::DomainParticipantEntity,
                publisher::PublisherEntity,
                subscriber::SubscriberEntity,
                topic::TopicEntity,
            },
            handle::InstanceHandleCounter,
        },
        listeners::domain_participant_listener::DomainParticipantListenerActor,
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
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
    },
    rtps_udp_transport::udp_transport::RtpsUdpTransportParticipantFactory,
    runtime::{
        actor::{Actor, ActorAddress, ActorBuilder, MailHandler},
        executor::Executor,
        oneshot::{oneshot, OneshotSender},
        timer::TimerDriver,
    },
    topic_definition::type_support::TypeSupport,
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
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, OnceLock,
    },
};
use tracing::warn;

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
    domain_participant_list: HashMap<InstanceHandle, Actor<DomainParticipantActor>>,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
    configuration: DustDdsConfiguration,
    transport: DdsTransportParticipantFactory,
}

impl Default for DomainParticipantFactoryActor {
    fn default() -> Self {
        Self {
            domain_participant_list: Default::default(),
            qos: Default::default(),
            default_participant_qos: Default::default(),
            configuration: Default::default(),
            transport: Box::new(RtpsUdpTransportParticipantFactory::default()),
        }
    }
}

impl DomainParticipantFactoryActor {
    pub fn new() -> Self {
        Default::default()
    }

    fn get_unique_participant_id(&mut self) -> u32 {
        static COUNTER: OnceLock<AtomicU32> = OnceLock::new();
        let c = COUNTER.get_or_init(|| AtomicU32::new(0));
        c.fetch_add(1, Ordering::Acquire)
    }

    fn create_new_guid_prefix(&mut self) -> GuidPrefix {
        let interface_address = NetworkInterface::show()
            .expect("Could not scan interfaces")
            .into_iter()
            .flat_map(|i| {
                i.addr
                    .into_iter()
                    .filter(|a| matches!(a, Addr::V4(v4) if !v4.ip.is_loopback()))
            })
            .next();
        let host_id = if let Some(interface) = interface_address {
            match interface.ip() {
                IpAddr::V4(a) => a.octets(),
                IpAddr::V6(_) => unimplemented!("IPv6 not yet implemented"),
            }
        } else {
            warn!("Failed to get Host ID from IP address, use 0 instead");
            [0; 4]
        };

        let app_id = std::process::id().to_ne_bytes();
        let instance_id = self.get_unique_participant_id().to_ne_bytes();

        [
            host_id[0],
            host_id[1],
            host_id[2],
            host_id[3], // Host ID
            app_id[0],
            app_id[1],
            app_id[2],
            app_id[3], // App ID
            instance_id[0],
            instance_id[1],
            instance_id[2],
            instance_id[3], // Instance ID
        ]
    }
}

pub struct CreateParticipant {
    pub domain_id: DomainId,
    pub qos: QosKind<DomainParticipantQos>,
    pub listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
    pub status_kind: Vec<StatusKind>,
    #[allow(clippy::type_complexity)]
    pub reply_sender: OneshotSender<
        DdsResult<(
            ActorAddress<DomainParticipantActor>,
            InstanceHandle,
            ActorAddress<StatusConditionActor>,
            ActorAddress<StatusConditionActor>,
        )>,
    >,
}
impl MailHandler<CreateParticipant> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: CreateParticipant) {
        let backend_executor = Executor::new();
        let backend_executor_handle = backend_executor.handle();

        let listener_executor = Executor::new();

        let timer_driver = TimerDriver::new();
        let timer_handle = timer_driver.handle();

        let domain_participant_qos = match message.qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let guid_prefix = self.create_new_guid_prefix();
        let participant_actor_builder = ActorBuilder::new();

        let mut transport = self
            .transport
            .create_participant(guid_prefix, message.domain_id);

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
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
            None,
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
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
            None,
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
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
            None,
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
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
            None,
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
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
            None,
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
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
            None,
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
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
            None,
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
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
            None,
            Vec::new(),
            TransportReaderKind::Stateful(dcps_subscription_transport_reader),
        );
        dcps_subscription_reader.enable();

        let mut builtin_subscriber = SubscriberEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            SubscriberQos::default(),
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
            None,
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
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
            None,
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
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
            None,
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
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
            None,
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
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        dcps_subscriptions_writer.enable();
        let mut builtin_publisher = PublisherEntity::new(
            PublisherQos::default(),
            instance_handle_counter.generate_new_instance_handle(),
            None,
            vec![],
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle()),
        );
        builtin_publisher.enable();
        builtin_publisher.insert_data_writer(dcps_participant_writer);
        builtin_publisher.insert_data_writer(dcps_topics_writer);
        builtin_publisher.insert_data_writer(dcps_publications_writer);
        builtin_publisher.insert_data_writer(dcps_subscriptions_writer);
        let instance_handle = InstanceHandle::new(transport.guid().into());

        let status_condition =
            Actor::spawn(StatusConditionActor::default(), &listener_executor.handle());
        let listener = message.listener.map(|l| {
            Actor::spawn(
                DomainParticipantListenerActor::new(l),
                &listener_executor.handle(),
            )
        });
        let domain_participant = DomainParticipantEntity::new(
            message.domain_id,
            domain_participant_qos,
            listener,
            message.status_kind,
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
            backend_executor,
            listener_executor,
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
            participant_actor_builder.build(domain_participant_actor, &backend_executor_handle);

        //****** Spawn the participant actor and tasks **********//

        // Start the regular participant announcement task
        let participant_address = participant_actor.address();
        let participant_announcement_interval =
            self.configuration.participant_announcement_interval();

        backend_executor_handle.spawn(async move {
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
            .insert(participant_handle, participant_actor);

        message.reply_sender.send(Ok((
            participant_address,
            participant_handle,
            participant_status_condition_address,
            builtin_subscriber_status_condition_address,
        )));
    }
}

pub struct DeleteParticipant {
    pub handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<Actor<DomainParticipantActor>>>,
}
impl MailHandler<DeleteParticipant> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: DeleteParticipant) {
        let result = self.domain_participant_list.remove(&message.handle).ok_or(
            DdsError::PreconditionNotMet(
                "Participant can only be deleted from its parent domain participant factory"
                    .to_string(),
            ),
        );
        message.reply_sender.send(result);
    }
}

pub struct GetParticipantList {
    pub reply_sender: OneshotSender<Vec<ActorAddress<DomainParticipantActor>>>,
}

impl MailHandler<GetParticipantList> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: GetParticipantList) {
        let participant_list = self
            .domain_participant_list
            .values()
            .map(|a| a.address())
            .collect();
        message.reply_sender.send(participant_list);
    }
}

pub struct SetDefaultParticipantQos {
    pub qos: QosKind<DomainParticipantQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetDefaultParticipantQos> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: SetDefaultParticipantQos) {
        let qos = match message.qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_participant_qos = qos;

        message.reply_sender.send(Ok(()))
    }
}

pub struct GetDefaultParticipantQos {
    pub reply_sender: OneshotSender<DomainParticipantQos>,
}

impl MailHandler<GetDefaultParticipantQos> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: GetDefaultParticipantQos) {
        let q = self.default_participant_qos.clone();
        message.reply_sender.send(q);
    }
}

pub struct SetQos {
    pub qos: QosKind<DomainParticipantFactoryQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}

impl MailHandler<SetQos> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: SetQos) {
        let qos = match message.qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;
        message.reply_sender.send(Ok(()));
    }
}

pub struct GetQos {
    pub reply_sender: OneshotSender<DomainParticipantFactoryQos>,
}
impl MailHandler<GetQos> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: GetQos) {
        let q = self.qos.clone();
        message.reply_sender.send(q);
    }
}

pub struct SetConfiguration {
    pub configuration: DustDdsConfiguration,
}
impl MailHandler<SetConfiguration> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: SetConfiguration) {
        self.configuration = message.configuration;
    }
}

pub struct GetConfiguration {
    pub reply_sender: OneshotSender<DustDdsConfiguration>,
}
impl MailHandler<GetConfiguration> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: GetConfiguration) {
        let c = self.configuration.clone();
        message.reply_sender.send(c);
    }
}

pub struct SetTransport {
    pub transport: DdsTransportParticipantFactory,
}
impl MailHandler<SetTransport> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: SetTransport) {
        self.transport = message.transport;
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
