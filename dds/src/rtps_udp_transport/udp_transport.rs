use crate::{
    dcps::channels::mpsc::MpscSender,
    std_runtime::{self},
    transport::{
        interface::{RtpsTransportParticipant, TransportParticipantFactory, WriteMessage},
        types::LOCATOR_KIND_UDP_V6,
    },
};
use core::{
    future::Future,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4},
    pin::Pin,
};
use dust_dds::transport::types::{
    ENTITYID_PARTICIPANT, Guid, GuidPrefix, LOCATOR_KIND_UDP_V4, Locator,
};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use std::{
    net::{ToSocketAddrs, UdpSocket},
    sync::Arc,
};
use tracing::info;

const MAX_DATAGRAM_SIZE: usize = 65507;

type LocatorAddress = [u8; 16];
// As of 9.6.1.4.1  Default multicast address
const DEFAULT_MULTICAST_LOCATOR_ADDRESS: LocatorAddress =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1];

const PB: i32 = 7400;
const DG: i32 = 250;
#[allow(non_upper_case_globals)]
const d0: i32 = 0;
fn port_builtin_multicast(domain_id: i32) -> u16 {
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

pub struct RtpsUdpTransportParticipantFactoryBuilder {
    interface_name: Option<String>,
    fragment_size: usize,
    udp_receive_buffer_size: Option<usize>,
}

impl Default for RtpsUdpTransportParticipantFactoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RtpsUdpTransportParticipantFactoryBuilder {
    /// Construct a transport factory builder with all the default options.
    pub fn new() -> Self {
        Self {
            interface_name: None,
            fragment_size: 1344,
            udp_receive_buffer_size: None,
        }
    }

    /// Set the network interface name to use for discovery
    pub fn interface_name(mut self, interface_name: Option<String>) -> Self {
        self.interface_name = interface_name;
        self
    }

    /// Set the maximum size for the data fragments. Types with serialized data above this size will be transmitted as fragments.
    pub fn fragment_size(mut self, fragment_size: usize) -> Self {
        self.fragment_size = fragment_size;
        self
    }

    /// Set the value of the SO_RCVBUF option on the UDP socket. [`None`] corresponds to the OS default
    pub fn udp_receive_buffer_size(mut self, udp_receive_buffer_size: Option<usize>) -> Self {
        self.udp_receive_buffer_size = udp_receive_buffer_size;
        self
    }

    /// Build a new participant factory
    pub fn build(self) -> Result<RtpsUdpTransportParticipantFactory, String> {
        let fragment_size_range = 8..=65000;
        if !fragment_size_range.contains(&self.fragment_size) {
            Err(format!(
                "Interface size out of range. Value must be between in {fragment_size_range:?}",
            ))
        } else {
            Ok(RtpsUdpTransportParticipantFactory {
                interface_name: self.interface_name,
                fragment_size: self.fragment_size,
                udp_receive_buffer_size: self.udp_receive_buffer_size,
            })
        }
    }
}

pub struct RtpsUdpTransportParticipantFactory {
    interface_name: Option<String>,
    fragment_size: usize,
    udp_receive_buffer_size: Option<usize>,
}

impl Default for RtpsUdpTransportParticipantFactory {
    fn default() -> Self {
        RtpsUdpTransportParticipantFactoryBuilder::new()
            .build()
            .expect("Default configuration should work")
    }
}

impl TransportParticipantFactory for RtpsUdpTransportParticipantFactory {
    async fn create_participant(
        &self,
        guid_prefix: GuidPrefix,
        domain_id: i32,
        data_channel_sender: MpscSender<Arc<[u8]>>,
    ) -> RtpsTransportParticipant {
        let interface_address_list = NetworkInterface::show()
            .expect("Could not scan interfaces")
            .into_iter()
            .filter(|x| {
                if let Some(if_name) = &self.interface_name {
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
            socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None).unwrap();
        default_unicast_socket
            .bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)).into())
            .unwrap();
        default_unicast_socket.set_nonblocking(false).unwrap();
        if let Some(buffer_size) = self.udp_receive_buffer_size {
            default_unicast_socket
                .set_recv_buffer_size(buffer_size)
                .unwrap();
        }

        let default_unicast_socket = std::net::UdpSocket::from(default_unicast_socket);
        let user_defined_unicast_port = default_unicast_socket.local_addr().unwrap().port().into();
        let default_unicast_locator_list: Vec<_> = interface_address_list
            .clone()
            .map(|a| Locator::from_ip_and_port(&a, user_defined_unicast_port))
            .collect();
        // Open socket for unicast metatraffic data
        let metatraffic_unicast_socket = Arc::new(
            std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))).unwrap(),
        );

        metatraffic_unicast_socket.set_nonblocking(false).unwrap();
        let metattrafic_unicast_locator_port = metatraffic_unicast_socket
            .local_addr()
            .unwrap()
            .port()
            .into();
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

        let metatraffic_multicast_socket = Arc::new(
            get_multicast_socket(
                DEFAULT_MULTICAST_LOCATOR_ADDRESS,
                port_builtin_multicast(domain_id),
                interface_address_list,
            )
            .unwrap(),
        );

        let message_writer = MessageWriter::new(
            guid_prefix,
            default_unicast_socket.try_clone().expect("Socket cloning"),
        );

        let guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);

        let global_participant = RtpsTransportParticipant {
            guid,
            message_writer: Box::new(message_writer.clone()),
            default_unicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            fragment_size: self.fragment_size,
        };

        let data_channel_sender_clone = data_channel_sender.clone();

        std::thread::Builder::new()
            .name("SomethingOnMetatrafficMulticastSocket".to_string())
            .spawn(move || {
                let mut buf = [0; MAX_DATAGRAM_SIZE];
                loop {
                    if let Ok(size) = metatraffic_multicast_socket.recv(&mut buf) {
                        if size > 0 {
                            std_runtime::executor::block_on(
                                data_channel_sender_clone.send(Arc::from(&buf[..size])),
                            )
                            .expect("chanel_message sender alive");
                        }
                    }
                }
            })
            .expect("failed to spawn thread");

        let data_channel_sender_clone = data_channel_sender.clone();
        std::thread::Builder::new()
            .name("SomethingOnMetatrafficUnicastSocket".to_string())
            .spawn(move || {
                let mut buf = [0; MAX_DATAGRAM_SIZE];
                loop {
                    if let Ok(size) = metatraffic_unicast_socket.recv(&mut buf) {
                        if size > 0 {
                            std_runtime::executor::block_on(
                                data_channel_sender_clone.send(Arc::from(&buf[..size])),
                            )
                            .expect("chanel_message sender alive")
                        }
                    }
                }
            })
            .expect("failed to spawn thread");

        let data_channel_sender_clone = data_channel_sender.clone();
        std::thread::Builder::new()
            .name("SomethingOnDefaultUnicastSocket".to_string())
            .spawn(move || {
                let mut buf = [0; MAX_DATAGRAM_SIZE];
                loop {
                    if let Ok(size) = default_unicast_socket.recv(&mut buf) {
                        if size > 0 {
                            std_runtime::executor::block_on(
                                data_channel_sender_clone.send(Arc::from(&buf[..size])),
                            )
                            .expect("chanel_message sender alive");
                        }
                    }
                }
            })
            .expect("failed to spawn thread");

        global_participant
    }
}

