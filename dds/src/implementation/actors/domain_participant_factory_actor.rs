use super::{
    data_reader_actor::DataReaderActor, data_writer_actor::DataWriterActor,
    message_sender_actor::MessageSenderActor,
};
use crate::{
    configuration::DustDdsConfiguration,
    data_representation_builtin_endpoints::{
        discovered_reader_data::DCPS_SUBSCRIPTION, discovered_topic_data::DCPS_TOPIC,
        discovered_writer_data::DCPS_PUBLICATION,
        spdp_discovered_participant_data::DCPS_PARTICIPANT,
    },
    dds_async::{
        domain_participant::DomainParticipantAsync,
        domain_participant_listener::DomainParticipantListenerAsync,
    },
    domain::domain_participant_factory::DomainId,
    implementation::{
        actor::{Actor, ActorAddress, DEFAULT_ACTOR_BUFFER_SIZE},
        actors::domain_participant_actor::DomainParticipantActor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantFactoryQos, DomainParticipantQos,
            QosKind,
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
        messages::overall_structure::RtpsMessage,
        participant::RtpsParticipant,
        reader::{RtpsReader, RtpsReaderKind, RtpsStatefulReader, RtpsStatelessReader},
        reader_locator::RtpsReaderLocator,
        types::{
            Guid, GuidPrefix, Locator, TopicKind, LOCATOR_INVALID, LOCATOR_KIND_UDP_V4,
            PROTOCOLVERSION, VENDOR_ID_S2E,
        },
        writer::RtpsWriter,
    },
};
use dust_dds_derive::actor_interface;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use std::{
    collections::{hash_map::Entry, HashMap},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, OnceLock,
    },
};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

#[derive(Default)]
pub struct DomainParticipantFactoryActor {
    domain_participant_list: HashMap<InstanceHandle, Actor<DomainParticipantActor>>,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
    configuration: DustDdsConfiguration,
    broadcast_sender_channels: HashMap<DomainId, broadcast::Sender<RtpsMessage>>,
}

impl DomainParticipantFactoryActor {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_unique_participant_id(&mut self) -> u32 {
        static COUNTER: OnceLock<AtomicU32> = OnceLock::new();
        let c = COUNTER.get_or_init(|| AtomicU32::new(0));
        c.fetch_add(1, Ordering::Acquire)
    }

    fn create_builtin_readers(
        &self,
        guid_prefix: GuidPrefix,
        handle: &tokio::runtime::Handle,
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
            "SpdpDiscoveredParticipantData".to_string(),
            String::from(DCPS_PARTICIPANT),
            spdp_reader_qos,
            None,
            vec![],
            handle,
        );

        let sedp_builtin_topics_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
        let sedp_builtin_topics_reader = DataReaderActor::new(
            create_builtin_stateful_reader(sedp_builtin_topics_reader_guid),
            "DiscoveredTopicData".to_string(),
            String::from(DCPS_TOPIC),
            sedp_data_reader_qos(),
            None,
            vec![],
            handle,
        );

        let sedp_builtin_publications_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
        let sedp_builtin_publications_reader = DataReaderActor::new(
            create_builtin_stateful_reader(sedp_builtin_publications_reader_guid),
            "DiscoveredWriterData".to_string(),
            String::from(DCPS_PUBLICATION),
            sedp_data_reader_qos(),
            None,
            vec![],
            handle,
        );

