use super::{
    data_reader_actor::DataReaderActor,
    data_writer_actor::DataWriterActor,
    domain_participant_actor,
    message_sender_actor::MessageSenderActor,
    status_condition_actor::StatusConditionActor,
    subscriber_actor::{self, SubscriberActor},
    topic_actor::TopicActor,
};
use crate::{
    builtin_topics::{DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC},
    configuration::DustDdsConfiguration,
    dds_async::{
        domain_participant::DomainParticipantAsync,
        domain_participant_listener::DomainParticipantListenerAsync,
    },
    domain::domain_participant_factory::DomainId,
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        actors::domain_participant_actor::DomainParticipantActor,
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        data_representation_inline_qos::{
            parameter_id_values::PID_STATUS_INFO,
            types::{StatusInfo, STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED},
        },
        runtime::{
            executor::{Executor, ExecutorHandle},
            mpsc::{mpsc_channel, MpscSender},
            timer::{TimerDriver, TimerHandle},
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantFactoryQos, DomainParticipantQos,
            QosKind, SubscriberQos, TopicQos,
        },
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::StatusKind,
        time::{Duration, DurationKind, DURATION_ZERO_NSEC, DURATION_ZERO_SEC},
    },
    rtps::{
        discovery_types::{
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
        },
        group::RtpsGroup,
        participant::RtpsParticipant,
        reader::{ReaderCacheChange, ReaderHistoryCache},
        types::{
            EntityId, Guid, GuidPrefix, BUILT_IN_READER_GROUP, BUILT_IN_TOPIC,
            ENTITYID_PARTICIPANT, PROTOCOLVERSION, VENDOR_ID_S2E,
        },
    },
    topic_definition::type_support::{DdsDeserialize, TypeSupport},
    xtypes::{deserialize::XTypesDeserialize, xcdr_deserializer::Xcdr1LeDeserializer},
};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex, OnceLock,
    },
};
use tracing::warn;

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

    fn create_builtin_topics(
        &self,
        guid_prefix: GuidPrefix,
        handle: &ExecutorHandle,
    ) -> HashMap<String, TopicActor> {
        let mut topic_list = HashMap::new();

        let spdp_topic_entity_id = EntityId::new([0, 0, 0], BUILT_IN_TOPIC);
        let spdp_topic_guid = Guid::new(guid_prefix, spdp_topic_entity_id);
        let (spdp_topic_participant, _) = TopicActor::new(
            spdp_topic_guid,
            TopicQos::default(),
            "SpdpDiscoveredParticipantData".to_string(),
            DCPS_PARTICIPANT,
            None,
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
            handle,
        );
        topic_list.insert(DCPS_PARTICIPANT.to_owned(), spdp_topic_participant);

        let sedp_topics_entity_id = EntityId::new([0, 0, 1], BUILT_IN_TOPIC);
        let sedp_topic_topics_guid = Guid::new(guid_prefix, sedp_topics_entity_id);
        let (sedp_topic_topics, _) = TopicActor::new(
            sedp_topic_topics_guid,
            TopicQos::default(),
            "DiscoveredTopicData".to_string(),
            DCPS_TOPIC,
            None,
            Arc::new(DiscoveredTopicData::get_type()),
            handle,
        );
        topic_list.insert(DCPS_TOPIC.to_owned(), sedp_topic_topics);

        let sedp_publications_entity_id = EntityId::new([0, 0, 2], BUILT_IN_TOPIC);
        let sedp_topic_publications_guid = Guid::new(guid_prefix, sedp_publications_entity_id);
        let (sedp_topic_publications, _) = TopicActor::new(
            sedp_topic_publications_guid,
            TopicQos::default(),
            "DiscoveredWriterData".to_string(),
            DCPS_PUBLICATION,
            None,
            Arc::new(DiscoveredWriterData::get_type()),
            handle,
        );

        topic_list.insert(DCPS_PUBLICATION.to_owned(), sedp_topic_publications);

        let sedp_subscriptions_entity_id = EntityId::new([0, 0, 3], BUILT_IN_TOPIC);
        let sedp_topic_subscriptions_guid = Guid::new(guid_prefix, sedp_subscriptions_entity_id);
        let (sedp_topic_subscriptions, _) = TopicActor::new(
            sedp_topic_subscriptions_guid,
            TopicQos::default(),
            "DiscoveredReaderData".to_string(),
            DCPS_SUBSCRIPTION,
            None,
            Arc::new(DiscoveredReaderData::get_type()),
            handle,
        );
        topic_list.insert(DCPS_SUBSCRIPTION.to_owned(), sedp_topic_subscriptions);

        topic_list
    }

    fn create_builtin_readers(
        &self,
        guid_prefix: GuidPrefix,
        transport: &Arc<Mutex<RtpsParticipant>>,
        topic_list: &HashMap<String, TopicActor>,
        builtin_subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        domain_participant_status_kind: Vec<StatusKind>,
        on_participant_discovery_sender: MpscSender<()>,
        executor_handle: ExecutorHandle,
        timer_handle: TimerHandle,
    ) -> Vec<DataReaderActor> {
        let spdp_reader_qos = DataReaderQos {
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(1),
            },
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: DurationKind::Finite(Duration::new(
                    DURATION_ZERO_SEC,
                    DURATION_ZERO_NSEC,
                )),
            },
            ..Default::default()
        };
        let spdp_builtin_participant_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER);
        let spdp_builtin_transport_reader =
            transport.lock().unwrap().create_builtin_stateless_reader(
                spdp_builtin_participant_reader_guid,
                Box::new(SpdpBuiltinReaderHistoryCache {
                    subscriber_address: builtin_subscriber_address.clone(),
                    reader_instance_handle: InstanceHandle::new(
                        spdp_builtin_participant_reader_guid.into(),
                    ),
                    participant_address: participant_address.clone(),
                    on_participant_discovery_sender,
                }),
            );
        let spdp_builtin_participant_reader = DataReaderActor::new(
            spdp_builtin_participant_reader_guid,
            spdp_builtin_transport_reader,
            DCPS_PARTICIPANT.to_string(),
            "SpdpDiscoveredParticipantData".to_string(),
            Arc::new(SpdpDiscoveredParticipantData::get_type()),
            spdp_reader_qos,
            None,
            vec![],
            vec![],
            domain_participant_status_kind.clone(),
            executor_handle.clone(),
            timer_handle.clone(),
        );

        let sedp_builtin_topics_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        let sedp_builtin_topics_transport_reader =
            transport.lock().unwrap().create_builtin_stateful_reader(
                sedp_builtin_topics_reader_guid,
                Box::new(SedpBuiltinTopicsReaderHistoryCache {
                    subscriber_address: builtin_subscriber_address.clone(),
                    reader_instance_handle: InstanceHandle::new(
                        sedp_builtin_topics_reader_guid.into(),
                    ),
                    participant_address: participant_address.clone(),
                }),
            );
        let sedp_builtin_topics_reader = DataReaderActor::new(
            sedp_builtin_topics_reader_guid,
            sedp_builtin_topics_transport_reader,
            DCPS_TOPIC.to_string(),
            "DiscoveredTopicData".to_string(),
            Arc::new(DiscoveredTopicData::get_type()),
            sedp_data_reader_qos(),
            None,
            vec![],
            vec![],
            domain_participant_status_kind.clone(),
            executor_handle.clone(),
            timer_handle.clone(),
        );

        let sedp_builtin_publications_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let sedp_builtin_publications_transport_reader =
            transport.lock().unwrap().create_builtin_stateful_reader(
                sedp_builtin_publications_reader_guid,
                Box::new(SedpBuiltinPublicationsReaderHistoryCache {
                    subscriber_address: builtin_subscriber_address.clone(),
                    reader_instance_handle: InstanceHandle::new(
                        sedp_builtin_publications_reader_guid.into(),
                    ),
                    participant_address: participant_address.clone(),
                }),
            );
        let sedp_builtin_publications_reader = DataReaderActor::new(
            sedp_builtin_publications_reader_guid,
            sedp_builtin_publications_transport_reader,
            DCPS_PUBLICATION.to_string(),
            "DiscoveredWriterData".to_string(),
            Arc::new(DiscoveredWriterData::get_type()),
            sedp_data_reader_qos(),
            None,
            vec![],
            vec![],
            domain_participant_status_kind.clone(),
            executor_handle.clone(),
            timer_handle.clone(),
        );

        let sedp_builtin_subscriptions_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let sedp_builtin_subscriptions_transport_reader =
            transport.lock().unwrap().create_builtin_stateful_reader(
                sedp_builtin_subscriptions_reader_guid,
                Box::new(SedpBuiltinSubscriptionsReaderHistoryCache {
                    subscriber_address: builtin_subscriber_address.clone(),
                    reader_instance_handle: InstanceHandle::new(
                        sedp_builtin_subscriptions_reader_guid.into(),
                    ),
                    participant_address: participant_address,
                }),
            );
        let sedp_builtin_subscriptions_reader = DataReaderActor::new(
            sedp_builtin_subscriptions_reader_guid,
            sedp_builtin_subscriptions_transport_reader,
            DCPS_SUBSCRIPTION.to_string(),
            "DiscoveredReaderData".to_string(),
            Arc::new(DiscoveredReaderData::get_type()),
            sedp_data_reader_qos(),
            None,
            vec![],
            vec![],
            domain_participant_status_kind,
            executor_handle,
            timer_handle,
        );

        vec![
            spdp_builtin_participant_reader,
            sedp_builtin_topics_reader,
            sedp_builtin_publications_reader,
            sedp_builtin_subscriptions_reader,
        ]
    }

    fn create_builtin_writers(
        &self,
        guid_prefix: GuidPrefix,
        participant: &Arc<Mutex<RtpsParticipant>>,
        topic_list: &HashMap<String, TopicActor>,
        handle: &ExecutorHandle,
    ) -> Vec<DataWriterActor> {
        let spdp_writer_qos = DataWriterQos {
            durability: DurabilityQosPolicy {
                kind: DurabilityQosPolicyKind::TransientLocal,
            },
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLast(1),
            },
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: DurationKind::Finite(Duration::new(
                    DURATION_ZERO_SEC,
                    DURATION_ZERO_NSEC,
                )),
            },
            ..Default::default()
        };
        let spdp_builtin_participant_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER);
        let spdp_builtin_participant_rtps_writer = participant
            .lock()
            .unwrap()
            .create_builtin_stateless_writer(spdp_builtin_participant_writer_guid);
        let spdp_builtin_participant_writer = DataWriterActor::new(
            spdp_builtin_participant_rtps_writer.clone(),
            spdp_builtin_participant_writer_guid,
            Duration::new(0, 200_000_000).into(),
            DCPS_PARTICIPANT.to_string(),
            "SpdpDiscoveredParticipantData".to_string(),
            None,
            vec![],
            spdp_writer_qos,
            handle,
        );

        let sedp_builtin_topics_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
        let sedp_builtin_topics_writer = DataWriterActor::new(
            participant
                .lock()
                .unwrap()
                .create_builtin_stateful_writer(sedp_builtin_topics_writer_guid),
            sedp_builtin_topics_writer_guid,
            Duration::new(0, 200_000_000).into(),
            DCPS_TOPIC.to_string(),
            "DiscoveredTopicData".to_string(),
            None,
            vec![],
            sedp_data_writer_qos(),
            handle,
        );

        let sedp_builtin_publications_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        let sedp_builtin_publications_writer = DataWriterActor::new(
            participant
                .lock()
                .unwrap()
                .create_builtin_stateful_writer(sedp_builtin_publications_writer_guid),
            sedp_builtin_publications_writer_guid,
            Duration::new(0, 200_000_000).into(),
            DCPS_PUBLICATION.to_string(),
            "DiscoveredWriterData".to_string(),
            None,
            vec![],
            sedp_data_writer_qos(),
            handle,
        );

        let sedp_builtin_subscriptions_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let sedp_builtin_subscriptions_writer = DataWriterActor::new(
            participant
                .lock()
                .unwrap()
                .create_builtin_stateful_writer(sedp_builtin_subscriptions_writer_guid),
            sedp_builtin_subscriptions_writer_guid,
            Duration::new(0, 200_000_000).into(),
            DCPS_SUBSCRIPTION.to_string(),
            "DiscoveredReaderData".to_string(),
            None,
            vec![],
            sedp_data_writer_qos(),
            handle,
        );

        vec![
            spdp_builtin_participant_writer,
            sedp_builtin_topics_writer,
            sedp_builtin_publications_writer,
            sedp_builtin_subscriptions_writer,
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
    type Result = DdsResult<ActorAddress<DomainParticipantActor>>;
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

        let socket = std::net::UdpSocket::bind("0.0.0.0:0000")?;
        let message_sender_actor =
            MessageSenderActor::new(socket, PROTOCOLVERSION, VENDOR_ID_S2E, guid_prefix);

        let rtps_participant = RtpsParticipant::new(
            guid_prefix,
            message.domain_id,
            self.configuration.domain_tag().to_string(),
            self.configuration.interface_name(),
            self.configuration.udp_receive_buffer_size(),
        )?;
        let participant_guid = rtps_participant.guid();
        let transport = Arc::new(Mutex::new(rtps_participant));
        let domain_participant_status_kind = message.status_kind.clone();

        let builtin_subscriber = SubscriberActor::new(
            SubscriberQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
            )),
            transport.clone(),
            None,
            vec![],
            domain_participant_status_kind.clone(),
            vec![],
            &executor_handle,
        );
        let builtin_subscriber_status_condition_address =
            builtin_subscriber.get_status_condition_address();
        let builtin_subscriber_actor = Actor::spawn(builtin_subscriber, &executor_handle);
        let builtin_subscriber_address = builtin_subscriber_actor.address();

        let topic_list = self.create_builtin_topics(guid_prefix, &executor.handle());
        let builtin_data_writer_list =
            self.create_builtin_writers(guid_prefix, &transport, &topic_list, &executor_handle);

        let (domain_participant, status_condition) = DomainParticipantActor::new(
            transport.clone(),
            Guid::new(guid_prefix, ENTITYID_PARTICIPANT),
            message.domain_id,
            self.configuration.domain_tag().to_string(),
            domain_participant_qos,
            self.configuration.fragment_size(),
            message.listener,
            message.status_kind,
            builtin_data_writer_list,
            message_sender_actor,
            executor,
            timer_driver,
        );
        let participant_actor = Actor::spawn(domain_participant, &executor_handle);
        let participant_actor_address = participant_actor.address();

        let (on_participant_discovery_sender, on_participant_discovery_receiver) = mpsc_channel();

        let builtin_data_reader_list = self.create_builtin_readers(
            guid_prefix,
            &transport,
            &topic_list,
            builtin_subscriber_address.clone(),
            participant_actor_address,
            domain_participant_status_kind,
            on_participant_discovery_sender,
            executor_handle.clone(),
            timer_handle.clone(),
        );
        for builtin_data_reader in builtin_data_reader_list {
            let instance_handle = builtin_data_reader.get_instance_handle();
            let data_reader_actor = Actor::spawn(builtin_data_reader, &executor_handle);
            builtin_subscriber_actor.send_actor_mail(subscriber_actor::AddDataReaderActor {
                instance_handle,
                data_reader_actor,
            });
        }
        participant_actor.send_actor_mail(domain_participant_actor::SetTopicList { topic_list });
        participant_actor.send_actor_mail(domain_participant_actor::SetBuiltInSubscriber {
            builtin_subscriber: builtin_subscriber_actor,
        });

        //****** Spawn the participant actor and tasks **********//
        let participant = DomainParticipantAsync::new(
            participant_actor.address(),
            status_condition.clone(),
            builtin_subscriber_address,
            builtin_subscriber_status_condition_address,
            message.domain_id,
            executor_handle.clone(),
            timer_handle.clone(),
        );

        // Start the regular participant announcement task
        let participant_clone = participant.clone();
        let participant_announcement_interval =
            self.configuration.participant_announcement_interval();

        executor_handle.spawn(async move {
            loop {
                let r = participant_clone.announce_participant().await;
                if r.is_err() {
                    break;
                }

                timer_handle.sleep(participant_announcement_interval).await;
            }
        });

        let participant_clone = participant.clone();
        executor_handle.spawn(async move {
            loop {
                let r = participant_clone.announce_participant().await;
                if r.is_err() {
                    break;
                }

                on_participant_discovery_receiver.recv().await;
            }
        });

        let participant_address = participant_actor.address();
        self.domain_participant_list.insert(
            InstanceHandle::new(participant_guid.into()),
            participant_actor,
        );
        Ok(participant_address)
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