impl Locator {
    pub fn from_ip_and_port(ip_addr: &Addr, port: u32) -> Self {
        match ip_addr.ip() {
            IpAddr::V4(a) => Locator::new(
                LOCATOR_KIND_UDP_V4,
                port,
                [
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    a.octets()[0],
                    a.octets()[1],
                    a.octets()[2],
                    a.octets()[3],
                ],
            ),
            IpAddr::V6(a) => Locator::new(LOCATOR_KIND_UDP_V6, port, a.octets()),
        }
    }
}

struct UdpLocator(Locator);

impl ToSocketAddrs for UdpLocator {
    type Iter = std::option::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        let locator_address = self.0.address();
        match self.0.kind() {
            LOCATOR_KIND_UDP_V4 => {
                let address = SocketAddrV4::new(
                    Ipv4Addr::new(
                        locator_address[12],
                        locator_address[13],
                        locator_address[14],
                        locator_address[15],
                    ),
                    self.0.port() as u16,
                );
                Ok(Some(SocketAddr::V4(address)).into_iter())
            }
            LOCATOR_KIND_UDP_V6 => todo!(),
            _ => Err(std::io::ErrorKind::InvalidInput.into()),
        }
    }
}

impl UdpLocator {
    fn is_multicast(&self) -> bool {
        let locator_address = self.0.address();
        match self.0.kind() {
            LOCATOR_KIND_UDP_V4 => Ipv4Addr::new(
                locator_address[12],
                locator_address[13],
                locator_address[14],
                locator_address[15],
            )
            .is_multicast(),
            LOCATOR_KIND_UDP_V6 => Ipv6Addr::from(locator_address).is_multicast(),
            _ => false,
        }
    }
}

