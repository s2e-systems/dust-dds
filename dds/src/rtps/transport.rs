use core::net::{Ipv4Addr, SocketAddr};

use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use tracing::info;

use crate::{
    domain::domain_participant_factory::DomainId,
    implementation::{
        actor::{Actor, ActorAddress, ActorBuilder},
        data_representation_builtin_endpoints::spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        runtime::executor::{block_on, Executor, ExecutorHandle},
    },
    rtps::{participant, types::ChangeKind},
    topic_definition::type_support::DdsDeserialize,
};

use super::{
    discovery_types::{
        ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
        ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
    },
    error::{RtpsError, RtpsErrorKind, RtpsResult},
    messages::overall_structure::RtpsMessageRead,
    participant::RtpsParticipant,
    reader::{ReaderCacheChange, ReaderHistoryCache, TransportReader},
    stateful_writer::{TransportWriter, WriterHistoryCache},
    types::{Guid, GuidPrefix, Locator, TopicKind, LOCATOR_KIND_UDP_V4},
};

pub trait Transport: Send + Sync {
    fn get_participant_discovery_writer(&self) -> Box<dyn TransportWriter>;

    fn get_topics_discovery_writer(&self) -> Box<dyn TransportWriter>;

    fn create_user_defined_reader(
        &mut self,
        topic_kind: TopicKind,
        reader_history_cache: Box<dyn ReaderHistoryCache>,
    ) -> Box<dyn TransportReader>;

    fn create_user_defined_writer(&mut self, topic_kind: TopicKind) -> Box<dyn TransportWriter>;
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
    rtps_participant: Actor<RtpsParticipant>,
    executor: Executor,
}

impl RtpsTransport {
    pub fn new(
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        domain_tag: String,
        interface_name: Option<&String>,
        udp_receive_buffer_size: Option<usize>,
        dcps_participant_reader_history_cache: Box<dyn ReaderHistoryCache>,
        sedp_builtin_topics_reader_history_cache: Box<dyn ReaderHistoryCache>,
        sedp_builtin_publications_reader_history_cache: Box<dyn ReaderHistoryCache>,
        sedp_builtin_subscriptions_reader_history_cache: Box<dyn ReaderHistoryCache>,
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
        let rtps_participant = rtps_participant_actor_builder.build(
            RtpsParticipant::new(
                guid_prefix,
                domain_id,
                domain_tag,
                default_unicast_locator_list,
                default_multicast_locator_list,
                metatraffic_unicast_locator_list,
                metatraffic_multicast_locator_list,
                participant_discovery_reader_history_cache,
                sedp_builtin_topics_reader_history_cache,
                sedp_builtin_publications_reader_history_cache,
                sedp_builtin_subscriptions_reader_history_cache,
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
            rtps_participant,
            executor,
        })
    }
}

pub struct RtpsSepdDiscovery {}

pub struct RtpsSpdpDiscovery {}

impl RtpsSpdpDiscovery {
    pub fn new() -> Self {
        // let spdp_builtin_participant_writer = RtpsStatelessWriter::new(
        //     Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER),
        //     message_sender,
        //     self.participant_proxy(),
        // );

        // let spdp_discovery_locator_list = [Locator::new(
        //     LOCATOR_KIND_UDP_V4,
        //     port_builtin_multicast(self.domain_id) as u32,
        //     DEFAULT_MULTICAST_LOCATOR_ADDRESS,
        // )];

        // for reader_locator in spdp_discovery_locator_list {
        //     stateless_writer.reader_locator_add(reader_locator);
        // }
        // let spdp_builtin_participant_reader = RtpsStatelessReader::new(
        //     Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER),
        //     spdp_builtin_participant_reader_history_cache,
        // );
        // Self {
        //     spdp_builtin_participant_writer,
        //     spdp_builtin_participant_reader,
        // }
        todo!()
    }
}

