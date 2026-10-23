use crate::{
    infrastructure::error::{DdsError, DdsResult},
    transport::{
        interface::{RtpsParticipant, Transport, Write},
        types::{LOCATOR_KIND_UDP_V6, TransportHandle},
    },
};
use core::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    sync::atomic::AtomicU32,
};
use dust_dds::transport::types::{LOCATOR_KIND_UDP_V4, Locator};
use mio::{Events, Interest, Poll, Token, Waker};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use std::{
    collections::HashMap,
    net::{ToSocketAddrs, UdpSocket},
    pin::Pin,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll as TaskPoll, Waker as TaskWaker},
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

struct TransportSocket {
    transport_handle: TransportHandle,
    socket: mio::net::UdpSocket,
}

const WAKER: Token = Token(0);

struct WaitForReadable<'b> {
    read_waker: &'b (Mutex<Option<TaskWaker>>, Condvar),
    waker: &'b Arc<Waker>,
    registered: bool,
}

impl<'b> Future for WaitForReadable<'b> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> TaskPoll<Self::Output> {
        let mut guard = self.read_waker.0.lock().unwrap();
        if self.registered && guard.is_none() {
            TaskPoll::Ready(())
        } else {
            *guard = Some(cx.waker().clone());
            self.registered = true;
            self.read_waker.1.notify_one();
            self.waker.wake().ok();
            TaskPoll::Pending
        }
    }
}

impl<'b> Drop for WaitForReadable<'b> {
    fn drop(&mut self) {
        let mut guard = self.read_waker.0.lock().unwrap();
        if guard.is_some() {
            *guard = None;
            self.waker.wake().ok();
        }
    }
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
    socket.set_nonblocking(true)?;

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
    socket.set_nonblocking(true)?;

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

pub struct ParticipantSockets {
    default_unicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    fragment_size: usize,
    socket_v4: Option<UdpSocket>,
    socket_v6: Option<UdpSocket>,
    v4_multicast_addresses: Vec<Ipv4Addr>,
    v6_multicast_indices: Vec<u32>,
    write_buffer: Box<[u8; MAX_DATAGRAM_SIZE]>,
    write_buffer_len: usize,
}
impl RtpsParticipant for ParticipantSockets {
    type Writer<'a> = UdpWriter<'a>;

    fn default_unicast_locator_list(&self) -> &[Locator] {
        &self.default_unicast_locator_list
    }

    fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        &self.metatraffic_unicast_locator_list
    }

    fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        &self.metatraffic_multicast_locator_list
    }

    fn default_multicast_locator_list(&self) -> &[Locator] {
        &self.default_multicast_locator_list
    }

    fn fragment_size(&self) -> usize {
        self.fragment_size
    }

    fn writer<'a>(&'a mut self, locator: &'a [Locator]) -> Self::Writer<'a> {
        self.write_buffer_len = 0;

        UdpWriter {
            socket_v4: self.socket_v4.as_ref(),
            socket_v6: self.socket_v6.as_ref(),
            v4_multicast_addresses: &self.v4_multicast_addresses,
            v6_multicast_indices: &self.v6_multicast_indices,
            locator,
            buffer: &mut self.write_buffer,
            buffer_len: &mut self.write_buffer_len,
        }
    }
}
pub struct RtpsUdpTransport {
    interface_name: Option<String>,
    fragment_size: usize,
    udp_receive_buffer_size: Option<usize>,
    ip_version: IpVersion,
    entity_counter: AtomicU32,
    participants: HashMap<TransportHandle, ParticipantSockets>,
    registry: mio::Registry,
    sockets: Vec<TransportSocket>,
    waker: Arc<Waker>,
    read_waker: Arc<(Mutex<Option<TaskWaker>>, Condvar)>,
    is_running: Arc<AtomicBool>,
    reactor_thread: Option<std::thread::JoinHandle<()>>,
    last_read_socket_index: usize,
    receive_buffer: Box<[u8; MAX_DATAGRAM_SIZE]>,
}

