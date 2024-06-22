use super::{
    data_reader_actor::DataReaderActor,
    data_writer_actor::DataWriterActor,
    domain_participant_actor::{self, FooTypeSupport},
    message_sender_actor::MessageSenderActor,
    status_condition_actor::StatusConditionActor,
    topic_actor::TopicActor,
};
use crate::{
    configuration::DustDdsConfiguration,
    data_representation_builtin_endpoints::{
        discovered_reader_data::{DiscoveredReaderData, DCPS_SUBSCRIPTION},
        discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
        discovered_writer_data::{DiscoveredWriterData, DCPS_PUBLICATION},
        spdp_discovered_participant_data::{SpdpDiscoveredParticipantData, DCPS_PARTICIPANT},
    },
    dds_async::{
        domain_participant::DomainParticipantAsync,
        domain_participant_listener::DomainParticipantListenerAsync,
    },
    domain::domain_participant_factory::DomainId,
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        actors::domain_participant_actor::DomainParticipantActor,
        runtime::{
            executor::{Executor, ExecutorHandle},
            timer::TimerDriver,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantFactoryQos, DomainParticipantQos,
            QosKind, TopicQos,
        },
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::StatusKind,
        time::{Duration, DurationKind, DURATION_ZERO_NSEC, DURATION_ZERO_SEC},
    },
    rtps::{
        behavior_types::DURATION_ZERO,
        discovery_types::{
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
        },
        endpoint::RtpsEndpoint,
        messages::overall_structure::RtpsMessageRead,
        participant::RtpsParticipant,
        reader::{RtpsReader, RtpsReaderKind, RtpsStatefulReader, RtpsStatelessReader},
        reader_locator::RtpsReaderLocator,
        types::{
            EntityId, Guid, GuidPrefix, Locator, TopicKind, BUILT_IN_TOPIC, LOCATOR_KIND_UDP_V4,
            PROTOCOLVERSION, VENDOR_ID_S2E,
        },
        writer::RtpsWriter,
    },
};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, OnceLock,
    },
};
use tracing::{info, warn};

