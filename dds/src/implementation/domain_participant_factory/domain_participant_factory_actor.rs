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
            entities::{
                data_reader::DataReaderEntity, data_writer::DataWriterEntity,
                domain_participant::DomainParticipantEntity, publisher::PublisherEntity,
                subscriber::SubscriberEntity, topic::TopicEntity,
            },
            handle::InstanceHandleCounter,
            services::{discovery_service, domain_participant_service, message_service},
        },
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
    rtps::{
        reader::{ReaderCacheChange, ReaderHistoryCache},
        transport::{RtpsTransport, Transport},
        types::GuidPrefix,
    },
    runtime::{
        actor::{Actor, ActorAddress, ActorBuilder, Mail, MailHandler},
        executor::Executor,
        timer::TimerDriver,
    },
    topic_definition::type_support::TypeSupport,
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
use tracing::{error, warn};

#[derive(Default)]
pub struct DomainParticipantFactoryActor {
    domain_participant_list: HashMap<InstanceHandle, Actor<DomainParticipantActor>>,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
    configuration: DustDdsConfiguration,
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
            .filter(|x| {
                if let Some(if_name) = self.configuration.interface_name() {
                    &x.name == if_name
                } else {
                    true
                }
            })
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
}
impl Mail for CreateParticipant {
    type Result = DdsResult<(
        ActorAddress<DomainParticipantActor>,
        InstanceHandle,
        ActorAddress<StatusConditionActor>,
        ActorAddress<StatusConditionActor>,
    )>;
}
impl MailHandler<CreateParticipant> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: CreateParticipant) -> <CreateParticipant as Mail>::Result {
        let executor = Executor::new();
        let executor_handle = executor.handle();

        let timer_driver = TimerDriver::new();
        let timer_handle = timer_driver.handle();

        let domain_participant_qos = match message.qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let guid_prefix = self.create_new_guid_prefix();
        let participant_actor_builder = ActorBuilder::new();

        let transport = Box::new(RtpsTransport::new(
            guid_prefix,
            message.domain_id,
            self.configuration.domain_tag().to_string(),
            self.configuration.interface_name(),
            self.configuration.udp_receive_buffer_size(),
            Box::new(DcpsParticipantReaderHistoryCache {
                participant_address: participant_actor_builder.address(),
            }),
            Box::new(DcpsTopicsReaderHistoryCache {
                participant_address: participant_actor_builder.address(),
            }),
            Box::new(DcpsPublicationsReaderHistoryCache {
                participant_address: participant_actor_builder.address(),
            }),
            Box::new(DcpsSubscriptionsReaderHistoryCache {
                participant_address: participant_actor_builder.address(),
            }),
        )?);

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

        let mut topic_list = HashMap::new();
        let spdp_topic_participant_handle = instance_handle_counter.generate_new_instance_handle();

        let mut spdp_topic_participant = TopicEntity::new(
            TopicQos::default(),
            "SpdpDiscoveredParticipantData".to_string(),
            DCPS_PARTICIPANT.to_owned(),
            spdp_topic_participant_handle,
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
        );
        spdp_topic_participant.enable();

        topic_list.insert(DCPS_PARTICIPANT.to_owned(), spdp_topic_participant);

        let sedp_topic_topics_handle = instance_handle_counter.generate_new_instance_handle();
        let mut sedp_topic_topics = TopicEntity::new(
            TopicQos::default(),
            "DiscoveredTopicData".to_string(),
            DCPS_TOPIC.to_owned(),
            sedp_topic_topics_handle,
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            Arc::new(DiscoveredTopicData::get_type()),
        );
        sedp_topic_topics.enable();

        topic_list.insert(DCPS_TOPIC.to_owned(), sedp_topic_topics);

        let sedp_topic_publications_handle = instance_handle_counter.generate_new_instance_handle();
        let mut sedp_topic_publications = TopicEntity::new(
            TopicQos::default(),
            "DiscoveredWriterData".to_string(),
            DCPS_PUBLICATION.to_owned(),
            sedp_topic_publications_handle,
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            Arc::new(DiscoveredWriterData::get_type()),
        );
        sedp_topic_publications.enable();
        topic_list.insert(DCPS_PUBLICATION.to_owned(), sedp_topic_publications);

        let sedp_topic_subscriptions_handle =
            instance_handle_counter.generate_new_instance_handle();
        let mut sedp_topic_subscriptions = TopicEntity::new(
            TopicQos::default(),
            "DiscoveredReaderData".to_string(),
            DCPS_SUBSCRIPTION.to_owned(),
            sedp_topic_subscriptions_handle,
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            Arc::new(DiscoveredReaderData::get_type()),
        );
        sedp_topic_subscriptions.enable();
        topic_list.insert(DCPS_SUBSCRIPTION.to_owned(), sedp_topic_subscriptions);

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