pub fn sedp_data_reader_qos() -> DataReaderQos {
    DataReaderQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(
                DURATION_ZERO_SEC,
                DURATION_ZERO_NSEC,
            )),
        },
        ..Default::default()
    }
}

pub fn sedp_data_writer_qos() -> DataWriterQos {
    DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(
                DURATION_ZERO_SEC,
                DURATION_ZERO_NSEC,
            )),
        },
        ..Default::default()
    }
}

struct SpdpBuiltinReaderHistoryCache {
    subscriber_address: ActorAddress<SubscriberActor>,
    reader_instance_handle: InstanceHandle,
    participant_address: ActorAddress<DomainParticipantActor>,
    on_participant_discovery_sender: MpscSender<()>,
}

impl ReaderHistoryCache for SpdpBuiltinReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        if let Some(p) = cache_change
            .inline_qos
            .parameter()
            .iter()
            .find(|&x| x.parameter_id() == PID_STATUS_INFO)
        {
            let mut deserializer = Xcdr1LeDeserializer::new(p.value());
            let status_info: StatusInfo =
                XTypesDeserialize::deserialize(&mut deserializer).unwrap();
            if status_info == STATUS_INFO_DISPOSED
                || status_info == STATUS_INFO_DISPOSED_UNREGISTERED
            {
                if let Ok(handle) =
                    InstanceHandle::deserialize_data(cache_change.data_value.as_ref())
                {
                    self.participant_address
                        .send_actor_mail(domain_participant_actor::RemoveDiscoveredParticipant {
                            handle,
                        })
                        .ok();
                }
            }
        } else {
            if let Ok(discovered_participant_data) =
                SpdpDiscoveredParticipantData::deserialize_data(cache_change.data_value.as_ref())
            {
                self.participant_address
                    .send_actor_mail(domain_participant_actor::AddDiscoveredParticipant {
                        discovered_participant_data,
                        // participant: participant.clone(),
                    })
                    .ok();
            }
        }

        self.on_participant_discovery_sender.send(()).ok();

        self.subscriber_address
            .send_actor_mail(subscriber_actor::AddChange {
                cache_change,
                reader_instance_handle: self.reader_instance_handle,
            })
            .ok();
    }
}

