use core::net::{Ipv4Addr, SocketAddr};

use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use tracing::info;

use crate::{
    domain::domain_participant_factory::DomainId,
    implementation::{
        actor::{Actor, ActorAddress, ActorBuilder},
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        runtime::executor::{block_on, Executor},
    },
    rtps::{
        discovery_types::{
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
        },
        participant,
    },
    topic_definition::type_support::DdsDeserialize,
};

use super::{
    discovery_types::ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
    error::{RtpsError, RtpsErrorKind, RtpsResult},
    messages::overall_structure::RtpsMessageRead,
    participant::RtpsParticipant,
    reader::{ReaderCacheChange, ReaderHistoryCache, TransportReader},
    stateful_writer::WriterHistoryCache,
    types::{
        EntityId, Guid, GuidPrefix, Locator, TopicKind, ENTITYID_PARTICIPANT, LOCATOR_KIND_UDP_V4,
        USER_DEFINED_READER_NO_KEY, USER_DEFINED_READER_WITH_KEY, USER_DEFINED_WRITER_NO_KEY,
        USER_DEFINED_WRITER_WITH_KEY,
    },
};

pub trait Transport: Send + Sync {
    fn guid(&self) -> [u8; 16];

    fn get_participant_discovery_writer(&self) -> Box<dyn WriterHistoryCache>;

    fn get_participant_discovery_reader(&self) -> Box<dyn TransportReader>;

    fn get_topics_discovery_writer(&self) -> Box<dyn WriterHistoryCache>;

    fn get_topics_discovery_reader(&self) -> Box<dyn TransportReader>;

    fn get_publications_discovery_writer(&self) -> Box<dyn WriterHistoryCache>;

    fn get_publications_discovery_reader(&self) -> Box<dyn TransportReader>;

    fn get_subscriptions_discovery_writer(&self) -> Box<dyn WriterHistoryCache>;

    fn get_subscriptions_discovery_reader(&self) -> Box<dyn TransportReader>;

    fn create_user_defined_reader(
        &mut self,
        topic_name: &str,
        topic_kind: TopicKind,
        reader_history_cache: Box<dyn ReaderHistoryCache>,
    ) -> Box<dyn TransportReader>;

    fn create_user_defined_writer(
        &mut self,
        topic_name: &str,
        topic_kind: TopicKind,
    ) -> Box<dyn WriterHistoryCache>;
}

const MAX_DATAGRAM_SIZE: usize = 65507;

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
    #[cfg(target_family = "unix")]
    socket.set_reuse_port(true)?;
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

pub fn read_message(
    socket: &mut std::net::UdpSocket,
    buf: &mut [u8],
) -> RtpsResult<RtpsMessageRead> {
    let (bytes, _) = socket.recv_from(buf)?;
    if bytes > 0 {
        Ok(RtpsMessageRead::try_from(&buf[0..bytes])?)
    } else {
        Err(RtpsError::new(RtpsErrorKind::NotEnoughData, ""))
    }
}

pub struct RtpsTransport {
    guid: Guid,
    rtps_participant: Actor<RtpsParticipant>,
    _executor: Executor,
    endpoint_counter: u8,
}