impl RtpsUdpTransport {
    fn add_socket(&mut self, transport_handle: TransportHandle, mut socket: mio::net::UdpSocket) {
        let token = Token(self.sockets.len() + 1);
        self.registry
            .register(&mut socket, token, Interest::READABLE)
            .expect("Failed to register socket");
        self.sockets.push(TransportSocket {
            transport_handle,
            socket,
        });
    }

    fn try_recv(&mut self) -> Option<(TransportHandle, usize)> {
        let num_sockets = self.sockets.len();
        if num_sockets == 0 {
            return None;
        }
        for i in 0..num_sockets {
            let idx = (self.last_read_socket_index + 1 + i) % num_sockets;
            let s = &mut self.sockets[idx];
            match s.socket.recv(&mut *self.receive_buffer) {
                Ok(size) if size > 0 => {
                    self.last_read_socket_index = idx;
                    return Some((s.transport_handle, size));
                }
                _ => {}
            }
        }
        None
    }

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

impl Default for RtpsUdpTransport {
    fn default() -> Self {
        let mut poll = Poll::new().expect("Failed to create mio Poll");
        let registry = poll
            .registry()
            .try_clone()
            .expect("Failed to clone mio Registry");
        let waker =
            Arc::new(Waker::new(poll.registry(), WAKER).expect("Failed to create mio Waker"));
        let read_waker = Arc::new((Mutex::new(None::<TaskWaker>), Condvar::new()));
        let is_running = Arc::new(AtomicBool::new(true));

        let is_running_clone = is_running.clone();
        let read_waker_clone = read_waker.clone();
        let reactor_thread = std::thread::Builder::new()
            .name("DustDdsUdpReactor".to_string())
            .spawn(move || {
                let mut events = Events::with_capacity(128);
                while is_running_clone.load(Ordering::Relaxed) {
                    {
                        let mut guard = read_waker_clone.0.lock().unwrap();
                        while guard.is_none() && is_running_clone.load(Ordering::Relaxed) {
                            guard = read_waker_clone.1.wait(guard).unwrap();
                        }
                    }
                    if !is_running_clone.load(Ordering::Relaxed) {
                        break;
                    }

                    match poll.poll(&mut events, None) {
                        Ok(_) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(_) => break,
                    }

                    if !is_running_clone.load(Ordering::Relaxed) {
                        break;
                    }

                    let mut has_socket_event = false;
                    for event in events.iter() {
                        if event.token() != WAKER {
                            has_socket_event = true;
                            break;
                        }
                    }

                    if has_socket_event {
                        let mut guard = read_waker_clone.0.lock().unwrap();
                        if let Some(w) = guard.take() {
                            w.wake();
                        }
                    }
                }
            })
            .expect("Failed to spawn reactor thread");

        Self {
            interface_name: None,
            fragment_size: 1344,
            udp_receive_buffer_size: None,
            ip_version: IpVersion::default(),
            entity_counter: AtomicU32::new(0),
            participants: HashMap::new(),
            registry,
            sockets: Vec::new(),
            waker,
            is_running,
            read_waker,
            reactor_thread: Some(reactor_thread),
            last_read_socket_index: 0,
            receive_buffer: vec![0u8; MAX_DATAGRAM_SIZE]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        }
    }
}

pub struct UdpWriter<'a> {
    socket_v4: Option<&'a UdpSocket>,
    socket_v6: Option<&'a UdpSocket>,
    v4_multicast_addresses: &'a [Ipv4Addr],
    v6_multicast_indices: &'a [u32],
    locator: &'a [Locator],
    buffer: &'a mut [u8; MAX_DATAGRAM_SIZE],
    buffer_len: &'a mut usize,
}

impl<'a> Write for UdpWriter<'a> {
    fn write(&mut self, buf: &[u8]) {
        let new_len = *self.buffer_len + buf.len();
        if new_len <= self.buffer.len() {
            self.buffer[*self.buffer_len..new_len].copy_from_slice(buf);
            *self.buffer_len = new_len;
        }
    }