impl Transport for RtpsTransport {
    fn get_participant_discovery_writer(&self) -> Box<dyn TransportWriter> {
        struct RtpsParticipantDiscoveryWriterHistoryCache {
            pub rtps_participant_address: ActorAddress<RtpsParticipant>,
        }
        impl TransportWriter for RtpsParticipantDiscoveryWriterHistoryCache {
            fn get_history_cache(
                &mut self,
            ) -> &mut dyn crate::rtps::stateful_writer::WriterHistoryCache {
                self
            }

            fn add_matched_reader(
                &mut self,
                reader_proxy: crate::implementation::data_representation_builtin_endpoints::discovered_reader_data::ReaderProxy,
                reliability_kind: crate::rtps::types::ReliabilityKind,
                durability_kind: crate::rtps::types::DurabilityKind,
            ) {
                todo!()
            }

            fn delete_matched_reader(&mut self, reader_guid: crate::rtps::types::Guid) {
                todo!()
            }

            fn are_all_changes_acknowledged(&self) -> bool {
                todo!()
            }

            fn writer_proxy(&self) -> crate::implementation::data_representation_builtin_endpoints::discovered_writer_data::WriterProxy{
                todo!()
            }
        }
        impl WriterHistoryCache for RtpsParticipantDiscoveryWriterHistoryCache {
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
        }

        Box::new(RtpsParticipantDiscoveryWriterHistoryCache {
            rtps_participant_address: self.rtps_participant.address(),
        })
    }

    fn get_topics_discovery_writer(&self) -> Box<dyn TransportWriter> {
        struct RtpsTopicsDiscoveryWriterHistoryCache {
            pub rtps_participant_address: ActorAddress<RtpsParticipant>,
        }
        impl TransportWriter for RtpsTopicsDiscoveryWriterHistoryCache {
            fn get_history_cache(
                &mut self,
            ) -> &mut dyn crate::rtps::stateful_writer::WriterHistoryCache {
                self
            }

            fn add_matched_reader(
                &mut self,
                reader_proxy: crate::implementation::data_representation_builtin_endpoints::discovered_reader_data::ReaderProxy,
                reliability_kind: crate::rtps::types::ReliabilityKind,
                durability_kind: crate::rtps::types::DurabilityKind,
            ) {
                todo!()
            }

            fn delete_matched_reader(&mut self, reader_guid: crate::rtps::types::Guid) {
                todo!()
            }

            fn are_all_changes_acknowledged(&self) -> bool {
                todo!()
            }

            fn writer_proxy(&self) -> crate::implementation::data_representation_builtin_endpoints::discovered_writer_data::WriterProxy{
                todo!()
            }
        }
        impl WriterHistoryCache for RtpsTopicsDiscoveryWriterHistoryCache {
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
        }

        Box::new(RtpsTopicsDiscoveryWriterHistoryCache {
            rtps_participant_address: self.rtps_participant.address(),
        })
    }

    // fn create_topics_discovery_reader(
    //     &mut self,
    //     reader_history_cache: Box<dyn ReaderHistoryCache>,
    // ) -> Box<dyn TransportReader> {
    //     todo!()
    //     // struct SedpTopicsDiscoveryReaderHistoryCache {
    //     //     reader_history_cache: Box<dyn ReaderHistoryCache>,
    //     // }
    //     // impl ReaderHistoryCache for SedpTopicsDiscoveryReaderHistoryCache {
    //     //     fn add_change(&mut self, cache_change: ReaderCacheChange) {
    //     //         self.reader_history_cache.add_change(cache_change);
    //     //     }
    //     // }
    //     // let reader_guid = Guid::new(
    //     //     self.rtps_participant.lock().unwrap().guid().prefix(),
    //     //     ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
    //     // );
    //     // let history_cache = Box::new(SedpTopicsDiscoveryReaderHistoryCache {
    //     //     reader_history_cache,
    //     // });
    //     // self.rtps_participant
    //     //     .lock()
    //     //     .unwrap()
    //     //     .create_builtin_stateless_reader(reader_guid, history_cache)
    // }