const MAX_DATAGRAM_SIZE: usize = 65507;

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
    ) -> HashMap<String, (Actor<TopicActor>, ActorAddress<StatusConditionActor>)> {
        let mut topic_list = HashMap::new();

        let spdp_topic_entity_id = EntityId::new([0, 0, 0], BUILT_IN_TOPIC);
        let spdp_topic_guid = Guid::new(guid_prefix, spdp_topic_entity_id);
        let (spdp_topic_participant, spdp_topic_participant_status_condition) = TopicActor::new(
            spdp_topic_guid,
            TopicQos::default(),
            "SpdpDiscoveredParticipantData".to_string(),
            DCPS_PARTICIPANT,
            None,
            Arc::new(FooTypeSupport::new::<SpdpDiscoveredParticipantData>()),
            handle,
        );
        topic_list.insert(
            DCPS_PARTICIPANT.to_owned(),
            (
                Actor::spawn(spdp_topic_participant, handle),
                spdp_topic_participant_status_condition,
            ),
        );

        let sedp_topics_entity_id = EntityId::new([0, 0, 1], BUILT_IN_TOPIC);
        let sedp_topic_topics_guid = Guid::new(guid_prefix, sedp_topics_entity_id);
        let (sedp_topic_topics, sedp_topic_topics_status_condition) = TopicActor::new(
            sedp_topic_topics_guid,
            TopicQos::default(),
            "DiscoveredTopicData".to_string(),
            DCPS_TOPIC,
            None,
            Arc::new(FooTypeSupport::new::<DiscoveredTopicData>()),
            handle,
        );
        topic_list.insert(
            DCPS_TOPIC.to_owned(),
            (
                Actor::spawn(sedp_topic_topics, handle),
                sedp_topic_topics_status_condition,
            ),
        );

        let sedp_publications_entity_id = EntityId::new([0, 0, 2], BUILT_IN_TOPIC);
        let sedp_topic_publications_guid = Guid::new(guid_prefix, sedp_publications_entity_id);
        let (sedp_topic_publications, sedp_topic_publications_status_condition) = TopicActor::new(
            sedp_topic_publications_guid,
            TopicQos::default(),
            "DiscoveredWriterData".to_string(),
            DCPS_PUBLICATION,
            None,
            Arc::new(FooTypeSupport::new::<DiscoveredWriterData>()),
            handle,
        );

        topic_list.insert(
            DCPS_PUBLICATION.to_owned(),
            (
                Actor::spawn(sedp_topic_publications, handle),
                sedp_topic_publications_status_condition,
            ),
        );

        let sedp_subscriptions_entity_id = EntityId::new([0, 0, 3], BUILT_IN_TOPIC);
        let sedp_topic_subscriptions_guid = Guid::new(guid_prefix, sedp_subscriptions_entity_id);
        let (sedp_topic_subscriptions, sedp_topic_subscriptions_status_condition) = TopicActor::new(
            sedp_topic_subscriptions_guid,
            TopicQos::default(),
            "DiscoveredReaderData".to_string(),
            DCPS_SUBSCRIPTION,
            None,
            Arc::new(FooTypeSupport::new::<DiscoveredReaderData>()),
            handle,
        );
        topic_list.insert(
            DCPS_SUBSCRIPTION.to_owned(),
            (
                Actor::spawn(sedp_topic_subscriptions, handle),
                sedp_topic_subscriptions_status_condition,
            ),
        );

        topic_list
    }

    fn create_builtin_readers(
        &self,
        guid_prefix: GuidPrefix,
        topic_list: &HashMap<String, (Actor<TopicActor>, ActorAddress<StatusConditionActor>)>,
        handle: &ExecutorHandle,
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
        let spdp_builtin_participant_reader = DataReaderActor::new(
            create_builtin_stateless_reader(spdp_builtin_participant_reader_guid),
            topic_list[DCPS_PARTICIPANT].0.address(),
            DCPS_PARTICIPANT.to_string(),
            "SpdpDiscoveredParticipantData".to_string(),
            topic_list[DCPS_PARTICIPANT].1.clone(),
            Arc::new(FooTypeSupport::new::<SpdpDiscoveredParticipantData>()),
            spdp_reader_qos,
            None,
            vec![],
            handle,
        );

        let sedp_builtin_topics_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        let sedp_builtin_topics_reader = DataReaderActor::new(
            create_builtin_stateful_reader(sedp_builtin_topics_reader_guid),
            topic_list[DCPS_TOPIC].0.address(),
            DCPS_TOPIC.to_string(),
            "DiscoveredTopicData".to_string(),
            topic_list[DCPS_TOPIC].1.clone(),
            Arc::new(FooTypeSupport::new::<DiscoveredTopicData>()),
            sedp_data_reader_qos(),
            None,
            vec![],
            handle,
        );

        let sedp_builtin_publications_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let sedp_builtin_publications_reader = DataReaderActor::new(
            create_builtin_stateful_reader(sedp_builtin_publications_reader_guid),
            topic_list[DCPS_PUBLICATION].0.address(),
            DCPS_PUBLICATION.to_string(),
            "DiscoveredWriterData".to_string(),
            topic_list[DCPS_PUBLICATION].1.clone(),
            Arc::new(FooTypeSupport::new::<DiscoveredWriterData>()),
            sedp_data_reader_qos(),
            None,
            vec![],
            handle,
        );

        let sedp_builtin_subscriptions_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let sedp_builtin_subscriptions_reader = DataReaderActor::new(
            create_builtin_stateful_reader(sedp_builtin_subscriptions_reader_guid),
            topic_list[DCPS_SUBSCRIPTION].0.address(),
            DCPS_SUBSCRIPTION.to_string(),
            "DiscoveredReaderData".to_string(),
            topic_list[DCPS_SUBSCRIPTION].1.clone(),
            Arc::new(FooTypeSupport::new::<DiscoveredReaderData>()),
            sedp_data_reader_qos(),
            None,
            vec![],
            handle,
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
        domain_id: DomainId,
        topic_list: &HashMap<String, (Actor<TopicActor>, ActorAddress<StatusConditionActor>)>,
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
        let mut spdp_builtin_participant_writer = DataWriterActor::new(
            create_builtin_stateless_writer(spdp_builtin_participant_writer_guid),
            topic_list[DCPS_PARTICIPANT].0.address(),
            DCPS_PARTICIPANT.to_string(),
            "SpdpDiscoveredParticipantData".to_string(),
            topic_list[DCPS_PARTICIPANT].1.clone(),
            None,
            vec![],
            spdp_writer_qos,
            handle,
        );

        let spdp_discovery_locator_list = [Locator::new(
            LOCATOR_KIND_UDP_V4,
            port_builtin_multicast(domain_id) as u32,
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
        )];

        for reader_locator in spdp_discovery_locator_list
            .iter()
            .map(|&locator| RtpsReaderLocator::new(locator, false))
        {
            spdp_builtin_participant_writer.reader_locator_add(reader_locator);
        }

        let sedp_builtin_topics_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
        let sedp_builtin_topics_writer = DataWriterActor::new(
            create_builtin_stateful_writer(sedp_builtin_topics_writer_guid),
            topic_list[DCPS_TOPIC].0.address(),
            DCPS_TOPIC.to_string(),
            "DiscoveredTopicData".to_string(),
            topic_list[DCPS_TOPIC].1.clone(),
            None,
            vec![],
            sedp_data_writer_qos(),
            handle,
        );

        let sedp_builtin_publications_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        let sedp_builtin_publications_writer = DataWriterActor::new(
            create_builtin_stateful_writer(sedp_builtin_publications_writer_guid),
            topic_list[DCPS_PUBLICATION].0.address(),
            DCPS_PUBLICATION.to_string(),
            "DiscoveredWriterData".to_string(),
            topic_list[DCPS_PUBLICATION].1.clone(),
            None,
            vec![],
            sedp_data_writer_qos(),
            handle,
        );

        let sedp_builtin_subscriptions_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let sedp_builtin_subscriptions_writer = DataWriterActor::new(
            create_builtin_stateful_writer(sedp_builtin_subscriptions_writer_guid),
            topic_list[DCPS_SUBSCRIPTION].0.address(),
            DCPS_SUBSCRIPTION.to_string(),
            "DiscoveredReaderData".to_string(),
            topic_list[DCPS_SUBSCRIPTION].1.clone(),
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

pub fn read_message(
    socket: &mut std::net::UdpSocket,
    buf: &mut [u8],
) -> DdsResult<RtpsMessageRead> {
    let (bytes, _) = socket.recv_from(buf)?;
    if bytes > 0 {
        Ok(RtpsMessageRead::try_from(&buf[0..bytes])?)
    } else {
        Err(DdsError::NoData)
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

        let domain_participant_qos = match message.qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let guid_prefix = self.create_new_guid_prefix();

        let socket = std::net::UdpSocket::bind("0.0.0.0:0000")?;
        let message_sender_actor =
            MessageSenderActor::new(socket, PROTOCOLVERSION, VENDOR_ID_S2E, guid_prefix);

        let mut rtps_participant = RtpsParticipant::new(
            guid_prefix,
            vec![],
            vec![],
            vec![],
            vec![],
            PROTOCOLVERSION,
            VENDOR_ID_S2E,
        );
        let participant_guid = rtps_participant.guid();

        let topic_list = self.create_builtin_topics(guid_prefix, &executor.handle());
        let builtin_data_writer_list = self.create_builtin_writers(
            guid_prefix,
            message.domain_id,
            &topic_list,
            &executor_handle,
        );
        let builtin_data_reader_list =
            self.create_builtin_readers(guid_prefix, &topic_list, &executor_handle);

        // Open socket for unicast user-defined data
        let interface_address_list = NetworkInterface::show()
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
                i.addr.into_iter().filter(|a| match a {
                    #[rustfmt::skip]
                    Addr::V4(_) => true,
                    _ => false,
                })
            });

        let default_unicast_socket =
            socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None)?;
        default_unicast_socket.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)).into())?;
        default_unicast_socket.set_nonblocking(false)?;
        if let Some(buffer_size) = self.configuration.udp_receive_buffer_size() {
            default_unicast_socket.set_recv_buffer_size(buffer_size)?;
        }
        let mut default_unicast_socket = std::net::UdpSocket::from(default_unicast_socket);
        let user_defined_unicast_port = default_unicast_socket.local_addr()?.port().into();
        let default_unicast_locator_list: Vec<Locator> = interface_address_list
            .clone()
            .map(|a| Locator::from_ip_and_port(&a, user_defined_unicast_port))
            .collect();
        rtps_participant.set_default_unicast_locator_list(default_unicast_locator_list);

        // Open socket for unicast metatraffic data
        let mut metatrafic_unicast_socket =
            std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))?;
        metatrafic_unicast_socket.set_nonblocking(false)?;
        let metattrafic_unicast_locator_port =
            metatrafic_unicast_socket.local_addr()?.port().into();
        let metatraffic_unicast_locator_list: Vec<Locator> = interface_address_list
            .clone()
            .map(|a| Locator::from_ip_and_port(&a, metattrafic_unicast_locator_port))
            .collect();
        rtps_participant.set_metatraffic_unicast_locator_list(metatraffic_unicast_locator_list);

        // Open socket for multicast metatraffic data
        let metatraffic_multicast_locator_list = vec![Locator::new(
            LOCATOR_KIND_UDP_V4,
            port_builtin_multicast(message.domain_id) as u32,
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
        )];
        rtps_participant.set_metatraffic_multicast_locator_list(metatraffic_multicast_locator_list);

        let timer_driver = TimerDriver::new();
        let timer_handle = timer_driver.handle();
        //****** Spawn the participant actor and tasks **********//
        let (
            domain_participant,
            status_condition,
            builtin_subscriber,
            builtin_subscriber_status_condition_address,
        ) = DomainParticipantActor::new(
            rtps_participant,
            message.domain_id,
            self.configuration.domain_tag().to_string(),
            domain_participant_qos,
            self.configuration.fragment_size(),
            message.listener,
            message.status_kind,
            topic_list,
            builtin_data_writer_list,
            builtin_data_reader_list,
            message_sender_actor,
            executor,
            timer_driver,
        );
        let participant_actor = Actor::spawn(domain_participant, &executor_handle);
        let participant = DomainParticipantAsync::new(
            participant_actor.address(),
            status_condition.clone(),
            builtin_subscriber,
            builtin_subscriber_status_condition_address,
            message.domain_id,
            executor_handle.clone(),
            timer_handle.clone(),
        );

        let participant_address_clone = participant_actor.address();
        let participant_clone = participant.clone();

        std::thread::spawn(move || {
            let mut buf = Box::new([0; MAX_DATAGRAM_SIZE]);
            loop {
                if let Ok(message) = read_message(&mut default_unicast_socket, buf.as_mut_slice()) {
                    let r = participant_address_clone.send_actor_mail(
                        domain_participant_actor::ProcessUserDefinedRtpsMessage {
                            rtps_message: message,
                            participant: participant_clone.clone(),
                            executor_handle: participant_clone.executor_handle().clone(),
                        },
                    );
                    if r.is_err() {
                        break;
                    }
                }
            }
        });

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

        let participant_address_clone = participant_actor.address();
        let participant_clone = participant.clone();

        std::thread::spawn(move || {
            let mut buf = Box::new([0; MAX_DATAGRAM_SIZE]);
            loop {
                if let Ok(message) =
                    read_message(&mut metatrafic_unicast_socket, buf.as_mut_slice())
                {
                    let r = participant_address_clone.send_actor_mail(
                        domain_participant_actor::ProcessMetatrafficRtpsMessage {
                            rtps_message: message,
                            participant: participant_clone.clone(),
                            executor_handle: participant_clone.executor_handle().clone(),
                        },
                    );

                    if r.is_err() {
                        break;
                    }
                }
            }
        });

        let participant_address_clone = participant_actor.address();
        let participant_clone = participant.clone();
        let mut socket = get_multicast_socket(
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
            port_builtin_multicast(message.domain_id),
            interface_address_list,
        )?;
        std::thread::spawn(move || {
            let mut buf = Box::new([0; MAX_DATAGRAM_SIZE]);
            loop {
                if let Ok(message) = read_message(&mut socket, buf.as_mut_slice()) {
                    let r = participant_address_clone.send_actor_mail(
                        domain_participant_actor::ProcessMetatrafficRtpsMessage {
                            rtps_message: message,
                            participant: participant_clone.clone(),
                            executor_handle: participant_clone.executor_handle().clone(),
                        },
                    );

                    if r.is_err() {
                        break;
                    }
                }
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

type LocatorAddress = [u8; 16];
// As of 9.6.1.4.1  Default multicast address
const DEFAULT_MULTICAST_LOCATOR_ADDRESS: LocatorAddress =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1];

const PB: i32 = 7400;
const DG: i32 = 250;
#[allow(non_upper_case_globals)]
const d0: i32 = 0;
const DEFAULT_HEARTBEAT_PERIOD: Duration = Duration::new(2, 0);
const DEFAULT_NACK_RESPONSE_DELAY: Duration = Duration::new(0, 200);
const DEFAULT_NACK_SUPPRESSION_DURATION: Duration =
    Duration::new(DURATION_ZERO_SEC, DURATION_ZERO_NSEC);

fn port_builtin_multicast(domain_id: DomainId) -> u16 {
    (PB + DG * domain_id + d0) as u16
}

fn get_multicast_socket(
    multicast_address: LocatorAddress,
    port: u16,
    interface_address_list: impl IntoIterator<Item = Addr>,
) -> std::io::Result<std::net::UdpSocket> {
    let socket_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;

    socket.set_reuse_address(true)?;
    socket.set_nonblocking(false)?;
    socket.set_read_timeout(Some(std::time::Duration::from_millis(50)))?;

    socket.bind(&socket_addr.into())?;
    let addr = Ipv4Addr::new(
        multicast_address[12],
        multicast_address[13],
        multicast_address[14],
        multicast_address[15],
    );
    for interface_addr in interface_address_list {
        match interface_addr {
            Addr::V4(a) => {
                let r = socket.join_multicast_v4(&addr, &a.ip);
                if let Err(e) = r {
                    info!(
                        "Failed to join multicast group on address {} with error {}",
                        a.ip, e
                    )
                }
            }
            Addr::V6(_) => (),
        }
    }

    socket.set_multicast_loop_v4(true)?;

    Ok(socket.into())
}

fn create_builtin_stateless_reader(guid: Guid) -> RtpsReaderKind {
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];

    RtpsReaderKind::Stateless(RtpsStatelessReader::new(RtpsReader::new(
        RtpsEndpoint::new(
            guid,
            TopicKind::WithKey,
            unicast_locator_list,
            multicast_locator_list,
        ),
        DURATION_ZERO,
        DURATION_ZERO,
        false,
    )))
}

fn create_builtin_stateful_reader(guid: Guid) -> RtpsReaderKind {
    const DEFAULT_HEARTBEAT_SUPPRESSION_DURATION: Duration =
        Duration::new(DURATION_ZERO_SEC, DURATION_ZERO_NSEC);
    const DEFAULT_HEARTBEAT_RESPONSE_DELAY: Duration = Duration::new(0, 500);

    let topic_kind = TopicKind::WithKey;
    let heartbeat_response_delay = DEFAULT_HEARTBEAT_SUPPRESSION_DURATION.into();
    let heartbeat_suppression_duration = DEFAULT_HEARTBEAT_RESPONSE_DELAY.into();
    let expects_inline_qos = false;
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];

    RtpsReaderKind::Stateful(RtpsStatefulReader::new(RtpsReader::new(
        RtpsEndpoint::new(
            guid,
            topic_kind,
            unicast_locator_list,
            multicast_locator_list,
        ),
        heartbeat_response_delay,
        heartbeat_suppression_duration,
        expects_inline_qos,
    )))
}