struct MessageWriter {
    guid_prefix: GuidPrefix,
    socket: UdpSocket,
}

impl Clone for MessageWriter {
    fn clone(&self) -> Self {
        Self {
            guid_prefix: self.guid_prefix,
            socket: self.socket.try_clone().expect("Socket cloning"),
        }
    }
}

impl MessageWriter {
    fn new(guid_prefix: GuidPrefix, socket: UdpSocket) -> Self {
        Self {
            guid_prefix,
            socket,
        }
    }
}
impl WriteMessage for MessageWriter {
    fn write_message(
        &self,
        datagram: &[u8],
        locator_list: &[Locator],
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        for &destination_locator in locator_list {
            if UdpLocator(destination_locator).is_multicast() {
                let socket2: socket2::Socket = self.socket.try_clone().unwrap().into();
                let interface_addresses = NetworkInterface::show();
                let interface_addresses: Vec<_> = interface_addresses
                    .expect("Could not scan interfaces")
                    .into_iter()
                    .flat_map(|i| {
                        i.addr.into_iter().filter_map(|a| match a {
                            Addr::V4(v4) => Some(v4.ip),
                            _ => None,
                        })
                    })
                    .collect();
                for address in interface_addresses {
                    if socket2.set_multicast_if_v4(&address).is_ok() {
                        self.socket
                            .send_to(datagram, UdpLocator(destination_locator))
                            .ok();
                    }
                }
            } else {
                self.socket
                    .send_to(datagram, UdpLocator(destination_locator))
                    .ok();
            }
        }
        Box::pin(async {})
    }