impl RtpsTransport {
    pub fn new(
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        domain_tag: String,
        interface_name: Option<&String>,
        udp_receive_buffer_size: Option<usize>,
        dcps_participant_reader_history_cache: Box<dyn ReaderHistoryCache>,
        dcps_topics_reader_history_cache: Box<dyn ReaderHistoryCache>,
        dcps_publications_reader_history_cache: Box<dyn ReaderHistoryCache>,
        dcps_subscriptions_reader_history_cache: Box<dyn ReaderHistoryCache>,
    ) -> RtpsResult<Self> {
        let executor = Executor::new();

        // Open socket for unicast user-defined data
        let interface_address_list = NetworkInterface::show()
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
                    Addr::V4(_) => true,
                    _ => false,
                })
            });

        let default_unicast_socket =
            socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None)?;
        default_unicast_socket.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)).into())?;
        default_unicast_socket.set_nonblocking(false)?;
        if let Some(buffer_size) = udp_receive_buffer_size {
            default_unicast_socket.set_recv_buffer_size(buffer_size)?;
        }

        let mut default_unicast_socket = std::net::UdpSocket::from(default_unicast_socket);
        let user_defined_unicast_port = default_unicast_socket.local_addr()?.port().into();
        let default_unicast_locator_list = interface_address_list
            .clone()
            .map(|a| Locator::from_ip_and_port(&a, user_defined_unicast_port))
            .collect();

        let default_multicast_locator_list = vec![];

        // Open socket for unicast metatraffic data
        let mut metatraffic_unicast_socket =
            std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))?;
        metatraffic_unicast_socket.set_nonblocking(false)?;
        let metattrafic_unicast_locator_port =
            metatraffic_unicast_socket.local_addr()?.port().into();
        let metatraffic_unicast_locator_list: Vec<Locator> = interface_address_list
            .clone()
            .map(|a| Locator::from_ip_and_port(&a, metattrafic_unicast_locator_port))
            .collect();

        // Open socket for multicast metatraffic data
        let metatraffic_multicast_locator_list = vec![Locator::new(
            LOCATOR_KIND_UDP_V4,
            port_builtin_multicast(domain_id) as u32,
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
        )];

        let mut metatraffic_multicast_socket = get_multicast_socket(
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
            port_builtin_multicast(domain_id),
            interface_address_list,
        )?;

        let rtps_participant_actor_builder = ActorBuilder::new();

        let participant_discovery_reader_history_cache =
            Box::new(ParticipantDiscoveryReaderHistoryCache {
                rtps_participant_address: rtps_participant_actor_builder.address(),
                dcps_participant_reader_history_cache,
            });

        let publications_discovery_reader_history_cache =
            Box::new(PublicationsDiscoveryReaderHistoryCache {
                rtps_participant_address: rtps_participant_actor_builder.address(),
                dcps_publications_reader_history_cache,
            });

        let subscriptions_discovery_reader_history_cache =
            Box::new(SubscriptionsDiscoveryReaderHistoryCache {
                rtps_participant_address: rtps_participant_actor_builder.address(),
                dcps_subscriptions_reader_history_cache,
            });
        let guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);
        let rtps_participant = rtps_participant_actor_builder.build(
            RtpsParticipant::new(
                guid,
                domain_id,
                domain_tag,
                default_unicast_locator_list,
                default_multicast_locator_list,
                metatraffic_unicast_locator_list,
                metatraffic_multicast_locator_list,
                participant_discovery_reader_history_cache,
                dcps_topics_reader_history_cache,
                publications_discovery_reader_history_cache,
                subscriptions_discovery_reader_history_cache,
            )?,
            &executor.handle(),
        );

        let rtps_participant_address = rtps_participant.address();
        std::thread::Builder::new()
            .name("RTPS metatraffic multicast discovery".to_string())
            .spawn(move || {
                let mut buf = Box::new([0; MAX_DATAGRAM_SIZE]);
                loop {
                    if let Ok(rtps_message) =
                        read_message(&mut metatraffic_multicast_socket, buf.as_mut_slice())
                    {
                        tracing::trace!(
                            rtps_message = ?rtps_message,
                            "Received metatraffic multicast RTPS message"
                        );
                        let r = rtps_participant_address.send_actor_mail(
                            participant::ProcessBuiltinRtpsMessage { rtps_message },
                        );
                        if r.is_err() {
                            break;
                        }
                    }
                }
            })
            .expect("failed to spawn thread");

        let rtps_participant_address = rtps_participant.address();
        std::thread::Builder::new()
            .name("RTPS metatraffic unicast discovery".to_string())
            .spawn(move || {
                let mut buf = Box::new([0; MAX_DATAGRAM_SIZE]);
                loop {
                    if let Ok(rtps_message) =
                        read_message(&mut metatraffic_unicast_socket, buf.as_mut_slice())
                    {
                        tracing::trace!(
                            rtps_message = ?rtps_message,
                            "Received metatraffic unicast RTPS message"
                        );

                        let r = rtps_participant_address.send_actor_mail(
                            participant::ProcessBuiltinRtpsMessage { rtps_message },
                        );
                        if r.is_err() {
                            break;
                        }
                    }
                }
            })
            .expect("failed to spawn thread");

        let rtps_participant_address = rtps_participant.address();
        std::thread::Builder::new()
            .name("RTPS user defined traffic".to_string())
            .spawn(move || {
                let mut buf = Box::new([0; MAX_DATAGRAM_SIZE]);
                loop {
                    if let Ok(rtps_message) =
                        read_message(&mut default_unicast_socket, buf.as_mut_slice())
                    {
                        tracing::trace!(
                            rtps_message = ?rtps_message,
                            "Received user defined data unicast RTPS message"
                        );
                        let r = rtps_participant_address.send_actor_mail(
                            participant::ProcessUserDefinedRtpsMessage { rtps_message },
                        );
                        if r.is_err() {
                            break;
                        }
                    }
                }
            })
            .expect("failed to spawn thread");

        // Heartbeat thread
        let rtps_participant_address = rtps_participant.address();
        std::thread::Builder::new()
            .name("RTPS heartbeat".to_string())
            .spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(50));
                let r = rtps_participant_address.send_actor_mail(participant::SendHeartbeat);
                if r.is_err() {
                    break;
                }
            })
            .expect("failed to spawn thread");

        Ok(Self {
            guid,
            rtps_participant,
            _executor: executor,
            endpoint_counter: 0,
        })
    }
}

