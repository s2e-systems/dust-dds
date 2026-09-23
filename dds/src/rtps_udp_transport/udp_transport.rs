use crate::{
    infrastructure::error::{DdsError, DdsResult},
    std_runtime::{self},
    transport::{
        interface::{
            RtpsTransportParticipant, TransportDataReceiver, TransportParticipantFactory,
            WriteMessage,
        },
        types::LOCATOR_KIND_UDP_V6,
    },
};
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use dust_dds::transport::types::{LOCATOR_KIND_UDP_V4, Locator};
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
const DEFAULT_MULTICAST_LOCATOR_ADDRESS_V6: LocatorAddress = [
    0xff, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, 239, 255, 0, 1,
];

const PB: i32 = 7400;
const DG: i32 = 250;
#[allow(non_upper_case_globals)]
const d0: i32 = 0;
fn port_builtin_multicast(domain_id: i32) -> u16 {
    (PB + DG * domain_id + d0) as u16
}

fn get_multicast_socket_v4(
    multicast_address: LocatorAddress,
    port: u16,
    interface_address_list: impl IntoIterator<Item = Ipv4Addr>,
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
        let r = socket.join_multicast_v4(&addr, &interface_addr);
        if let Err(e) = r {
            info!(
                "Failed to join multicast group on address {} with error {}",
                interface_addr, e
            );
        }
    }

    socket.set_multicast_loop_v4(true)?;

    Ok(socket.into())
}

fn get_multicast_socket_v6(
    multicast_address: LocatorAddress,
    port: u16,
    interface_indices: impl IntoIterator<Item = u32>,
) -> std::io::Result<std::net::UdpSocket> {
    let socket_addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, port));

    let socket = Socket::new(
        socket2::Domain::IPV6,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;

    socket.set_reuse_address(true)?;
    #[cfg(target_family = "unix")]
    socket.set_reuse_port(true)?;
    socket.set_only_v6(true)?;
    socket.set_nonblocking(false)?;

    socket.bind(&socket_addr.into())?;
    let addr = Ipv6Addr::from(multicast_address);
    let mut joined_any = false;
    for interface_index in interface_indices {
        let r = socket.join_multicast_v6(&addr, interface_index);
        if let Err(e) = r {
            info!(
                "Failed to join multicast group on interface index {} with error {}",
                interface_index, e
            );
        } else {
            joined_any = true;
        }
    }
    if !joined_any {
        let r = socket.join_multicast_v6(&addr, 0);
        if let Err(e) = r {
            info!(
                "Failed to join multicast group on default interface with error {}",
                e
            );
        }
    }

    socket.set_multicast_loop_v6(true)?;

    Ok(socket.into())
}

/// The IP version used by the transport
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IpVersion {
    /// UDP over IPv4
    #[default]
    V4,
    /// UDP over IPv6
    V6,
    /// Both UDP over IPv4 and UDP over IPv6
    Both,
}

pub struct RtpsUdpTransportParticipantFactory {
    interface_name: Option<String>,
    fragment_size: usize,
    udp_receive_buffer_size: Option<usize>,
    ip_version: IpVersion,
}

impl RtpsUdpTransportParticipantFactory {
    /// Set the name of the specific interface to use or None for communicating
    /// through all available interfaces
    pub fn set_interface_name(&mut self, interface_name: Option<String>) -> &mut Self {
        self.interface_name = interface_name;
        self
    }

    /// Set the value of the SO_RCVBUF option on the UDP socket. [`None`] corresponds to the OS default
    pub fn set_udp_receive_buffer_size(
        &mut self,
        udp_receive_buffer_size: Option<usize>,
    ) -> &mut Self {
        self.udp_receive_buffer_size = udp_receive_buffer_size;
        self
    }

    /// Set the fragment size in a range between 8 to 65000. This value is the maximum size of the payload
    /// transmitted in a single RTPS data submessage. Sizes larger than this value will be transmitted in
    /// separate message using RTPS data fragments
    pub fn set_fragment_size(&mut self, fragment_size: usize) -> DdsResult<&mut Self> {
        let fragment_size_range = 8..=65000;
        if !fragment_size_range.contains(&self.fragment_size) {
            return Err(DdsError::BadParameter);
        }
        self.fragment_size = fragment_size;
        Ok(self)
    }