fn create_builtin_stateful_writer(guid: Guid) -> RtpsWriter {
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];
    let topic_kind = TopicKind::WithKey;
    let push_mode = true;
    let heartbeat_period = DEFAULT_HEARTBEAT_PERIOD.into();
    let nack_response_delay = DEFAULT_NACK_RESPONSE_DELAY.into();
    let nack_suppression_duration = DEFAULT_NACK_SUPPRESSION_DURATION.into();
    let data_max_size_serialized = usize::MAX;

    RtpsWriter::new(
        RtpsEndpoint::new(
            guid,
            topic_kind,
            unicast_locator_list,
            multicast_locator_list,
        ),
        push_mode,
        heartbeat_period,
        nack_response_delay,
        nack_suppression_duration,
        data_max_size_serialized,
    )
}

fn create_builtin_stateless_writer(guid: Guid) -> RtpsWriter {
    let heartbeat_period = DEFAULT_HEARTBEAT_PERIOD.into();
    let nack_response_delay = DEFAULT_NACK_RESPONSE_DELAY.into();
    let nack_suppression_duration = DEFAULT_NACK_SUPPRESSION_DURATION.into();
    let unicast_locator_list = &[];
    let multicast_locator_list = &[];

    RtpsWriter::new(
        RtpsEndpoint::new(
            guid,
            TopicKind::WithKey,
            unicast_locator_list,
            multicast_locator_list,
        ),
        true,
        heartbeat_period,
        nack_response_delay,
        nack_suppression_duration,
        usize::MAX,
    )
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