    fn flush(&mut self) {
        if *self.buffer_len == 0 {
            return;
        }
        let data = &self.buffer[..*self.buffer_len];
        for &destination_locator in self.locator {
            match destination_locator.kind() {
                LOCATOR_KIND_UDP_V4 => {
                    if let Some(socket) = self.socket_v4 {
                        if UdpLocator(destination_locator).is_multicast() {
                            let socket2: socket2::Socket = socket.try_clone().unwrap().into();
                            for address in self.v4_multicast_addresses {
                                if socket2.set_multicast_if_v4(address).is_ok() {
                                    socket.send_to(data, UdpLocator(destination_locator)).ok();
                                }
                            }
                        } else {
                            socket.send_to(data, UdpLocator(destination_locator)).ok();
                        }
                    }
                }
                LOCATOR_KIND_UDP_V6 => {
                    if let Some(socket) = self.socket_v6 {
                        if UdpLocator(destination_locator).is_multicast() {
                            let socket2: socket2::Socket = socket.try_clone().unwrap().into();
                            if self.v6_multicast_indices.is_empty() {
                                socket.send_to(data, UdpLocator(destination_locator)).ok();
                            } else {
                                for &index in self.v6_multicast_indices {
                                    if socket2.set_multicast_if_v6(index).is_ok() {
                                        socket.send_to(data, UdpLocator(destination_locator)).ok();
                                    }
                                }
                            }
                        } else {
                            socket.send_to(data, UdpLocator(destination_locator)).ok();
                        }
                    }
                }
                _ => {}
            }
        }
        *self.buffer_len = 0;
    }
}

impl Transport for RtpsUdpTransport {
    fn create_participant(&mut self, domain_id: i32) -> TransportHandle {
        let transport_handle: TransportHandle = self
            .entity_counter
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed)
            .to_ne_bytes();
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
            default_unicast_socket.set_nonblocking(true).unwrap();
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