impl Transport for RtpsTransport {
    fn guid(&self) -> [u8; 16] {
        self.guid.into()
    }

    fn get_participant_discovery_writer(&self) -> Box<dyn WriterHistoryCache> {
        struct RtpsParticipantDiscoveryWriterHistoryCache {
            guid: Guid,
            rtps_participant_address: ActorAddress<RtpsParticipant>,
        }
        impl WriterHistoryCache for RtpsParticipantDiscoveryWriterHistoryCache {
            fn guid(&self) -> [u8; 16] {
                self.guid.into()
            }

            fn add_change(&mut self, cache_change: crate::rtps::cache_change::RtpsCacheChange) {
                self.rtps_participant_address
                    .send_actor_mail(participant::AddParticipantDiscoveryCacheChange {
                        cache_change,
                    })
                    .ok();
            }

            fn remove_change(&mut self, sequence_number: crate::rtps::types::SequenceNumber) {
                self.rtps_participant_address
                    .send_actor_mail(participant::RemoveParticipantDiscoveryCacheChange {
                        sequence_number,
                    })
                    .ok();
            }

            fn are_all_changes_acknowledged(&self) -> bool {
                todo!()
            }
        }

        Box::new(RtpsParticipantDiscoveryWriterHistoryCache {
            guid: Guid::new(self.guid.prefix(), ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER),
            rtps_participant_address: self.rtps_participant.address(),
        })
    }

    fn get_participant_discovery_reader(&self) -> Box<dyn TransportReader> {
        pub struct RtpsParticipantDiscoveryReader {
            guid: Guid,
        }

        impl TransportReader for RtpsParticipantDiscoveryReader {
            fn guid(&self) -> [u8; 16] {
                self.guid.into()
            }

            fn is_historical_data_received(&self) -> bool {
                true
            }
        }

        Box::new(RtpsParticipantDiscoveryReader {
            guid: Guid::new(self.guid.prefix(), ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER),
        })
    }

    fn get_topics_discovery_writer(&self) -> Box<dyn WriterHistoryCache> {
        struct RtpsTopicsDiscoveryWriterHistoryCache {
            guid: Guid,
            rtps_participant_address: ActorAddress<RtpsParticipant>,
        }

        impl WriterHistoryCache for RtpsTopicsDiscoveryWriterHistoryCache {
            fn guid(&self) -> [u8; 16] {
                self.guid.into()
            }

            fn add_change(&mut self, cache_change: crate::rtps::cache_change::RtpsCacheChange) {
                self.rtps_participant_address
                    .send_actor_mail(participant::AddTopicsDiscoveryCacheChange { cache_change })
                    .ok();
            }

            fn remove_change(&mut self, sequence_number: crate::rtps::types::SequenceNumber) {
                self.rtps_participant_address
                    .send_actor_mail(participant::RemoveTopicsDiscoveryCacheChange {
                        sequence_number,
                    })
                    .ok();
            }

            fn are_all_changes_acknowledged(&self) -> bool {
                todo!()
            }
        }

        Box::new(RtpsTopicsDiscoveryWriterHistoryCache {
            guid: Guid::new(self.guid.prefix(), ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER),
            rtps_participant_address: self.rtps_participant.address(),
        })
    }