    // fn create_topics_discovery_writer(&mut self) -> Box<dyn TransportWriter> {
    //     todo!()
    //     // let writer_guid = Guid::new(
    //     //     self.rtps_participant.lock().unwrap().guid().prefix(),
    //     //     ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
    //     // );
    //     // self.rtps_participant
    //     //     .lock()
    //     //     .unwrap()
    //     //     .create_builtin_stateful_writer(writer_guid)
    // }

    // fn create_publications_discovery_reader(
    //     &mut self,
    //     reader_history_cache: Box<dyn ReaderHistoryCache>,
    // ) -> Box<dyn TransportReader> {
    //     todo!()
    //     // struct SedpPublicationsDiscoveryReaderHistoryCache {
    //     //     reader_history_cache: Box<dyn ReaderHistoryCache >,
    //     // }
    //     // impl ReaderHistoryCache for SedpPublicationsDiscoveryReaderHistoryCache {
    //     //     fn add_change(&mut self, cache_change: ReaderCacheChange) {
    //     //         self.reader_history_cache.add_change(cache_change);
    //     //     }
    //     // }
    //     // let reader_guid = Guid::new(
    //     //     self.rtps_participant.lock().unwrap().guid().prefix(),
    //     //     ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
    //     // );
    //     // let history_cache = Box::new(SedpPublicationsDiscoveryReaderHistoryCache {
    //     //     reader_history_cache,
    //     // });
    //     // self.rtps_participant
    //     //     .lock()
    //     //     .unwrap()
    //     //     .create_builtin_stateless_reader(reader_guid, history_cache)
    // }

    // fn create_publications_discovery_writer(&mut self) -> Box<dyn TransportWriter> {
    //     todo!()
    //     // let writer_guid = Guid::new(
    //     //     self.rtps_participant.lock().unwrap().guid().prefix(),
    //     //     ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
    //     // );
    //     // self.rtps_participant
    //     //     .lock()
    //     //     .unwrap()
    //     //     .create_builtin_stateful_writer(writer_guid)
    // }

    // fn create_subscriptions_discovery_reader(
    //     &mut self,
    //     reader_history_cache: Box<dyn ReaderHistoryCache>,
    // ) -> Box<dyn TransportReader> {
    //     todo!()
    //     // struct SedpSubscriptionsDiscoveryReaderHistoryCache {
    //     //     reader_history_cache: Box<dyn ReaderHistoryCache>,
    //     // }
    //     // impl ReaderHistoryCache for SedpSubscriptionsDiscoveryReaderHistoryCache {
    //     //     fn add_change(&mut self, cache_change: ReaderCacheChange) {
    //     //         self.reader_history_cache.add_change(cache_change);
    //     //     }
    //     // }
    //     // let reader_guid = Guid::new(
    //     //     self.rtps_participant.lock().unwrap().guid().prefix(),
    //     //     ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
    //     // );
    //     // let history_cache = Box::new(SedpSubscriptionsDiscoveryReaderHistoryCache {
    //     //     reader_history_cache,
    //     // });
    //     // self.rtps_participant
    //     //     .lock()
    //     //     .unwrap()
    //     //     .create_builtin_stateless_reader(reader_guid, history_cache)
    // }

    // fn create_subscriptions_discovery_writer(&mut self) -> Box<dyn TransportWriter> {
    //     todo!()
    //     // let writer_guid = Guid::new(
    //     //     self.rtps_participant.lock().unwrap().guid().prefix(),
    //     //     ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
    //     // );
    //     // self.rtps_participant
    //     //     .lock()
    //     //     .unwrap()
    //     //     .create_builtin_stateful_writer(writer_guid)
    // }

    fn create_user_defined_reader(
        &mut self,
        topic_kind: TopicKind,
        reader_history_cache: Box<dyn ReaderHistoryCache>,
    ) -> Box<dyn TransportReader> {
        block_on(
            self.rtps_participant
                .send_actor_mail(participant::CreateReader {
                    topic_kind,
                    reader_history_cache,
                })
                .receive_reply(),
        )
    }

    fn create_user_defined_writer(&mut self, topic_kind: TopicKind) -> Box<dyn TransportWriter> {
        block_on(
            self.rtps_participant
                .send_actor_mail(participant::CreateWriter {
                    topic_kind,
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