struct SedpBuiltinTopicsReaderHistoryCache {
    pub subscriber_address: ActorAddress<SubscriberActor>,
    pub reader_instance_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for SedpBuiltinTopicsReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        if let Ok(discovered_topic_data) =
            DiscoveredTopicData::deserialize_data(cache_change.data_value.as_ref())
        {
            self.participant_address
                .send_actor_mail(domain_participant_actor::AddMatchedTopic {
                    discovered_topic_data,
                    // participant: participant.clone(),
                })
                .ok();
        }
        self.subscriber_address
            .send_actor_mail(subscriber_actor::AddChange {
                cache_change,
                reader_instance_handle: self.reader_instance_handle,
            })
            .ok();
    }
}

struct SedpBuiltinPublicationsReaderHistoryCache {
    pub subscriber_address: ActorAddress<SubscriberActor>,
    pub reader_instance_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for SedpBuiltinPublicationsReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        if let Some(p) = cache_change
            .inline_qos
            .parameter()
            .iter()
            .find(|&x| x.parameter_id() == PID_STATUS_INFO)
        {
            let mut deserializer = Xcdr1LeDeserializer::new(p.value());
            let status_info: StatusInfo =
                XTypesDeserialize::deserialize(&mut deserializer).unwrap();
            if status_info == STATUS_INFO_DISPOSED
                || status_info == STATUS_INFO_DISPOSED_UNREGISTERED
            {
                if let Ok(discovered_writer_handle) =
                    InstanceHandle::deserialize_data(cache_change.data_value.as_ref())
                {
                    self.participant_address
                        .send_actor_mail(domain_participant_actor::RemoveMatchedWriter {
                            discovered_writer_handle,
                        })
                        .ok();
                }
            }
        } else {
            if let Ok(discovered_writer_data) =
                DiscoveredWriterData::deserialize_data(cache_change.data_value.as_ref())
            {
                self.participant_address
                    .send_actor_mail(domain_participant_actor::AddMatchedWriter {
                        discovered_writer_data,
                        // participant: participant.clone(),
                    })
                    .ok();
            }
        }
        self.subscriber_address
            .send_actor_mail(subscriber_actor::AddChange {
                cache_change,
                reader_instance_handle: self.reader_instance_handle,
            })
            .ok();
    }
}

