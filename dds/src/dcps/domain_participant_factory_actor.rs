use super::actor::MailHandler;
use crate::{
    builtin_topics::{DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC},
    configuration::DustDdsConfiguration,
    dcps::{
        actor::{Actor, ActorAddress},
        data_reader::{DataReaderEntity, TransportReaderKind},
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        data_writer::{DataWriterEntity, TransportWriterKind},
        domain_participant_actor::{DomainParticipantActor, DomainParticipantEntity},
        domain_participant_actor_mail::{
            DiscoveryServiceMail, DomainParticipantMail, MessageServiceMail, ParticipantServiceMail,
        },
        handle::InstanceHandleCounter,
        listeners::domain_participant_listener::ListenerMail,
        publisher::PublisherEntity,
        status_condition_actor::StatusConditionActor,
        subscriber::SubscriberEntity,
        topic::TopicEntity,
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
    runtime::{ChannelReceive, ChannelSend, DdsRuntime, OneshotSend, Spawner, Timer},
    transport::{
        interface::{
            HistoryCache, TransportParticipant, TransportParticipantFactory,
            TransportStatelessWriter,
        },
        types::{
            CacheChange, EntityId, GuidPrefix, ReliabilityKind, BUILT_IN_READER_WITH_KEY,
            BUILT_IN_WRITER_WITH_KEY,
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
use core::{future::Future, pin::Pin};

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

pub struct DomainParticipantFactoryActor<R: DdsRuntime, T> {
    domain_participant_list: Vec<(InstanceHandle, R::ChannelSender<DomainParticipantMail<R>>)>,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
    configuration: DustDdsConfiguration,
    transport: T,
    entity_counter: u32,
    app_id: [u8; 4],
    host_id: [u8; 4],
}

impl<R: DdsRuntime, T: TransportParticipantFactory> DomainParticipantFactoryActor<R, T> {
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
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        status_kind: Vec<StatusKind>,
        clock_handle: R::ClockHandle,
        mut timer_handle: R::TimerHandle,
        spawner_handle: R::SpawnerHandle,
    ) -> DdsResult<(
        R::ChannelSender<DomainParticipantMail<R>>,
        InstanceHandle,
        ActorAddress<R, StatusConditionActor<R>>,
    )> {
        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let guid_prefix = self.create_new_guid_prefix();
        let (participant_sender, mut participant_receiver) = R::channel();

        let mut transport = self
            .transport
            .create_participant(guid_prefix, domain_id)
            .await;

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
            String::from(DCPS_PARTICIPANT),
            spdp_topic_participant_handle,
            Actor::spawn(StatusConditionActor::default(), &spawner_handle),
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
            String::from(DCPS_TOPIC),
            sedp_topic_topics_handle,
            Actor::spawn(StatusConditionActor::default(), &spawner_handle),
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
            String::from(DCPS_PUBLICATION),
            sedp_topic_publications_handle,
            Actor::spawn(StatusConditionActor::default(), &spawner_handle),
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
            String::from(DCPS_SUBSCRIPTION),
            sedp_topic_subscriptions_handle,
            Actor::spawn(StatusConditionActor::default(), &spawner_handle),
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

        let dcps_participant_transport_reader = transport
            .create_stateless_reader(
                ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
                Box::new(DcpsParticipantReaderHistoryCache::<R> {
                    participant_address: participant_sender.clone(),
                }),
            )
            .await;
        let mut dcps_participant_reader = DataReaderEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            spdp_reader_qos,
            String::from(DCPS_PARTICIPANT),
            "SpdpDiscoveredParticipantData".to_string(),
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &spawner_handle),
            None,
            Vec::new(),
            TransportReaderKind::Stateless(dcps_participant_transport_reader),
        );
        dcps_participant_reader.enable();
        let dcps_topic_transport_reader = transport
            .create_stateful_reader(
                ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
                ReliabilityKind::Reliable,
                Box::new(DcpsTopicsReaderHistoryCache::<R> {
                    participant_address: participant_sender.clone(),
                }),
            )
            .await;
        let mut dcps_topic_reader = DataReaderEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            sedp_data_reader_qos(),
            String::from(DCPS_TOPIC),
            "DiscoveredTopicData".to_string(),
            Arc::new(DiscoveredTopicData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &spawner_handle),
            None,
            Vec::new(),
            TransportReaderKind::Stateful(dcps_topic_transport_reader),
        );
        dcps_topic_reader.enable();
        let dcps_publication_transport_reader = transport
            .create_stateful_reader(
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                ReliabilityKind::Reliable,
                Box::new(DcpsPublicationsReaderHistoryCache::<R> {
                    participant_address: participant_sender.clone(),
                }),
            )
            .await;
        let mut dcps_publication_reader = DataReaderEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            sedp_data_reader_qos(),
            String::from(DCPS_PUBLICATION),
            "DiscoveredWriterData".to_string(),
            Arc::new(DiscoveredWriterData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &spawner_handle),
            None,
            Vec::new(),
            TransportReaderKind::Stateful(dcps_publication_transport_reader),
        );
        dcps_publication_reader.enable();
        let dcps_subscription_transport_reader = transport
            .create_stateful_reader(
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                ReliabilityKind::Reliable,
                Box::new(DcpsSubscriptionsReaderHistoryCache::<R> {
                    participant_address: participant_sender.clone(),
                }),
            )
            .await;
        let mut dcps_subscription_reader = DataReaderEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            sedp_data_reader_qos(),
            String::from(DCPS_SUBSCRIPTION),
            "DiscoveredReaderData".to_string(),
            Arc::new(DiscoveredReaderData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &spawner_handle),
            None,
            Vec::new(),
            TransportReaderKind::Stateful(dcps_subscription_transport_reader),
        );
        dcps_subscription_reader.enable();

        let data_reader_list = vec![
            dcps_participant_reader,
            dcps_topic_reader,
            dcps_publication_reader,
            dcps_subscription_reader,
        ];
        let mut builtin_subscriber = SubscriberEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            SubscriberQos::default(),
            data_reader_list,
            Actor::spawn(StatusConditionActor::default(), &spawner_handle),
            None,
            vec![],
        );
        builtin_subscriber.enable();

        let mut dcps_participant_transport_writer = transport
            .create_stateless_writer(ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER)
            .await;
        for &discovery_locator in transport.metatraffic_multicast_locator_list() {
            dcps_participant_transport_writer.add_reader_locator(discovery_locator);
        }
        let mut dcps_participant_writer = DataWriterEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            TransportWriterKind::Stateless(dcps_participant_transport_writer),
            String::from(DCPS_PARTICIPANT),
            "SpdpDiscoveredParticipantData".to_string(),
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &spawner_handle),
            None,
            vec![],
            spdp_writer_qos,
        );
        dcps_participant_writer.enable();

        let dcps_topics_transport_writer = transport
            .create_stateful_writer(
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
                ReliabilityKind::Reliable,
            )
            .await;
        let mut dcps_topics_writer = DataWriterEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            TransportWriterKind::Stateful(dcps_topics_transport_writer),
            String::from(DCPS_TOPIC),
            "DiscoveredTopicData".to_string(),
            Arc::new(DiscoveredTopicData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &spawner_handle),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        dcps_topics_writer.enable();
        let dcps_publications_transport_writer = transport
            .create_stateful_writer(
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                ReliabilityKind::Reliable,
            )
            .await;
        let mut dcps_publications_writer = DataWriterEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            TransportWriterKind::Stateful(dcps_publications_transport_writer),
            String::from(DCPS_PUBLICATION),
            "DiscoveredWriterData".to_string(),
            Arc::new(DiscoveredWriterData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &spawner_handle),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        dcps_publications_writer.enable();

        let dcps_subscriptions_transport_writer = transport
            .create_stateful_writer(
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                ReliabilityKind::Reliable,
            )
            .await;
        let mut dcps_subscriptions_writer = DataWriterEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            TransportWriterKind::Stateful(dcps_subscriptions_transport_writer),
            String::from(DCPS_SUBSCRIPTION),
            "DiscoveredReaderData".to_string(),
            Arc::new(DiscoveredReaderData::get_type()),
            Actor::spawn(StatusConditionActor::default(), &spawner_handle),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        dcps_subscriptions_writer.enable();
        let builtin_data_writer_list = vec![
            dcps_participant_writer,
            dcps_topics_writer,
            dcps_publications_writer,
            dcps_subscriptions_writer,
        ];
        let mut builtin_publisher = PublisherEntity::new(
            PublisherQos::default(),
            instance_handle_counter.generate_new_instance_handle(),
            builtin_data_writer_list,
            None,
            vec![],
        );
        builtin_publisher.enable();
        let instance_handle = InstanceHandle::new(transport.guid().into());

        let domain_participant = DomainParticipantEntity::new(
            domain_id,
            domain_participant_qos,
            listener_sender,
            status_kind,
            instance_handle,
            builtin_publisher,
            builtin_subscriber,
            topic_list,
            String::from(self.configuration.domain_tag()),
        );

        let mut domain_participant_actor: DomainParticipantActor<R, T> =
            DomainParticipantActor::new(
                domain_participant,
                transport,
                instance_handle_counter,
                clock_handle,
                timer_handle.clone(),
                spawner_handle.clone(),
            );
        let participant_handle = domain_participant_actor
            .domain_participant
            .instance_handle();

        let builtin_subscriber_status_condition_address = domain_participant_actor
            .domain_participant
            .builtin_subscriber()
            .status_condition()
            .address();

        spawner_handle.spawn(async move {
            while let Some(m) = participant_receiver.receive().await {
                domain_participant_actor.handle(m).await;
            }
        });

        //****** Spawn the participant actor and tasks **********//

        // Start the regular participant announcement task
        let participant_address = participant_sender.clone();
        let participant_announcement_interval =
            self.configuration.participant_announcement_interval();

        spawner_handle.spawn(async move {
            while participant_address
                .send(DomainParticipantMail::Discovery(
                    DiscoveryServiceMail::AnnounceParticipant,
                ))
                .await
                .is_ok()
            {
                timer_handle.delay(participant_announcement_interval).await;
            }
        });

        if self.qos.entity_factory.autoenable_created_entities {
            let (reply_sender, _reply_receiver) = R::oneshot();
            participant_sender
                .send(DomainParticipantMail::Participant(
                    ParticipantServiceMail::Enable { reply_sender },
                ))
                .await
                .ok();
        }

        let participant_address = participant_sender.clone();
        self.domain_participant_list
            .push((participant_handle, participant_sender));

        Ok((
            participant_address,
            participant_handle,
            builtin_subscriber_status_condition_address,
        ))
    }

    pub fn delete_participant(
        &mut self,
        handle: InstanceHandle,
    ) -> DdsResult<R::ChannelSender<DomainParticipantMail<R>>> {
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

pub enum DomainParticipantFactoryMail<R: DdsRuntime> {
    CreateParticipant {
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        status_kind: Vec<StatusKind>,
        clock_handle: R::ClockHandle,
        timer_handle: R::TimerHandle,
        spawner_handle: R::SpawnerHandle,
        #[allow(clippy::type_complexity)]
        reply_sender: R::OneshotSender<
            DdsResult<(
                R::ChannelSender<DomainParticipantMail<R>>,
                InstanceHandle,
                ActorAddress<R, StatusConditionActor<R>>,
            )>,
        >,
    },
    DeleteParticipant {
        handle: InstanceHandle,
        reply_sender: R::OneshotSender<DdsResult<R::ChannelSender<DomainParticipantMail<R>>>>,
    },
    SetDefaultParticipantQos {
        qos: QosKind<DomainParticipantQos>,
        reply_sender: R::OneshotSender<DdsResult<()>>,
    },
    GetDefaultParticipantQos {
        reply_sender: R::OneshotSender<DomainParticipantQos>,
    },
    SetQos {
        qos: QosKind<DomainParticipantFactoryQos>,
        reply_sender: R::OneshotSender<DdsResult<()>>,
    },
    GetQos {
        reply_sender: R::OneshotSender<DomainParticipantFactoryQos>,
    },
    SetConfiguration {
        configuration: DustDdsConfiguration,
    },
    GetConfiguration {
        reply_sender: R::OneshotSender<DustDdsConfiguration>,
    },
}

impl<R: DdsRuntime, T: TransportParticipantFactory> MailHandler
    for DomainParticipantFactoryActor<R, T>
{
    type Mail = DomainParticipantFactoryMail<R>;

    async fn handle(&mut self, message: Self::Mail) {
        match message {
            DomainParticipantFactoryMail::CreateParticipant {
                domain_id,
                qos,
                listener_sender,
                status_kind,
                clock_handle,
                timer_handle,
                spawner_handle,
                reply_sender,
            } => reply_sender.send(
                self.create_participant(
                    domain_id,
                    qos,
                    listener_sender,
                    status_kind,
                    clock_handle,
                    timer_handle,
                    spawner_handle,
                )
                .await,
            ),
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
        }
    }
}

struct DcpsParticipantReaderHistoryCache<R: DdsRuntime> {
    participant_address: R::ChannelSender<DomainParticipantMail<R>>,
}

impl<R: DdsRuntime> HistoryCache for DcpsParticipantReaderHistoryCache<R> {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DomainParticipantMail::Message(
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

struct DcpsTopicsReaderHistoryCache<R: DdsRuntime> {
    pub participant_address: R::ChannelSender<DomainParticipantMail<R>>,
}

impl<R: DdsRuntime> HistoryCache for DcpsTopicsReaderHistoryCache<R> {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DomainParticipantMail::Message(
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

struct DcpsSubscriptionsReaderHistoryCache<R: DdsRuntime> {
    pub participant_address: R::ChannelSender<DomainParticipantMail<R>>,
}

impl<R: DdsRuntime> HistoryCache for DcpsSubscriptionsReaderHistoryCache<R> {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DomainParticipantMail::Message(
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

struct DcpsPublicationsReaderHistoryCache<R: DdsRuntime> {
    pub participant_address: R::ChannelSender<DomainParticipantMail<R>>,
}

impl<R: DdsRuntime> HistoryCache for DcpsPublicationsReaderHistoryCache<R> {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.participant_address
                .send(DomainParticipantMail::Message(
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