    fn get_topics_discovery_reader(&self) -> Box<dyn TransportReader> {
        pub struct RtpsTopicsDiscoveryReader {
            guid: Guid,
        }

        impl TransportReader for RtpsTopicsDiscoveryReader {
            fn guid(&self) -> [u8; 16] {
                self.guid.into()
            }

            fn is_historical_data_received(&self) -> bool {
                true
            }
        }

        Box::new(RtpsTopicsDiscoveryReader {
            guid: Guid::new(self.guid.prefix(), ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR),
        })
    }

    fn get_publications_discovery_writer(&self) -> Box<dyn WriterHistoryCache> {
        struct RtpsPublicationsDiscoveryWriterHistoryCache {
            guid: Guid,
            rtps_participant_address: ActorAddress<RtpsParticipant>,
        }

        impl WriterHistoryCache for RtpsPublicationsDiscoveryWriterHistoryCache {
            fn guid(&self) -> [u8; 16] {
                self.guid.into()
            }

            fn add_change(&mut self, cache_change: crate::rtps::cache_change::RtpsCacheChange) {
                self.rtps_participant_address
                    .send_actor_mail(participant::AddPublicationsDiscoveryCacheChange {
                        cache_change,
                    })
                    .ok();
            }

            fn remove_change(&mut self, sequence_number: crate::rtps::types::SequenceNumber) {
                self.rtps_participant_address
                    .send_actor_mail(participant::RemovePublicationsDiscoveryCacheChange {
                        sequence_number,
                    })
                    .ok();
            }

            fn are_all_changes_acknowledged(&self) -> bool {
                todo!()
            }
        }

        Box::new(RtpsPublicationsDiscoveryWriterHistoryCache {
            guid: Guid::new(
                self.guid.prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ),
            rtps_participant_address: self.rtps_participant.address(),
        })
    }

    fn get_publications_discovery_reader(&self) -> Box<dyn TransportReader> {
        pub struct RtpsPublicationsDiscoveryReader {
            guid: Guid,
        }

        impl TransportReader for RtpsPublicationsDiscoveryReader {
            fn guid(&self) -> [u8; 16] {
                self.guid.into()
            }

            fn is_historical_data_received(&self) -> bool {
                true
            }
        }

        Box::new(RtpsPublicationsDiscoveryReader {
            guid: Guid::new(
                self.guid.prefix(),
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ),
        })
    }

    fn get_subscriptions_discovery_writer(&self) -> Box<dyn WriterHistoryCache> {
        struct RtpsSubscriptionsDiscoveryWriterHistoryCache {
            guid: Guid,
            rtps_participant_address: ActorAddress<RtpsParticipant>,
        }

        impl WriterHistoryCache for RtpsSubscriptionsDiscoveryWriterHistoryCache {
            fn guid(&self) -> [u8; 16] {
                self.guid.into()
            }

            fn add_change(&mut self, cache_change: crate::rtps::cache_change::RtpsCacheChange) {
                self.rtps_participant_address
                    .send_actor_mail(participant::AddSubscriptionsDiscoveryCacheChange {
                        cache_change,
                    })
                    .ok();
            }

            fn remove_change(&mut self, sequence_number: crate::rtps::types::SequenceNumber) {
                self.rtps_participant_address
                    .send_actor_mail(participant::RemoveSubscriptionsDiscoveryCacheChange {
                        sequence_number,
                    })
                    .ok();
            }

            fn are_all_changes_acknowledged(&self) -> bool {
                todo!()
            }
        }

        Box::new(RtpsSubscriptionsDiscoveryWriterHistoryCache {
            guid: Guid::new(
                self.guid.prefix(),
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ),
            rtps_participant_address: self.rtps_participant.address(),
        })
    }

    fn get_subscriptions_discovery_reader(&self) -> Box<dyn TransportReader> {
        pub struct RtpsSubscriptionsDiscoveryReader {
            guid: Guid,
        }

        impl TransportReader for RtpsSubscriptionsDiscoveryReader {
            fn guid(&self) -> [u8; 16] {
                self.guid.into()
            }

            fn is_historical_data_received(&self) -> bool {
                true
            }
        }

        Box::new(RtpsSubscriptionsDiscoveryReader {
            guid: Guid::new(
                self.guid.prefix(),
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            ),
        })
    }