        let sedp_builtin_subscriptions_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let sedp_builtin_subscriptions_reader = DataReaderActor::new(
            create_builtin_stateful_reader(sedp_builtin_subscriptions_reader_guid),
            "DiscoveredReaderData".to_string(),
            String::from(DCPS_SUBSCRIPTION),
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
        spdp_discovery_locator_list: &[Locator],
        handle: &tokio::runtime::Handle,
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
            "SpdpDiscoveredParticipantData".to_string(),
            String::from(DCPS_PARTICIPANT),
            None,
            vec![],
            spdp_writer_qos,
            handle,
        );

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
            "DiscoveredTopicData".to_string(),
            String::from(DCPS_TOPIC),
            None,
            vec![],
            sedp_data_writer_qos(),
            handle,
        );

        let sedp_builtin_publications_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
        let sedp_builtin_publications_writer = DataWriterActor::new(
            create_builtin_stateful_writer(sedp_builtin_publications_writer_guid),
            "DiscoveredWriterData".to_string(),
            String::from(DCPS_PUBLICATION),
            None,
            vec![],
            sedp_data_writer_qos(),
            handle,
        );

        let sedp_builtin_subscriptions_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
        let sedp_builtin_subscriptions_writer = DataWriterActor::new(
            create_builtin_stateful_writer(sedp_builtin_subscriptions_writer_guid),
            "DiscoveredReaderData".to_string(),
            String::from(DCPS_SUBSCRIPTION),
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

pub async fn read_message(socket: &mut tokio::net::UdpSocket) -> DdsResult<RtpsMessage> {
    let mut buf = vec![0; 65507];
    let (bytes, _) = socket.recv_from(&mut buf).await?;
    buf.truncate(bytes);
    if bytes > 0 {
        Ok(RtpsMessage::try_from(Arc::from(buf.into_boxed_slice()))?)
    } else {
        Err(DdsError::NoData)
    }
}

#[actor_interface]
impl DomainParticipantFactoryActor {
    async fn create_participant(
        &mut self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        runtime_handle: tokio::runtime::Handle,
    ) -> DdsResult<ActorAddress<DomainParticipantActor>> {
        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let interface_address_list =
            get_interface_address_list(self.configuration.interface_name());

        let host_id = if let Some(interface) = interface_address_list.first() {
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

        #[rustfmt::skip]
        let guid_prefix = [
            host_id[0],  host_id[1], host_id[2], host_id[3], // Host ID
            app_id[0], app_id[1], app_id[2], app_id[3], // App ID
            instance_id[0], instance_id[1], instance_id[2], instance_id[3], // Instance ID
        ];

        let default_unicast_socket = if self.configuration.udp_transport_enabled() {
            let default_unicast_socket =
                socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None).map_err(
                    |_| DdsError::Error("Failed to create default unicast socket".to_string()),
                )?;
            default_unicast_socket
                .bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)).into())
                .map_err(|_| {
                    DdsError::Error("Failed to bind to default unicast socket".to_string())
                })?;
            default_unicast_socket
                .set_nonblocking(true)
                .map_err(|_| DdsError::Error("Failed to set socket non-blocking".to_string()))?;
            if let Some(buffer_size) = self.configuration.udp_receive_buffer_size() {
                default_unicast_socket
                    .set_recv_buffer_size(buffer_size)
                    .map_err(|_| {
                        DdsError::Error(
                            "Failed to set default unicast socket receive buffer size".to_string(),
                        )
                    })?;
            }

            Some(std::net::UdpSocket::from(default_unicast_socket))
        } else {
            None
        };

        let metattrafic_unicast_socket = if self.configuration.udp_transport_enabled() {
            let metattrafic_unicast_socket =
                std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))).map_err(
                    |_| DdsError::Error("Failed to open metatraffic socket".to_string()),
                )?;
            metattrafic_unicast_socket
                .set_nonblocking(true)
                .map_err(|_| {
                    DdsError::Error("Failed to set metatraffic socket non-blocking".to_string())
                })?;
            Some(metattrafic_unicast_socket)
        } else {
            None
        };

        let default_unicast_locator_list: Vec<Locator> = if let Some(s) = &default_unicast_socket {
            let port = s
                .local_addr()
                .map_err(|_| DdsError::Error("Failed to get socket address".to_string()))?
                .port()
                .into();
            interface_address_list
                .iter()
                .map(|a| Locator::from_ip_and_port(a, port))
                .collect()
        } else {
            vec![]
        };

        let default_multicast_locator_list = vec![];

        let metatraffic_unicast_locator_list: Vec<Locator> =
            if let Some(s) = &metattrafic_unicast_socket {
                let port = s
                    .local_addr()
                    .map_err(|_| {
                        DdsError::Error("Failed to get metatraffic socket address".to_string())
                    })?
                    .port()
                    .into();
                interface_address_list
                    .iter()
                    .map(|a| Locator::from_ip_and_port(a, port))
                    .collect()
            } else {
                vec![]
            };

        let metatraffic_multicast_locator_list = if self.configuration.udp_transport_enabled() {
            vec![Locator::new(
                LOCATOR_KIND_UDP_V4,
                port_builtin_multicast(domain_id) as u32,
                DEFAULT_MULTICAST_LOCATOR_ADDRESS,
            )]
        } else {
            vec![]
        };

        let spdp_discovery_locator_list = if metatraffic_multicast_locator_list.is_empty() {
            // This is a placeholder to have at least one locator on the list
            // of the SPDP stateless writer which is needed to make sure messages get
            // sent.
            vec![LOCATOR_INVALID]
        } else {
            metatraffic_multicast_locator_list.clone()
        };

        let socket = if self.configuration.udp_transport_enabled() {
            Some(std::net::UdpSocket::bind("0.0.0.0:0000")?)
        } else {
            None
        };
        let broadcast_sender = match self.broadcast_sender_channels.entry(domain_id) {
            Entry::Vacant(e) => {
                let (s, _) = broadcast::channel(32);
                e.insert(s.clone());
                s
            }
            Entry::Occupied(e) => e.get().clone(),
        };
        let message_sender_actor = MessageSenderActor::new(
            socket,
            broadcast_sender.clone(),
            PROTOCOLVERSION,
            VENDOR_ID_S2E,
            guid_prefix,
        );

        let rtps_participant = RtpsParticipant::new(
            guid_prefix,
            default_unicast_locator_list,
            default_multicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            PROTOCOLVERSION,
            VENDOR_ID_S2E,
        );
        let participant_guid = rtps_participant.guid();

        let builtin_data_writer_list =
            self.create_builtin_writers(guid_prefix, &spdp_discovery_locator_list, &runtime_handle);
        let builtin_data_reader_list = self.create_builtin_readers(guid_prefix, &runtime_handle);

        let domain_participant = DomainParticipantActor::new(
            rtps_participant,
            domain_id,
            self.configuration.domain_tag().to_string(),
            domain_participant_qos,
            self.configuration.fragment_size(),
            listener,
            status_kind,
            builtin_data_writer_list,
            builtin_data_reader_list,
            message_sender_actor,
            &runtime_handle,
        );

        let status_condition = domain_participant.get_statuscondition();
        let builtin_subscriber = domain_participant.get_built_in_subscriber();
        let builtin_subscriber_status_condition_address =
            builtin_subscriber.upgrade()?.get_statuscondition().await;

        let participant_actor = Actor::spawn(
            domain_participant,
            &runtime_handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );
        let participant_address = participant_actor.address();
        self.domain_participant_list.insert(
            InstanceHandle::new(participant_guid.into()),
            participant_actor,
        );
        let participant = DomainParticipantAsync::new(
            participant_address.clone(),
            status_condition.clone(),
            builtin_subscriber,
            builtin_subscriber_status_condition_address,
            domain_id,
            runtime_handle.clone(),
        );

        let participant_address_clone = participant_address.clone();
        let participant_clone = participant.clone();
        let mut socket = get_multicast_socket(
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
            port_builtin_multicast(domain_id),
            &interface_address_list,
        )
        .map_err(|_| DdsError::Error("Failed to open socket".to_string()))?;
        runtime_handle.spawn(async move {
            loop {
                if let Ok(message) = read_message(&mut socket).await {
                    if let Ok(p) = participant_address_clone.upgrade() {
                        let r = p
                            .process_metatraffic_rtps_message(message, participant_clone.clone())
                            .await;

                        if r.is_err() {
                            error!("Error processing metatraffic RTPS message. {:?}", r);
                        }

                        p.send_message().await;
                    } else {
                        break;
                    };
                }
            }
        });

        let participant_address_clone = participant_address.clone();
        let participant_clone = participant.clone();

        if let Some(metattrafic_unicast_socket) = metattrafic_unicast_socket {
            let mut socket =
                tokio::net::UdpSocket::from_std(metattrafic_unicast_socket).map_err(|_| {
                    DdsError::Error("Failed to open metattrafic unicast socket".to_string())
                })?;
            runtime_handle.spawn(async move {
                loop {
                    if let Ok(message) = read_message(&mut socket).await {
                        if let Ok(p) = participant_address_clone.upgrade() {
                            let r = p
                                .process_metatraffic_rtps_message(
                                    message,
                                    participant_clone.clone(),
                                )
                                .await;
                            if r.is_err() {
                                error!("Error processing metatraffic RTPS message. {:?}", r);
                            }

                            p.send_message().await;
                        } else {
                            break;
                        }
                    }
                }
            });
        }

        if let Some(default_unicast_socket) = default_unicast_socket {
            let participant_address_clone = participant_address.clone();
            let participant_clone = participant.clone();
            let mut socket =
                tokio::net::UdpSocket::from_std(default_unicast_socket).map_err(|_| {
                    DdsError::Error("Failed to open default unicast socket".to_string())
                })?;
            runtime_handle.spawn(async move {
                loop {
                    if let Ok(message) = read_message(&mut socket).await {
                        if let Ok(p) = participant_address_clone.upgrade() {
                            p.process_user_defined_rtps_message(message, participant_clone.clone())
                                .await;
                        } else {
                            break;
                        }
                    }
                }
            });
        }

        let participant_address_clone = participant_address.clone();
        let mut interval =
            tokio::time::interval(self.configuration.participant_announcement_interval());
        runtime_handle.spawn(async move {
            loop {
                interval.tick().await;
                if let Ok(p) = participant_address_clone.upgrade() {
                    let r = p.announce_participant().await;
                    if r.is_err() {
                        error!("Error announcing participant: {:?}", r);
                    }
                } else {
                    break;
                }
            }
        });

        let participant_address_clone = participant_address.clone();
        let participant_clone = participant.clone();
        let mut receiver = broadcast_sender.subscribe();
        runtime_handle.spawn(async move {
            loop {
                if let Ok(message) = receiver.recv().await {
                    if let Ok(p) = participant_address_clone.upgrade() {
                        let r = p
                            .process_metatraffic_rtps_message(message, participant_clone.clone())
                            .await;
                        if r.is_err() {
                            error!("Error processing metatraffic RTPS message. {:?}", r);
                        }
                    } else {
                        break;
                    }
                }
            }
        });

        let participant_address_clone = participant_address.clone();
        let participant_clone = participant.clone();
        let mut receiver = broadcast_sender.subscribe();
        runtime_handle.spawn(async move {
            loop {
                if let Ok(message) = receiver.recv().await {
                    if let Ok(p) = participant_address_clone.upgrade() {
                        p.process_user_defined_rtps_message(message, participant_clone.clone())
                            .await;
                    } else {
                        break;
                    }
                }
            }
        });

        Ok(participant_address)
    }

    async fn delete_participant(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        let is_participant_empty = self.domain_participant_list[&handle].is_empty().await;
        if is_participant_empty {
            self.domain_participant_list.remove(&handle);
            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Domain participant still contains other entities".to_string(),
            ))
        }
    }

    async fn lookup_participant(
        &self,
        domain_id: DomainId,
    ) -> DdsResult<Option<ActorAddress<DomainParticipantActor>>> {
        for dp in self.domain_participant_list.values() {
            if dp.get_domain_id().await == domain_id {
                return Ok(Some(dp.address()));
            }
        }

        Ok(None)
    }

    fn set_default_participant_qos(&mut self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_participant_qos = qos;

        Ok(())
    }

    fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        Ok(self.default_participant_qos.clone())
    }

    fn set_qos(&mut self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;

        Ok(())
    }

    fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        Ok(self.qos.clone())
    }

    fn set_configuration(&mut self, configuration: DustDdsConfiguration) -> DdsResult<()> {
        self.configuration = configuration;
        Ok(())
    }

    fn get_configuration(&self) -> DdsResult<DustDdsConfiguration> {
        Ok(self.configuration.clone())
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

fn port_builtin_multicast(domain_id: DomainId) -> u16 {
    (PB + DG * domain_id + d0) as u16
}

fn get_interface_address_list(interface_name: Option<&String>) -> Vec<Addr> {
    NetworkInterface::show()
        .expect("Could not scan interfaces")
        .into_iter()
        .filter(|x| {
            if let Some(if_name) = interface_name {
                &x.name == if_name
            } else {
                true
            }
        })
        .flat_map(|i| {
            i.addr.into_iter().filter(|a| match a {
                #[rustfmt::skip]
                Addr::V4(v4) if !v4.ip.is_loopback() => true,
                _ => false,
            })
        })
        .collect()
}

fn get_multicast_socket(
    multicast_address: LocatorAddress,
    port: u16,
    interface_address_list: &[Addr],
) -> std::io::Result<tokio::net::UdpSocket> {
    let socket_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;

    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
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

    tokio::net::UdpSocket::from_std(socket.into())
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
    const DEFAULT_HEARTBEAT_PERIOD: Duration = Duration::new(2, 0);
    const DEFAULT_NACK_RESPONSE_DELAY: Duration = Duration::new(0, 200);
    const DEFAULT_NACK_SUPPRESSION_DURATION: Duration =
        Duration::new(DURATION_ZERO_SEC, DURATION_ZERO_NSEC);

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
        DURATION_ZERO,
        DURATION_ZERO,
        DURATION_ZERO,
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
