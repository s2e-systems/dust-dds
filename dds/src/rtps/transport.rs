use core::net::{Ipv4Addr, SocketAddr};

use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use tracing::info;

use crate::{
    domain::domain_participant_factory::DomainId,
    rtps::participant,
    runtime::{
        actor::{Actor, ActorAddress, ActorBuilder},
        executor::{block_on, Executor},
    },
    transport::{
        history_cache::HistoryCache,
        participant::TransportParticipant,
        reader::{TransportStatefulReader, TransportStatelessReader, WriterProxy},
        types::{
            EntityId, Guid, GuidPrefix, Locator, ProtocolVersion, ReliabilityKind, VendorId,
            ENTITYID_PARTICIPANT, LOCATOR_KIND_UDP_V4,
        },
        writer::{TransportStatefulWriter, TransportStatelessWriter},
    },
};

use super::{
    error::{RtpsError, RtpsErrorKind, RtpsResult},
    messages::overall_structure::RtpsMessageRead,
    participant::RtpsParticipant,
    types::{PROTOCOLVERSION, VENDOR_ID_S2E},
};

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
    fragment_size: usize,
    rtps_participant: Actor<RtpsParticipant>,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    _executor: Executor,
}

impl RtpsTransport {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        interface_name: Option<&str>,
        udp_receive_buffer_size: Option<usize>,
        fragment_size: usize,
    ) -> RtpsResult<Self> {
        let executor = Executor::new();

        // Open socket for unicast user-defined data
        let interface_address_list = NetworkInterface::show()
            .expect("Could not scan interfaces")
            .into_iter()
            .filter(|x| {
                if let Some(if_name) = interface_name {
                    x.name == if_name
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
        let default_unicast_locator_list: Vec<_> = interface_address_list
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

        let guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);
        let rtps_participant = rtps_participant_actor_builder.build(
            RtpsParticipant::new(
                guid,
                default_unicast_locator_list.clone(),
                default_multicast_locator_list.clone(),
                metatraffic_unicast_locator_list.clone(),
                metatraffic_multicast_locator_list.clone(),
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
            default_unicast_locator_list,
            default_multicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            fragment_size,
            _executor: executor,
        })
    }
}

impl TransportParticipant for RtpsTransport {
    fn guid(&self) -> Guid {
        self.guid
    }

    fn protocol_version(&self) -> ProtocolVersion {
        PROTOCOLVERSION
    }

    fn vendor_id(&self) -> VendorId {
        VENDOR_ID_S2E
    }

    fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        &self.metatraffic_unicast_locator_list
    }

    fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        &self.metatraffic_multicast_locator_list
    }

    fn default_unicast_locator_list(&self) -> &[Locator] {
        &self.default_unicast_locator_list
    }

    fn default_multicast_locator_list(&self) -> &[Locator] {
        &self.default_multicast_locator_list
    }

    fn create_stateless_reader(
        &mut self,
        entity_id: EntityId,
        reader_history_cache: Box<dyn HistoryCache>,
    ) -> Box<dyn TransportStatelessReader> {
        let guid = Guid::new(self.guid.prefix(), entity_id);
        struct StatelessReader {
            guid: Guid,
        }

        impl TransportStatelessReader for StatelessReader {
            fn guid(&self) -> Guid {
                self.guid
            }
        }

        self.rtps_participant
            .send_actor_mail(participant::CreateStatelessReader {
                reader_guid: guid,
                reader_history_cache,
            });

        Box::new(StatelessReader { guid })
    }

    fn create_stateless_writer(
        &mut self,
        entity_id: EntityId,
    ) -> Box<dyn TransportStatelessWriter> {
        let guid = Guid::new(self.guid.prefix(), entity_id);
        block_on(
            self.rtps_participant
                .send_actor_mail(participant::CreateStatelessWriter {
                    writer_guid: guid,
                    rtps_participant_address: self.rtps_participant.address(),
                })
                .receive_reply(),
        )
    }

    fn create_stateful_reader(
        &mut self,
        entity_id: EntityId,
        _reliability_kind: ReliabilityKind,
        reader_history_cache: Box<dyn HistoryCache>,
    ) -> Box<dyn TransportStatefulReader> {
        let guid = Guid::new(self.guid.prefix(), entity_id);
        struct StatefulReader {
            rtps_participant_address: ActorAddress<RtpsParticipant>,
            guid: Guid,
        }

        impl TransportStatefulReader for StatefulReader {
            fn guid(&self) -> Guid {
                self.guid
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

            fn add_matched_writer(&mut self, writer_proxy: WriterProxy) {
                block_on(
                    self.rtps_participant_address
                        .send_actor_mail(participant::AddMatchedWriter {
                            reader: self.guid,
                            writer_proxy,
                        })
                        .expect("Actor must exist")
                        .receive_reply(),
                )
            }

            fn remove_matched_writer(&mut self, _remote_writer_guid: Guid) {
                todo!()
            }
        }

        self.rtps_participant
            .send_actor_mail(participant::CreateStatefulReader {
                reader_guid: guid,
                reader_history_cache,
            });

        Box::new(StatefulReader {
            rtps_participant_address: self.rtps_participant.address(),
            guid,
        })
    }

    fn create_stateful_writer(
        &mut self,
        entity_id: EntityId,
        _reliability_kind: ReliabilityKind,
    ) -> Box<dyn TransportStatefulWriter> {
        let guid = Guid::new(self.guid.prefix(), entity_id);
        block_on(
            self.rtps_participant
                .send_actor_mail(participant::CreateStatefulWriter {
                    writer_guid: guid,
                    data_max_size_serialized: self.fragment_size,
                    rtps_participant_address: self.rtps_participant.address(),
                })
                .receive_reply(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{sync_channel, SyncSender};

    use crate::transport::{
        history_cache::CacheChange,
        types::{ChangeKind, DurabilityKind, ENTITYID_UNKNOWN},
        writer::ReaderProxy,
    };

    use super::*;

    #[test]
    fn basic_transport_stateful_reader_writer_usage() {
        let guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let domain_id = 0;
        let interface_name = None;
        let udp_receive_buffer_size = None;
        let mut transport = RtpsTransport::new(
            guid_prefix,
            domain_id,
            interface_name,
            udp_receive_buffer_size,
            1344,
        )
        .unwrap();

        struct MockHistoryCache(SyncSender<CacheChange>);

        impl HistoryCache for MockHistoryCache {
            fn add_change(&mut self, cache_change: CacheChange) {
                self.0.send(cache_change).unwrap();
            }

            fn remove_change(&mut self, _sequence_number: i64) {
                todo!()
            }
        }

        let entity_id = EntityId::new([1, 2, 3], 4);
        let reliability_kind = ReliabilityKind::Reliable;
        let (sender, receiver) = sync_channel(0);
        let reader_history_cache = Box::new(MockHistoryCache(sender));
        let mut reader =
            transport.create_stateful_reader(entity_id, reliability_kind, reader_history_cache);

        let entity_id = EntityId::new([5, 6, 7], 8);
        let mut writer = transport.create_stateful_writer(entity_id, reliability_kind);

        let reader_proxy = ReaderProxy {
            remote_reader_guid: reader.guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            reliability_kind,
            durability_kind: DurabilityKind::Volatile,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        };
        writer.add_matched_reader(reader_proxy);

        let writer_proxy = WriterProxy {
            remote_writer_guid: writer.guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            reliability_kind,
            durability_kind: DurabilityKind::Volatile,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
        };
        reader.add_matched_writer(writer_proxy);

        let cache_change = CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: writer.guid(),
            sequence_number: 1,
            source_timestamp: None,
            instance_handle: None,
            data_value: vec![0, 0, 0, 0, 1, 2, 3, 4].into(),
        };
        writer.history_cache().add_change(cache_change.clone());

        let received_cache_change = receiver
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        assert_eq!(cache_change, received_cache_change);
    }

    #[test]
    fn basic_transport_stateless_reader_writer_usage() {
        let guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let domain_id = 0;
        let interface_name = None;
        let udp_receive_buffer_size = None;
        let mut transport = RtpsTransport::new(
            guid_prefix,
            domain_id,
            interface_name,
            udp_receive_buffer_size,
            1344,
        )
        .unwrap();

        struct MockHistoryCache(SyncSender<CacheChange>);

        impl HistoryCache for MockHistoryCache {
            fn add_change(&mut self, cache_change: CacheChange) {
                self.0.send(cache_change).unwrap();
            }

            fn remove_change(&mut self, _sequence_number: i64) {
                todo!()
            }
        }

        let entity_id = EntityId::new([1, 2, 3], 4);
        let (sender, receiver) = sync_channel(0);
        let reader_history_cache = Box::new(MockHistoryCache(sender));
        let _reader = transport.create_stateless_reader(entity_id, reader_history_cache);

        let entity_id = EntityId::new([5, 6, 7], 8);
        let mut writer = transport.create_stateless_writer(entity_id);
        for locator in transport.default_unicast_locator_list() {
            writer.add_reader_locator(locator.clone());
        }

        let cache_change = CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: writer.guid(),
            sequence_number: 1,
            source_timestamp: None,
            instance_handle: None,
            data_value: vec![0, 0, 0, 0, 1, 2, 3, 4].into(),
        };
        writer.history_cache().add_change(cache_change.clone());

        let received_cache_change = receiver
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        assert_eq!(cache_change, received_cache_change);
    }
}