    fn create_user_defined_reader(
        &mut self,
        topic_name: &str,
        topic_kind: TopicKind,
        reader_history_cache: Box<dyn ReaderHistoryCache>,
    ) -> Box<dyn TransportReader> {
        struct UserDefinedTransportReader {
            rtps_participant_address: ActorAddress<RtpsParticipant>,
            guid: Guid,
        }

        impl TransportReader for UserDefinedTransportReader {
            fn guid(&self) -> [u8; 16] {
                self.guid.into()
            }

            fn is_historical_data_received(&self) -> bool {
                if let Ok(r) = self
                    .rtps_participant_address
                    .send_actor_mail(participant::IsHistoricalDataReceived { guid: self.guid })
                {
                    block_on(r.receive_reply())
                } else {
                    false
                }
            }
        }

        self.endpoint_counter += 1;
        let entity_kind = match topic_kind {
            TopicKind::WithKey => USER_DEFINED_READER_WITH_KEY,
            TopicKind::NoKey => USER_DEFINED_READER_NO_KEY,
        };

        let data_reader_counter = self.endpoint_counter;
        let entity_key: [u8; 3] = [0, data_reader_counter, 0];
        let entity_id = EntityId::new(entity_key, entity_kind);
        let reader_guid = Guid::new(self.guid.prefix(), entity_id);

        self.rtps_participant
            .send_actor_mail(participant::CreateReader {
                reader_guid,
                topic_name: topic_name.to_owned(),
                reader_history_cache,
            });

        Box::new(UserDefinedTransportReader {
            rtps_participant_address: self.rtps_participant.address(),
            guid: reader_guid,
        })
    }

    fn create_user_defined_writer(
        &mut self,
        topic_name: &str,
        topic_kind: TopicKind,
    ) -> Box<dyn WriterHistoryCache> {
        let entity_kind = match topic_kind {
            TopicKind::WithKey => USER_DEFINED_WRITER_WITH_KEY,
            TopicKind::NoKey => USER_DEFINED_WRITER_NO_KEY,
        };
        self.endpoint_counter += 1;
        let data_writer_counter = self.endpoint_counter;
        let entity_key: [u8; 3] = [0, 0, data_writer_counter];
        let entity_id = EntityId::new(entity_key, entity_kind);
        let writer_guid = Guid::new(self.guid.prefix(), entity_id);

        block_on(
            self.rtps_participant
                .send_actor_mail(participant::CreateWriter {
                    writer_guid,
                    topic_name: topic_name.to_owned(),
                    rtps_participant_address: self.rtps_participant.address(),
                })
                .receive_reply(),
        )
    }
}

struct ParticipantDiscoveryReaderHistoryCache {
    rtps_participant_address: ActorAddress<RtpsParticipant>,
    dcps_participant_reader_history_cache: Box<dyn ReaderHistoryCache>,
}

impl ReaderHistoryCache for ParticipantDiscoveryReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        if let Ok(discovered_participant_data) =
            SpdpDiscoveredParticipantData::deserialize_data(cache_change.data_value.as_ref())
        {
            self.rtps_participant_address
                .send_actor_mail(participant::AddDiscoveredParticipant {
                    discovered_participant_data,
                })
                .unwrap();
        }
        self.dcps_participant_reader_history_cache
            .add_change(cache_change);
    }
}

struct PublicationsDiscoveryReaderHistoryCache {
    rtps_participant_address: ActorAddress<RtpsParticipant>,
    dcps_publications_reader_history_cache: Box<dyn ReaderHistoryCache>,
}

impl ReaderHistoryCache for PublicationsDiscoveryReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        if let Ok(discovered_writer_data) =
            DiscoveredWriterData::deserialize_data(cache_change.data_value.as_ref())
        {
            self.rtps_participant_address
                .send_actor_mail(participant::AddDiscoveredWriter {
                    discovered_writer_data,
                })
                .unwrap();
        }
        self.dcps_publications_reader_history_cache
            .add_change(cache_change);
    }
}

struct SubscriptionsDiscoveryReaderHistoryCache {
    rtps_participant_address: ActorAddress<RtpsParticipant>,
    dcps_subscriptions_reader_history_cache: Box<dyn ReaderHistoryCache>,
}

impl ReaderHistoryCache for SubscriptionsDiscoveryReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        if let Ok(discovered_reader_data) =
            DiscoveredReaderData::deserialize_data(cache_change.data_value.as_ref())
        {
            self.rtps_participant_address
                .send_actor_mail(participant::AddDiscoveredReader {
                    discovered_reader_data,
                })
                .unwrap();
        }
        self.dcps_subscriptions_reader_history_cache
            .add_change(cache_change);
    }
}