        let mut dcps_participant_reader = DataReaderEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            spdp_reader_qos,
            topic_list[DCPS_PARTICIPANT].topic_name().to_owned(),
            topic_list[DCPS_PARTICIPANT].type_name().to_owned(),
            topic_list[DCPS_PARTICIPANT].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            Vec::new(),
            transport.get_participant_discovery_reader(),
        );
        dcps_participant_reader.enable();
        let mut dcps_topic_reader = DataReaderEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            sedp_data_reader_qos(),
            topic_list[DCPS_TOPIC].topic_name().to_owned(),
            topic_list[DCPS_TOPIC].type_name().to_owned(),
            topic_list[DCPS_TOPIC].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            Vec::new(),
            transport.get_topics_discovery_reader(),
        );
        dcps_topic_reader.enable();
        let mut dcps_publication_reader = DataReaderEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            sedp_data_reader_qos(),
            topic_list[DCPS_PUBLICATION].topic_name().to_owned(),
            topic_list[DCPS_PUBLICATION].type_name().to_owned(),
            topic_list[DCPS_PUBLICATION].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            Vec::new(),
            transport.get_topics_discovery_reader(),
        );
        dcps_publication_reader.enable();
        let mut dcps_subscription_reader = DataReaderEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            sedp_data_reader_qos(),
            topic_list[DCPS_SUBSCRIPTION].topic_name().to_owned(),
            topic_list[DCPS_SUBSCRIPTION].type_name().to_owned(),
            topic_list[DCPS_SUBSCRIPTION].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            Vec::new(),
            transport.get_topics_discovery_reader(),
        );
        dcps_subscription_reader.enable();

        let mut builtin_subscriber = SubscriberEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            SubscriberQos::default(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
        );
        builtin_subscriber.enable();
        builtin_subscriber.insert_data_reader(dcps_participant_reader);
        builtin_subscriber.insert_data_reader(dcps_topic_reader);
        builtin_subscriber.insert_data_reader(dcps_publication_reader);
        builtin_subscriber.insert_data_reader(dcps_subscription_reader);

        let mut dcps_participant_writer = DataWriterEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            transport.get_participant_discovery_writer(),
            topic_list[DCPS_PARTICIPANT].topic_name().to_owned(),
            topic_list[DCPS_PARTICIPANT].type_name().to_owned(),
            topic_list[DCPS_PARTICIPANT].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            spdp_writer_qos,
        );
        dcps_participant_writer.enable();

        let mut dcps_topics_writer = DataWriterEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            transport.get_topics_discovery_writer(),
            topic_list[DCPS_TOPIC].topic_name().to_owned(),
            topic_list[DCPS_TOPIC].type_name().to_owned(),
            topic_list[DCPS_TOPIC].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        dcps_topics_writer.enable();
        let mut dcps_publications_writer = DataWriterEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            transport.get_publications_discovery_writer(),
            topic_list[DCPS_PUBLICATION].topic_name().to_owned(),
            topic_list[DCPS_PUBLICATION].type_name().to_owned(),
            topic_list[DCPS_PUBLICATION].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
            None,
            vec![],
            sedp_data_writer_qos(),
        );
        dcps_publications_writer.enable();

        let mut dcps_subscriptions_writer = DataWriterEntity::new(
            instance_handle_counter.generate_new_instance_handle(),
            transport.get_subscriptions_discovery_writer(),
            topic_list[DCPS_SUBSCRIPTION].topic_name().to_owned(),
            topic_list[DCPS_SUBSCRIPTION].type_name().to_owned(),
            topic_list[DCPS_SUBSCRIPTION].type_support().clone(),
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
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
            Actor::spawn(StatusConditionActor::default(), &executor.handle()),
        );
        builtin_publisher.enable();
        builtin_publisher.insert_data_writer(dcps_participant_writer);
        builtin_publisher.insert_data_writer(dcps_topics_writer);
        builtin_publisher.insert_data_writer(dcps_publications_writer);
        builtin_publisher.insert_data_writer(dcps_subscriptions_writer);
        let instance_handle = InstanceHandle::new(transport.guid().into());

        let status_condition = Actor::spawn(StatusConditionActor::default(), &executor.handle());
        let domain_participant = DomainParticipantEntity::new(
            message.domain_id,
            domain_participant_qos,
            message.listener,
            message.status_kind,
            status_condition,
            instance_handle,
            builtin_publisher,
            builtin_subscriber,
            topic_list,
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
            .get_statuscondition();
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
            loop {
                if let Ok(r) =
                    participant_address.send_actor_mail(discovery_service::AnnounceParticipant)
                {
                    if let Err(announce_result) = r.receive_reply().await {
                        error!("Error announcing participant: {:?}", announce_result);
                    }
                    timer_handle.sleep(participant_announcement_interval).await;
                } else {
                    break;
                }
            }
        });

        if self.qos.entity_factory.autoenable_created_entities {
            participant_actor.send_actor_mail(domain_participant_service::Enable {
                domain_participant_address: participant_actor.address(),
            });
        }

        let participant_address = participant_actor.address();
        self.domain_participant_list
            .insert(participant_handle.into(), participant_actor);

        Ok((
            participant_address,
            participant_handle.into(),
            participant_status_condition_address,
            builtin_subscriber_status_condition_address,
        ))
    }
}