    fn guid_prefix(&self) -> GuidPrefix {
        self.guid_prefix
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::transport::types::{DurabilityKind, ENTITYID_UNKNOWN};
//     use dust_dds::transport::types::ChangeKind;
//     use std::sync::mpsc::{SyncSender, sync_channel};

// #[test]
// fn basic_transport_stateless_reader_writer_usage() {
//     let guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
//     let domain_id = 0;
//     let transport = RtpsUdpTransportParticipantFactoryBuilder::new()
//         .build()
//         .unwrap();
//     let mut participant =
//         block_on(async { transport.create_participant(guid_prefix, domain_id).await });

//     struct MockHistoryCache(SyncSender<CacheChange>);

//     impl HistoryCache for MockHistoryCache {
//         fn add_change(
//             &mut self,
//             cache_change: CacheChange,
//         ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
//             self.0.send(cache_change).unwrap();
//             Box::pin(async {})
//         }
//         fn remove_change(
//             &mut self,
//             _sequence_number: i64,
//         ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
//             Box::pin(async {
//                 unimplemented!();
//             })
//         }
//     }

//     let entity_id = EntityId::new([1, 2, 3], 4);
//     let (sender, receiver) = sync_channel(0);
//     let reader_history_cache = Box::new(MockHistoryCache(sender));
//     let _reader = block_on(async {
//         participant
//             .create_stateless_reader(entity_id, reader_history_cache)
//             .await
//     });

//     let entity_id = EntityId::new([5, 6, 7], 8);
//     let mut writer = block_on(async { participant.create_stateless_writer(entity_id).await });
//     for locator in participant.default_unicast_locator_list() {
//         writer.reader_locator_add(locator.clone());
//     }

//     let cache_change = CacheChange {
//         kind: ChangeKind::Alive,
//         writer_guid: writer.guid(),
//         sequence_number: 1,
//         source_timestamp: None,
//         instance_handle: None,
//         data_value: vec![0, 0, 0, 0, 1, 2, 3, 4].into(),
//     };
//     block_on(async {
//         writer
//             .history_cache()
//             .add_change(cache_change.clone())
//             .await
//     });

//     let received_cache_change = receiver
//         .recv_timeout(std::time::Duration::from_secs(30))
//         .unwrap();
//     assert_eq!(cache_change, received_cache_change);
// }

// #[test]
// fn basic_transport_stateful_reader_writer_usage() {
//     let guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
//     let domain_id = 0;
//     let transport = RtpsUdpTransportParticipantFactoryBuilder::new()
//         .build()
//         .unwrap();
//     let mut participant =
//         block_on(async { transport.create_participant(guid_prefix, domain_id).await });

//     struct MockHistoryCache(SyncSender<CacheChange>);

//     impl HistoryCache for MockHistoryCache {
//         fn add_change(
//             &mut self,
//             cache_change: CacheChange,
//         ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
//             self.0.send(cache_change).unwrap();
//             Box::pin(async {})
//         }
//         fn remove_change(
//             &mut self,
//             _sequence_number: i64,
//         ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
//             Box::pin(async {
//                 unimplemented!();
//             })
//         }
//     }

//     let entity_id = EntityId::new([1, 2, 3], 4);
//     let reliability_kind = ReliabilityKind::BestEffort;
//     let (sender, receiver) = sync_channel(0);
//     let reader_history_cache = Box::new(MockHistoryCache(sender));
//     let mut reader = block_on(async {
//         participant
//             .create_stateful_reader(entity_id, reliability_kind, reader_history_cache)
//             .await
//     });

//     let entity_id = EntityId::new([5, 6, 7], 8);
//     let mut writer = block_on(async {
//         participant
//             .create_stateful_writer(entity_id, reliability_kind)
//             .await
//     });

//     let reader_proxy = ReaderProxy {
//         remote_reader_guid: reader.guid(),
//         remote_group_entity_id: ENTITYID_UNKNOWN,
//         reliability_kind,
//         durability_kind: DurabilityKind::Volatile,
//         unicast_locator_list: vec![],
//         multicast_locator_list: vec![],
//         expects_inline_qos: false,
//     };
//     block_on(async { writer.add_matched_reader(reader_proxy).await });

//     let writer_proxy = WriterProxy {
//         remote_writer_guid: writer.guid(),
//         remote_group_entity_id: ENTITYID_UNKNOWN,
//         reliability_kind,
//         durability_kind: DurabilityKind::Volatile,
//         unicast_locator_list: vec![],
//         multicast_locator_list: vec![],
//     };
//     block_on(async { reader.add_matched_writer(writer_proxy).await });
//     let cache_change = CacheChange {
//         kind: ChangeKind::Alive,
//         writer_guid: writer.guid(),
//         sequence_number: 1,
//         source_timestamp: None,
//         instance_handle: None,
//         data_value: vec![0, 0, 0, 0, 1, 2, 3, 4].into(),
//     };
//     block_on(async {
//         writer
//             .history_cache()
//             .add_change(cache_change.clone())
//             .await
//     });

//     let received_cache_change = receiver
//         .recv_timeout(std::time::Duration::from_secs(3))
//         .unwrap();
//     assert_eq!(cache_change, received_cache_change);
// }
// }