            let metatraffic_unicast_socket =
                std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))).unwrap();
            metatraffic_unicast_socket.set_nonblocking(true).unwrap();
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

            let metatraffic_multicast_socket = get_multicast_socket_v4(
                DEFAULT_MULTICAST_LOCATOR_ADDRESS,
                port_builtin_multicast(domain_id),
                v4_multicast_addresses.clone(),
            )
            .unwrap();

            socket_v4 = Some(default_unicast_socket.try_clone().expect("Socket cloning"));

            self.add_socket(
                transport_handle,
                mio::net::UdpSocket::from_std(metatraffic_multicast_socket),
            );
            self.add_socket(
                transport_handle,
                mio::net::UdpSocket::from_std(metatraffic_unicast_socket),
            );
            self.add_socket(
                transport_handle,
                mio::net::UdpSocket::from_std(default_unicast_socket),
            );
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

            let socket2_default_unicast_socket_v6 =
                socket2::Socket::new(socket2::Domain::IPV6, socket2::Type::DGRAM, None).unwrap();
            socket2_default_unicast_socket_v6.set_only_v6(true).ok();
            socket2_default_unicast_socket_v6
                .bind(&SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0)).into())
                .unwrap();
            socket2_default_unicast_socket_v6
                .set_nonblocking(true)
                .unwrap();
            if let Some(buffer_size) = self.udp_receive_buffer_size {
                socket2_default_unicast_socket_v6
                    .set_recv_buffer_size(buffer_size)
                    .unwrap();
            }

            let std_default_unicast_socket_v6 =
                std::net::UdpSocket::from(socket2_default_unicast_socket_v6);
            let user_defined_unicast_port = std_default_unicast_socket_v6
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

            let socket2_metatraffic_unicast_socket_v6 =
                socket2::Socket::new(socket2::Domain::IPV6, socket2::Type::DGRAM, None).unwrap();
            socket2_metatraffic_unicast_socket_v6.set_only_v6(true).ok();
            socket2_metatraffic_unicast_socket_v6
                .bind(&SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0)).into())
                .unwrap();
            socket2_metatraffic_unicast_socket_v6
                .set_nonblocking(true)
                .unwrap();
            let std_metatraffic_unicast_socket_v6 =
                std::net::UdpSocket::from(socket2_metatraffic_unicast_socket_v6);
            let metatraffic_unicast_port = std_metatraffic_unicast_socket_v6
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

            let std_metatraffic_multicast_socket_v6 = get_multicast_socket_v6(
                DEFAULT_MULTICAST_LOCATOR_ADDRESS_V6,
                port_builtin_multicast(domain_id),
                v6_multicast_indices.clone(),
            )
            .unwrap();

            metatraffic_multicast_locator_list.push(Locator::new(
                LOCATOR_KIND_UDP_V6,
                port_builtin_multicast(domain_id) as u32,
                DEFAULT_MULTICAST_LOCATOR_ADDRESS_V6,
            ));

            socket_v6 = Some(
                std_default_unicast_socket_v6
                    .try_clone()
                    .expect("Socket cloning"),
            );

            self.add_socket(
                transport_handle,
                mio::net::UdpSocket::from_std(std_metatraffic_multicast_socket_v6),
            );
            self.add_socket(
                transport_handle,
                mio::net::UdpSocket::from_std(std_metatraffic_unicast_socket_v6),
            );
            self.add_socket(
                transport_handle,
                mio::net::UdpSocket::from_std(std_default_unicast_socket_v6),
            );
        }

        if use_v4 || use_v6 {
            self.waker.wake().ok();
        }

        let participant_sockets = ParticipantSockets {
            default_unicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_multicast_locator_list: Vec::new(),
            fragment_size: self.fragment_size,
            socket_v4,
            socket_v6,
            v4_multicast_addresses,
            v6_multicast_indices,
            write_buffer: Box::new([0; MAX_DATAGRAM_SIZE]),
            write_buffer_len: 0,
        };

        self.participants
            .insert(transport_handle, participant_sockets);

        transport_handle
    }

    fn delete_participant(&mut self, handle: TransportHandle) {
        self.participants.remove(&handle);
    }

    async fn read(&mut self) -> (TransportHandle, &[u8]) {
        loop {
            if let Some((handle, size)) = self.try_recv() {
                return (handle, &self.receive_buffer[..size]);
            }
            WaitForReadable {
                read_waker: &self.read_waker,
                waker: &self.waker,
                registered: false,
            }
            .await;
        }
    }

    type Participant = ParticipantSockets;

    fn get_participant_mut(&mut self, handle: &TransportHandle) -> Option<&mut Self::Participant> {
        self.participants.get_mut(handle)
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

#[cfg(test)]
mod tests {
    use super::*;
    use core::net::Ipv6Addr;
    use dust_dds::transport::types::{LOCATOR_KIND_UDP_V4, LOCATOR_KIND_UDP_V6};

    #[test]
    fn test_mio_multicast_udp() {
        let mut poll = Poll::new().unwrap();
        let mc_port = 17400;
        let mc_socket = get_multicast_socket_v4(
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
            mc_port,
            [Ipv4Addr::LOCALHOST, Ipv4Addr::new(127, 0, 0, 1)],
        )
        .unwrap();

        let mut mio_mc = mio::net::UdpSocket::from_std(mc_socket);
        poll.registry()
            .register(&mut mio_mc, Token(1), Interest::READABLE)
            .unwrap();

        let sender = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let dest = SocketAddr::from(([239, 255, 0, 1], mc_port));
        let socket2: socket2::Socket = sender.try_clone().unwrap().into();
        socket2.set_multicast_if_v4(&Ipv4Addr::LOCALHOST).unwrap();
        sender.send_to(b"multicast ping", dest).unwrap();

        let mut events = Events::with_capacity(10);
        poll.poll(&mut events, Some(std::time::Duration::from_millis(500)))
            .unwrap();
        let tokens: Vec<_> = events.iter().map(|e| e.token()).collect();
        assert!(tokens.contains(&Token(1)));
    }

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
        let factory = RtpsUdpTransport::default();
        assert_eq!(factory.ip_version(), IpVersion::V4);
    }
}