    /// Get the value of the currently configured interface name
    pub fn interface_name(&self) -> Option<&String> {
        self.interface_name.as_ref()
    }

    /// Get the value of the currently configured fragment size
    pub fn fragment_size(&self) -> usize {
        self.fragment_size
    }

    /// Get the value of the currently configured buffer size
    pub fn udp_receive_buffer_size(&self) -> Option<usize> {
        self.udp_receive_buffer_size
    }

    /// Set the IP version to use: IPv4, IPv6, or both
    pub fn set_ip_version(&mut self, ip_version: IpVersion) -> &mut Self {
        self.ip_version = ip_version;
        self
    }

    /// Get the value of the currently configured IP version
    pub fn ip_version(&self) -> IpVersion {
        self.ip_version
    }
}

impl Default for RtpsUdpTransportParticipantFactory {
    fn default() -> Self {
        Self {
            interface_name: None,
            fragment_size: 1344,
            udp_receive_buffer_size: None,
            ip_version: IpVersion::default(),
        }
    }
}

impl TransportParticipantFactory for RtpsUdpTransportParticipantFactory {
    fn create_participant(
        &self,
        domain_id: i32,
        data_channel_sender: TransportDataReceiver,
    ) -> RtpsTransportParticipant {
        let interfaces: Vec<_> = NetworkInterface::show()
            .expect("Could not scan interfaces")
            .into_iter()
            .filter(|interface| {
                self.interface_name
                    .as_ref()
                    .is_none_or(|interface_name| interface_name == &interface.name)
            })
            .collect();

        let use_v4 = self.ip_version == IpVersion::V4 || self.ip_version == IpVersion::Both;
        let use_v6 = self.ip_version == IpVersion::V6 || self.ip_version == IpVersion::Both;

        let mut default_unicast_locator_list = Vec::new();
        let mut metatraffic_unicast_locator_list = Vec::new();
        let mut metatraffic_multicast_locator_list = Vec::new();

        let v4_multicast_addresses: Vec<Ipv4Addr> = if use_v4 {
            interfaces
                .iter()
                .flat_map(|interface| {
                    interface.addr.iter().filter_map(|a| match a {
                        Addr::V4(v4) => Some(v4.ip),
                        Addr::V6(_) => None,
                    })
                })
                .collect()
        } else {
            Vec::new()
        };

        let mut v6_multicast_indices: Vec<u32> = if use_v6 {
            interfaces
                .iter()
                .filter(|i| i.addr.iter().any(|a| matches!(a, Addr::V6(_))))
                .map(|i| i.index)
                .collect()
        } else {
            Vec::new()
        };
        v6_multicast_indices.sort_unstable();
        v6_multicast_indices.dedup();

        let mut socket_v4 = None;
        let mut socket_v6 = None;

        if use_v4 {
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
            let user_defined_unicast_port =
                default_unicast_socket.local_addr().unwrap().port().into();
            for addr in &v4_multicast_addresses {
                default_unicast_locator_list.push(Locator::new(
                    LOCATOR_KIND_UDP_V4,
                    user_defined_unicast_port,
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
                        addr.octets()[0],
                        addr.octets()[1],
                        addr.octets()[2],
                        addr.octets()[3],
                    ],
                ));
            }

            let metatraffic_unicast_socket = Arc::new(
                std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))).unwrap(),
            );
            metatraffic_unicast_socket.set_nonblocking(false).unwrap();
            let metatraffic_unicast_port = metatraffic_unicast_socket
                .local_addr()
                .unwrap()
                .port()
                .into();
            for addr in &v4_multicast_addresses {
                metatraffic_unicast_locator_list.push(Locator::new(
                    LOCATOR_KIND_UDP_V4,
                    metatraffic_unicast_port,
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
                        addr.octets()[0],
                        addr.octets()[1],
                        addr.octets()[2],
                        addr.octets()[3],
                    ],
                ));
            }

            metatraffic_multicast_locator_list.push(Locator::new(
                LOCATOR_KIND_UDP_V4,
                port_builtin_multicast(domain_id) as u32,
                DEFAULT_MULTICAST_LOCATOR_ADDRESS,
            ));

            let metatraffic_multicast_socket = Arc::new(
                get_multicast_socket_v4(
                    DEFAULT_MULTICAST_LOCATOR_ADDRESS,
                    port_builtin_multicast(domain_id),
                    v4_multicast_addresses.clone(),
                )
                .unwrap(),
            );

            socket_v4 = Some(default_unicast_socket.try_clone().expect("Socket cloning"));

            let data_channel_sender_clone = data_channel_sender.clone();
            std::thread::Builder::new()
                .name("SomethingOnMetatrafficMulticastSocket".to_string())
                .spawn(move || {
                    let mut buf = [0; MAX_DATAGRAM_SIZE];
                    loop {
                        if let Ok(size) = metatraffic_multicast_socket.recv(&mut buf) {
                            if size > 0 {
                                std_runtime::executor::block_on(
                                    data_channel_sender_clone.receive_message(buf[..size].to_vec()),
                                );
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
                                    data_channel_sender_clone.receive_message(buf[..size].to_vec()),
                                );
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
                                    data_channel_sender_clone.receive_message(buf[..size].to_vec()),
                                );
                            }
                        }
                    }
                })
                .expect("failed to spawn thread");
        }

        if use_v6 {
            let v6_addresses: Vec<Ipv6Addr> = interfaces
                .iter()
                .flat_map(|interface| {
                    interface.addr.iter().filter_map(|a| match a {
                        Addr::V4(_) => None,
                        Addr::V6(v6) => Some(v6.ip),
                    })
                })
                .collect();

            let default_unicast_socket_v6 =
                socket2::Socket::new(socket2::Domain::IPV6, socket2::Type::DGRAM, None).unwrap();
            default_unicast_socket_v6.set_only_v6(true).ok();
            default_unicast_socket_v6
                .bind(&SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0)).into())
                .unwrap();
            default_unicast_socket_v6.set_nonblocking(false).unwrap();
            if let Some(buffer_size) = self.udp_receive_buffer_size {
                default_unicast_socket_v6
                    .set_recv_buffer_size(buffer_size)
                    .unwrap();
            }

            let default_unicast_socket_v6 = std::net::UdpSocket::from(default_unicast_socket_v6);
            let user_defined_unicast_port = default_unicast_socket_v6
                .local_addr()
                .unwrap()
                .port()
                .into();
            for addr in &v6_addresses {
                default_unicast_locator_list.push(Locator::new(
                    LOCATOR_KIND_UDP_V6,
                    user_defined_unicast_port,
                    addr.octets(),
                ));
            }

            let metatraffic_unicast_socket_v6 =
                socket2::Socket::new(socket2::Domain::IPV6, socket2::Type::DGRAM, None).unwrap();
            metatraffic_unicast_socket_v6.set_only_v6(true).ok();
            metatraffic_unicast_socket_v6
                .bind(&SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0)).into())
                .unwrap();
            metatraffic_unicast_socket_v6
                .set_nonblocking(false)
                .unwrap();
            let metatraffic_unicast_socket_v6 =
                std::net::UdpSocket::from(metatraffic_unicast_socket_v6);
            let metatraffic_unicast_port = metatraffic_unicast_socket_v6
                .local_addr()
                .unwrap()
                .port()
                .into();
            for addr in &v6_addresses {
                metatraffic_unicast_locator_list.push(Locator::new(
                    LOCATOR_KIND_UDP_V6,
                    metatraffic_unicast_port,
                    addr.octets(),
                ));
            }

            let metatraffic_multicast_socket_v6 = Arc::new(
                get_multicast_socket_v6(
                    DEFAULT_MULTICAST_LOCATOR_ADDRESS_V6,
                    port_builtin_multicast(domain_id),
                    v6_multicast_indices.clone(),
                )
                .unwrap(),
            );

            metatraffic_multicast_locator_list.push(Locator::new(
                LOCATOR_KIND_UDP_V6,
                port_builtin_multicast(domain_id) as u32,
                DEFAULT_MULTICAST_LOCATOR_ADDRESS_V6,
            ));

            socket_v6 = Some(
                default_unicast_socket_v6
                    .try_clone()
                    .expect("Socket cloning"),
            );

            let data_channel_sender_clone = data_channel_sender.clone();
            std::thread::Builder::new()
                .name("SomethingOnMetatrafficMulticastSocketV6".to_string())
                .spawn(move || {
                    let mut buf = [0; MAX_DATAGRAM_SIZE];
                    loop {
                        if let Ok(size) = metatraffic_multicast_socket_v6.recv(&mut buf) {
                            if size > 0 {
                                std_runtime::executor::block_on(
                                    data_channel_sender_clone.receive_message(buf[..size].to_vec()),
                                );
                            }
                        }
                    }
                })
                .expect("failed to spawn thread");

            let data_channel_sender_clone = data_channel_sender.clone();
            std::thread::Builder::new()
                .name("SomethingOnMetatrafficUnicastSocketV6".to_string())
                .spawn(move || {
                    let mut buf = [0; MAX_DATAGRAM_SIZE];
                    loop {
                        if let Ok(size) = metatraffic_unicast_socket_v6.recv(&mut buf) {
                            if size > 0 {
                                std_runtime::executor::block_on(
                                    data_channel_sender_clone.receive_message(buf[..size].to_vec()),
                                );
                            }
                        }
                    }
                })
                .expect("failed to spawn thread");

            let data_channel_sender_clone = data_channel_sender.clone();
            std::thread::Builder::new()
                .name("SomethingOnDefaultUnicastSocketV6".to_string())
                .spawn(move || {
                    let mut buf = [0; MAX_DATAGRAM_SIZE];
                    loop {
                        if let Ok(size) = default_unicast_socket_v6.recv(&mut buf) {
                            if size > 0 {
                                std_runtime::executor::block_on(
                                    data_channel_sender_clone.receive_message(buf[..size].to_vec()),
                                );
                            }
                        }
                    }
                })
                .expect("failed to spawn thread");
        }

        let message_writer = MessageWriter::new(
            socket_v4,
            socket_v6,
            v4_multicast_addresses,
            v6_multicast_indices,
        );

        RtpsTransportParticipant {
            message_writer: Box::new(message_writer),
            default_unicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_multicast_locator_list: Vec::new(),
            fragment_size: self.fragment_size,
        }
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
            LOCATOR_KIND_UDP_V6 => {
                let address =
                    SocketAddrV6::new(Ipv6Addr::from(locator_address), self.0.port() as u16, 0, 0);
                Ok(Some(SocketAddr::V6(address)).into_iter())
            }
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
    socket_v4: Option<UdpSocket>,
    socket_v6: Option<UdpSocket>,
    buffer: Box<[u8; MAX_DATAGRAM_SIZE]>,
    v4_multicast_addresses: Vec<Ipv4Addr>,
    v6_multicast_indices: Vec<u32>,
}