struct SedpBuiltinSubscriptionsReaderHistoryCache {
    pub subscriber_address: ActorAddress<SubscriberActor>,
    pub reader_instance_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for SedpBuiltinSubscriptionsReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        if let Some(p) = cache_change
            .inline_qos
            .parameter()
            .iter()
            .find(|&x| x.parameter_id() == PID_STATUS_INFO)
        {
            let mut deserializer = Xcdr1LeDeserializer::new(p.value());
            let status_info: StatusInfo =
                XTypesDeserialize::deserialize(&mut deserializer).unwrap();
            if status_info == STATUS_INFO_DISPOSED
                || status_info == STATUS_INFO_DISPOSED_UNREGISTERED
            {
                if let Ok(discovered_reader_handle) =
                    InstanceHandle::deserialize_data(cache_change.data_value.as_ref())
                {
                    self.participant_address
                        .send_actor_mail(domain_participant_actor::RemoveMatchedReader {
                            discovered_reader_handle,
                        })
                        .ok();
                }
            }
        } else {
            if let Ok(discovered_reader_data) =
                DiscoveredReaderData::deserialize_data(cache_change.data_value.as_ref())
            {
                self.participant_address
                    .send_actor_mail(domain_participant_actor::AddMatchedReader {
                        discovered_reader_data,
                        // participant: participant.clone(),
                    })
                    .ok();
            }
        }

        self.subscriber_address
            .send_actor_mail(subscriber_actor::AddChange {
                cache_change,
                reader_instance_handle: self.reader_instance_handle,
            })
            .ok();
    }
}