pub struct DeleteParticipant {
    pub handle: InstanceHandle,
}
impl Mail for DeleteParticipant {
    type Result = DdsResult<Actor<DomainParticipantActor>>;
}
impl MailHandler<DeleteParticipant> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: DeleteParticipant) -> <DeleteParticipant as Mail>::Result {
        self.domain_participant_list
            .remove(&message.handle)
            .ok_or(DdsError::PreconditionNotMet(
                "Participant can only be deleted from its parent domain participant factory"
                    .to_string(),
            ))
    }
}

pub struct GetParticipantList;
impl Mail for GetParticipantList {
    type Result = Vec<ActorAddress<DomainParticipantActor>>;
}
impl MailHandler<GetParticipantList> for DomainParticipantFactoryActor {
    fn handle(&mut self, _: GetParticipantList) -> <GetParticipantList as Mail>::Result {
        self.domain_participant_list
            .values()
            .map(|a| a.address())
            .collect()
    }
}

pub struct SetDefaultParticipantQos {
    pub qos: QosKind<DomainParticipantQos>,
}
impl Mail for SetDefaultParticipantQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDefaultParticipantQos> for DomainParticipantFactoryActor {
    fn handle(
        &mut self,
        message: SetDefaultParticipantQos,
    ) -> <SetDefaultParticipantQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_participant_qos = qos;

        Ok(())
    }
}

pub struct GetDefaultParticipantQos;
impl Mail for GetDefaultParticipantQos {
    type Result = DomainParticipantQos;
}
impl MailHandler<GetDefaultParticipantQos> for DomainParticipantFactoryActor {
    fn handle(
        &mut self,
        _: GetDefaultParticipantQos,
    ) -> <GetDefaultParticipantQos as Mail>::Result {
        self.default_participant_qos.clone()
    }
}

pub struct SetQos {
    pub qos: QosKind<DomainParticipantFactoryQos>,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;

        Ok(())
    }
}

pub struct GetQos;
impl Mail for GetQos {
    type Result = DomainParticipantFactoryQos;
}
impl MailHandler<GetQos> for DomainParticipantFactoryActor {
    fn handle(&mut self, _: GetQos) -> <GetQos as Mail>::Result {
        self.qos.clone()
    }
}

pub struct SetConfiguration {
    pub configuration: DustDdsConfiguration,
}
impl Mail for SetConfiguration {
    type Result = ();
}
impl MailHandler<SetConfiguration> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: SetConfiguration) -> <SetConfiguration as Mail>::Result {
        self.configuration = message.configuration;
    }
}

pub struct GetConfiguration;
impl Mail for GetConfiguration {
    type Result = DustDdsConfiguration;
}
impl MailHandler<GetConfiguration> for DomainParticipantFactoryActor {
    fn handle(&mut self, _: GetConfiguration) -> <GetConfiguration as Mail>::Result {
        self.configuration.clone()
    }
}

struct DcpsParticipantReaderHistoryCache {
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for DcpsParticipantReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        self.participant_address
            .send_actor_mail(message_service::AddBuiltinParticipantsDetectorCacheChange {
                cache_change,
            })
            .ok();
    }
}

struct DcpsTopicsReaderHistoryCache {
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for DcpsTopicsReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        self.participant_address
            .send_actor_mail(message_service::AddBuiltinTopicsDetectorCacheChange {
                cache_change,
                participant_address: self.participant_address.clone(),
            })
            .ok();
    }
}

struct DcpsSubscriptionsReaderHistoryCache {
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for DcpsSubscriptionsReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        self.participant_address
            .send_actor_mail(
                message_service::AddBuiltinSubscriptionsDetectorCacheChange {
                    cache_change,
                    participant_address: self.participant_address.clone(),
                },
            )
            .ok();
    }
}

struct DcpsPublicationsReaderHistoryCache {
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for DcpsPublicationsReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        self.participant_address
            .send_actor_mail(message_service::AddBuiltinPublicationsDetectorCacheChange {
                cache_change,
                participant_address: self.participant_address.clone(),
            })
            .ok();
    }
}