impl Clone for MessageWriter {
    fn clone(&self) -> Self {
        Self {
            socket_v4: self
                .socket_v4
                .as_ref()
                .map(|s| s.try_clone().expect("Socket cloning")),
            socket_v6: self
                .socket_v6
                .as_ref()
                .map(|s| s.try_clone().expect("Socket cloning")),
            buffer: vec![0u8; MAX_DATAGRAM_SIZE]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            v4_multicast_addresses: self.v4_multicast_addresses.clone(),
            v6_multicast_indices: self.v6_multicast_indices.clone(),
        }
    }
}

impl MessageWriter {
    fn new(
        socket_v4: Option<UdpSocket>,
        socket_v6: Option<UdpSocket>,
        v4_multicast_addresses: Vec<Ipv4Addr>,
        v6_multicast_indices: Vec<u32>,
    ) -> Self {
        Self {
            socket_v4,
            socket_v6,
            buffer: vec![0u8; MAX_DATAGRAM_SIZE]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            v4_multicast_addresses,
            v6_multicast_indices,
        }
    }
}

impl WriteMessage for MessageWriter {
    fn write_buffer_mut(&mut self) -> &mut [u8] {
        self.buffer.as_mut_slice()
    }

    fn write_message(&mut self, len: usize, locator_list: &[Locator]) {
        let datagram = &self.buffer[..len];
        for &destination_locator in locator_list {
            match destination_locator.kind() {
                LOCATOR_KIND_UDP_V4 => {
                    if let Some(ref socket) = self.socket_v4 {
                        if UdpLocator(destination_locator).is_multicast() {
                            let socket2: socket2::Socket = socket.try_clone().unwrap().into();
                            for address in &self.v4_multicast_addresses {
                                if socket2.set_multicast_if_v4(address).is_ok() {
                                    socket
                                        .send_to(datagram, UdpLocator(destination_locator))
                                        .ok();
                                }
                            }
                        } else {
                            socket
                                .send_to(datagram, UdpLocator(destination_locator))
                                .ok();
                        }
                    }
                }
                LOCATOR_KIND_UDP_V6 => {
                    if let Some(ref socket) = self.socket_v6 {
                        if UdpLocator(destination_locator).is_multicast() {
                            let socket2: socket2::Socket = socket.try_clone().unwrap().into();
                            if self.v6_multicast_indices.is_empty() {
                                socket
                                    .send_to(datagram, UdpLocator(destination_locator))
                                    .ok();
                            } else {
                                for &index in &self.v6_multicast_indices {
                                    if socket2.set_multicast_if_v6(index).is_ok() {
                                        socket
                                            .send_to(datagram, UdpLocator(destination_locator))
                                            .ok();
                                    }
                                }
                            }
                        } else {
                            socket
                                .send_to(datagram, UdpLocator(destination_locator))
                                .ok();
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::net::Ipv6Addr;
    use dust_dds::transport::types::{LOCATOR_KIND_UDP_V4, LOCATOR_KIND_UDP_V6};

    #[test]
    fn udp_locator_v4_to_socket_addrs() {
        let locator = Locator::new(
            LOCATOR_KIND_UDP_V4,
            7400,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
        );
        let mut addrs = UdpLocator(locator).to_socket_addrs().unwrap();
        assert_eq!(
            addrs.next(),
            Some(SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(127, 0, 0, 1),
                7400
            )))
        );
    }

    #[test]
    fn udp_locator_v6_to_socket_addrs() {
        let ip_v6 = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1);
        let locator = Locator::new(LOCATOR_KIND_UDP_V6, 7400, ip_v6.octets());
        let mut addrs = UdpLocator(locator).to_socket_addrs().unwrap();
        assert_eq!(
            addrs.next(),
            Some(SocketAddr::V6(SocketAddrV6::new(ip_v6, 7400, 0, 0)))
        );
    }

    #[test]
    fn udp_locator_is_multicast() {
        let v4_multicast =
            Locator::new(LOCATOR_KIND_UDP_V4, 7400, DEFAULT_MULTICAST_LOCATOR_ADDRESS);
        assert!(UdpLocator(v4_multicast).is_multicast());

        let v4_unicast = Locator::new(
            LOCATOR_KIND_UDP_V4,
            7400,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 1],
        );
        assert!(!UdpLocator(v4_unicast).is_multicast());

        let v6_multicast = Locator::new(
            LOCATOR_KIND_UDP_V6,
            7400,
            DEFAULT_MULTICAST_LOCATOR_ADDRESS_V6,
        );
        assert!(UdpLocator(v6_multicast).is_multicast());

        let v6_unicast = Locator::new(
            LOCATOR_KIND_UDP_V6,
            7400,
            Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1).octets(),
        );
        assert!(!UdpLocator(v6_unicast).is_multicast());
    }

    #[test]
    fn default_ip_version_is_v4() {
        assert_eq!(IpVersion::default(), IpVersion::V4);
        let factory = RtpsUdpTransportParticipantFactory::default();
        assert_eq!(factory.ip_version(), IpVersion::V4);
    }
}
