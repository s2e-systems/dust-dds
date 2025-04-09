use core::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4};
use dust_dds::{
    domain::domain_participant_factory::DomainId,
    rtps::{
        error::{RtpsError, RtpsErrorKind, RtpsResult},
        messages::{
            self,
            overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
            types::TIME_INVALID,
        },
        stateful_reader::RtpsStatefulReader,
        stateful_writer::RtpsStatefulWriter,
        stateless_reader::RtpsStatelessReader,
        stateless_writer::RtpsStatelessWriter,
        types::{PROTOCOLVERSION, VENDOR_ID_S2E},
    },
    transport::{
        factory::TransportParticipantFactory,
        history_cache::{CacheChange, HistoryCache},
        participant::TransportParticipant,
        reader::{TransportStatefulReader, TransportStatelessReader, WriterProxy},
        types::{
            EntityId, Guid, GuidPrefix, Locator, ProtocolVersion, ReliabilityKind, VendorId,
            ENTITYID_PARTICIPANT, LOCATOR_KIND_UDP_V4,
        },
        writer::{ReaderProxy, TransportStatefulWriter, TransportStatelessWriter},
    },
};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use std::{
    net::{ToSocketAddrs, UdpSocket},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

use crate::transport::types::LOCATOR_KIND_UDP_V6;

use super::stateless_writer::WriteMessage;

pub struct MessageReceiver {
    source_version: ProtocolVersion,
    source_vendor_id: VendorId,
    source_guid_prefix: GuidPrefix,
    dest_guid_prefix: GuidPrefix,
    have_timestamp: bool,
    timestamp: messages::types::Time,
    submessages: std::vec::IntoIter<RtpsSubmessageReadKind>,
}

impl Iterator for MessageReceiver {
    type Item = RtpsSubmessageReadKind;

    fn next(&mut self) -> Option<Self::Item> {
        for submessage in self.submessages.by_ref() {
            match &submessage {
                RtpsSubmessageReadKind::AckNack(_)
                | RtpsSubmessageReadKind::Data(_)
                | RtpsSubmessageReadKind::DataFrag(_)
                | RtpsSubmessageReadKind::Gap(_)
                | RtpsSubmessageReadKind::Heartbeat(_)
                | RtpsSubmessageReadKind::HeartbeatFrag(_)
                | RtpsSubmessageReadKind::NackFrag(_) => return Some(submessage),

                RtpsSubmessageReadKind::InfoDestination(m) => {
                    self.dest_guid_prefix = m.guid_prefix();
                }
                RtpsSubmessageReadKind::InfoReply(_) => todo!(),
                RtpsSubmessageReadKind::InfoSource(m) => {
                    self.source_vendor_id = m.vendor_id();
                    self.source_version = m.protocol_version();
                    self.source_guid_prefix = m.guid_prefix();
                }
                RtpsSubmessageReadKind::InfoTimestamp(m) => {
                    if !m.invalidate_flag() {
                        self.have_timestamp = true;
                        self.timestamp = m.timestamp();
                    } else {
                        self.have_timestamp = false;
                        self.timestamp = TIME_INVALID;
                    }
                }
                RtpsSubmessageReadKind::Pad(_) => (),
            }
        }
        None
    }
}

pub const GUIDPREFIX_UNKNOWN: GuidPrefix = [0; 12];
impl MessageReceiver {
    pub fn new(message: RtpsMessageRead) -> Self {
        let header = message.header();
        Self {
            source_version: header.version(),
            source_vendor_id: header.vendor_id(),
            source_guid_prefix: header.guid_prefix(),
            dest_guid_prefix: GUIDPREFIX_UNKNOWN,
            have_timestamp: false,
            timestamp: TIME_INVALID,
            submessages: message.submessages().into_iter(),
        }
    }

    fn process_message(
        mut self,
        stateless_reader_list: &mut [RtpsStatelessReader],
        stateful_reader_list: &mut [Arc<Mutex<RtpsStatefulReader>>],
        stateful_writer_list: &mut [StatefulWriterActor],
        message_writer: &impl WriteMessage,
    ) {
        for submessage in self.submessages {
            match &submessage {
                RtpsSubmessageReadKind::AckNack(acknack_submessage) => {
                    for StatefulWriterActor {
                        rtps_stateful_writer,
                        add_matched_reader_receiver: _,
                        remove_matched_reader_receiver: _,
                        add_change_stateful_writer_receiver: _,
                        is_change_acknowledged_receiver: _,
                        remove_change_receiver: _,
                    } in stateful_writer_list.iter_mut()
                    {
                        rtps_stateful_writer.on_acknack_submessage_received(
                            acknack_submessage,
                            self.source_guid_prefix,
                            message_writer,
                        );
                    }
                }
                RtpsSubmessageReadKind::Data(data_submessage) => {
                    let source_timestamp = if self.have_timestamp {
                        Some(self.timestamp)
                    } else {
                        None
                    };
                    for stateless_reader in stateless_reader_list.iter_mut() {
                        stateless_reader.on_data_submessage_received(
                            data_submessage,
                            self.source_guid_prefix,
                            source_timestamp,
                        );
                    }
                    for rtps_stateful_reader in stateful_reader_list.iter_mut() {
                        rtps_stateful_reader
                            .lock()
                            .expect("rtps_stateful_reader alive")
                            .on_data_submessage_received(
                                data_submessage,
                                self.source_guid_prefix,
                                source_timestamp,
                            );
                    }
                }
                RtpsSubmessageReadKind::DataFrag(datafrag_submessage) => {
                    let source_timestamp = if self.have_timestamp {
                        Some(self.timestamp)
                    } else {
                        None
                    };
                    for rtps_stateful_reader in stateful_reader_list.iter_mut() {
                        rtps_stateful_reader
                            .lock()
                            .expect("rtps_stateful_reader alive")
                            .on_data_frag_submessage_received(
                                datafrag_submessage,
                                self.source_guid_prefix,
                                source_timestamp,
                            );
                    }
                }
                RtpsSubmessageReadKind::HeartbeatFrag(heartbeat_frag_submessage) => {
                    for rtps_stateful_reader in stateful_reader_list.iter_mut() {
                        rtps_stateful_reader
                            .lock()
                            .expect("rtps_stateful_reader alive")
                            .on_heartbeat_frag_submessage_received(
                                heartbeat_frag_submessage,
                                self.source_guid_prefix,
                            );
                    }
                }
                RtpsSubmessageReadKind::Gap(gap_submessage) => {
                    for rtps_stateful_reader in stateful_reader_list.iter_mut() {
                        rtps_stateful_reader
                            .lock()
                            .expect("rtps_stateful_reader alive")
                            .on_gap_submessage_received(gap_submessage, self.source_guid_prefix);
                    }
                }
                RtpsSubmessageReadKind::Heartbeat(heartbeat_submessage) => {
                    for rtps_stateful_reader in stateful_reader_list.iter_mut() {
                        rtps_stateful_reader
                            .lock()
                            .expect("rtps_stateful_reader alive")
                            .on_heartbeat_submessage_received(
                                heartbeat_submessage,
                                self.source_guid_prefix,
                                message_writer,
                            );
                    }
                }
                RtpsSubmessageReadKind::NackFrag(nackfrag_submessage) => {
                    for StatefulWriterActor {
                        rtps_stateful_writer,
                        add_matched_reader_receiver: _,
                        remove_matched_reader_receiver: _,
                        add_change_stateful_writer_receiver: _,
                        is_change_acknowledged_receiver: _,
                        remove_change_receiver: _,
                    } in stateful_writer_list.iter_mut()
                    {
                        rtps_stateful_writer.on_nack_frag_submessage_received(
                            nackfrag_submessage,
                            self.source_guid_prefix,
                            message_writer,
                        );
                    }
                }

                RtpsSubmessageReadKind::InfoDestination(m) => {
                    self.dest_guid_prefix = m.guid_prefix();
                }
                RtpsSubmessageReadKind::InfoReply(_) => todo!(),
                RtpsSubmessageReadKind::InfoSource(m) => {
                    self.source_vendor_id = m.vendor_id();
                    self.source_version = m.protocol_version();
                    self.source_guid_prefix = m.guid_prefix();
                }
                RtpsSubmessageReadKind::InfoTimestamp(m) => {
                    if !m.invalidate_flag() {
                        self.have_timestamp = true;
                        self.timestamp = m.timestamp();
                    } else {
                        self.have_timestamp = false;
                        self.timestamp = TIME_INVALID;
                    }
                }
                RtpsSubmessageReadKind::Pad(_) => (),
            }
        }
    }
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
                    println!(
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

pub struct UdpTransportParticipantFactoryBuilder {
    interface_name: Option<String>,
    fragment_size: usize,
    udp_receive_buffer_size: Option<usize>,
}

impl Default for UdpTransportParticipantFactoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl UdpTransportParticipantFactoryBuilder {
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
    pub fn build(self) -> Result<UdpTransportParticipantFactory, String> {
        let fragment_size_range = 8..=65000;
        if !fragment_size_range.contains(&self.fragment_size) {
            Err(format!(
                "Interface size out of range. Value must be between in {:?}",
                fragment_size_range
            ))
        } else {
            Ok(UdpTransportParticipantFactory {
                interface_name: self.interface_name,
                fragment_size: self.fragment_size,
                udp_receive_buffer_size: self.udp_receive_buffer_size,
            })
        }
    }
}

pub struct UdpTransportParticipantFactory {
    interface_name: Option<String>,
    fragment_size: usize,
    udp_receive_buffer_size: Option<usize>,
}

impl Default for UdpTransportParticipantFactory {
    fn default() -> Self {
        UdpTransportParticipantFactoryBuilder::new()
            .build()
            .expect("Default configuration should work")
    }
}

impl TransportParticipantFactory for UdpTransportParticipantFactory {
    fn create_participant(
        &self,
        guid_prefix: GuidPrefix,
        domain_id: i32,
    ) -> Box<dyn TransportParticipant> {
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

        let mut default_unicast_socket = std::net::UdpSocket::from(default_unicast_socket);
        let user_defined_unicast_port = default_unicast_socket.local_addr().unwrap().port().into();
        let default_unicast_locator_list: Vec<_> = interface_address_list
            .clone()
            .map(|a| Locator::from_ip_and_port(&a, user_defined_unicast_port))
            .collect();

        // Open socket for unicast metatraffic data
        let mut metatraffic_unicast_socket =
            std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))).unwrap();
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

        let mut metatraffic_multicast_socket = get_multicast_socket(
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
            port_builtin_multicast(domain_id),
            interface_address_list,
        )
        .unwrap();

        let message_writer = Arc::new(MessageWriter::new(
            guid_prefix,
            std::net::UdpSocket::bind("0.0.0.0:0000").unwrap(),
        ));

        let guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);

        let (add_stateless_reader_sender, add_stateless_reader_receiver) = channel();
        let (add_stateful_reader_sender, add_stateful_reader_receiver) = channel();
        let (add_stateful_writer_sender, add_stateful_writer_receiver) = channel();

        let global_participant = UdpTransportParticipant {
            guid,
            message_writer: message_writer.clone(),
            default_unicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            fragment_size: self.fragment_size,
            add_stateless_reader_sender,
            add_stateful_reader_sender,
            add_stateful_writer_sender,
        };

        let (socket_sender, socket_receiver) = channel();
        let socket_sender_clone = socket_sender.clone();
        std::thread::Builder::new()
            .name("default_unicast_socket".to_string())
            .spawn(move || {
                let mut buf = [0; MAX_DATAGRAM_SIZE];
                loop {
                    if let Ok(rtps_message) =
                        read_message(&mut default_unicast_socket, buf.as_mut_slice())
                    {
                        socket_sender_clone
                            .send(rtps_message)
                            .expect("receiver available");
                    }
                }
            })
            .expect("failed to spawn thread");
        let socket_sender_clone = socket_sender.clone();
        std::thread::Builder::new()
            .name("metatraffic_multicast_socket".to_string())
            .spawn(move || {
                let mut buf = [0; MAX_DATAGRAM_SIZE];
                loop {
                    if let Ok(rtps_message) =
                        read_message(&mut metatraffic_multicast_socket, buf.as_mut_slice())
                    {
                        socket_sender_clone
                            .send(rtps_message)
                            .expect("receiver available");
                    }
                }
            })
            .expect("failed to spawn thread");
        let socket_sender_clone = socket_sender.clone();
        std::thread::Builder::new()
            .name("metatraffic_unicast_socket".to_string())
            .spawn(move || {
                let mut buf = [0; MAX_DATAGRAM_SIZE];
                loop {
                    if let Ok(rtps_message) =
                        read_message(&mut metatraffic_unicast_socket, buf.as_mut_slice())
                    {
                        socket_sender_clone
                            .send(rtps_message)
                            .expect("receiver available");
                    }
                }
            })
            .expect("failed to spawn thread");

        std::thread::Builder::new()
            .name("Socket receiver".to_string())
            .spawn(move || {
                let mut stateless_reader_list = Vec::new();
                let mut stateful_reader_list = Vec::new();
                let mut stateful_writer_list = Vec::new();
                loop {
                    if let Ok(stateless_reader) = add_stateless_reader_receiver.try_recv() {
                        stateless_reader_list.push(stateless_reader);
                    }
                    if let Ok(stateful_writer) = add_stateful_writer_receiver.try_recv() {
                        stateful_writer_list.push(stateful_writer);
                    }
                    if let Ok(stateful_reader_pair) = add_stateful_reader_receiver.try_recv() {
                        stateful_reader_list.push(stateful_reader_pair);
                    }

                    for StatefulWriterActor {
                        rtps_stateful_writer,
                        add_matched_reader_receiver,
                        remove_matched_reader_receiver,
                        add_change_stateful_writer_receiver,
                        is_change_acknowledged_receiver,
                        remove_change_receiver,
                    } in &mut stateful_writer_list
                    {
                        if let Ok(reader_proxy) = add_matched_reader_receiver.try_recv() {
                            rtps_stateful_writer.add_matched_reader(&reader_proxy);
                        }
                        if let Ok(reader_guid) = remove_matched_reader_receiver.try_recv() {
                            rtps_stateful_writer.delete_matched_reader(reader_guid);
                        }
                        if let Ok((sequence_number, reply_sender)) =
                            is_change_acknowledged_receiver.try_recv()
                        {
                            reply_sender
                                .send(rtps_stateful_writer.is_change_acknowledged(sequence_number))
                                .expect("receiver available");
                        }
                        if let Ok(change) = add_change_stateful_writer_receiver.try_recv() {
                            rtps_stateful_writer.add_change(change);
                            rtps_stateful_writer.write_message(message_writer.as_ref());

                        }
                        if let Ok(sequence_number) = remove_change_receiver.try_recv() {
                            rtps_stateful_writer.remove_change(sequence_number);
                        }
                        rtps_stateful_writer.write_message(message_writer.as_ref());
                    }

                    if let Ok(rtps_message) = socket_receiver.try_recv() {
                        MessageReceiver::new(rtps_message).process_message(
                            &mut stateless_reader_list,
                            &mut stateful_reader_list,
                            &mut stateful_writer_list,
                            message_writer.as_ref(),
                        );
                    }
                }
            })
            .expect("failed to spawn thread");

        Box::new(global_participant)
    }
}


struct StatefulWriterActor {
    rtps_stateful_writer: RtpsStatefulWriter,
    add_matched_reader_receiver: Receiver<ReaderProxy>,
    remove_matched_reader_receiver: Receiver<Guid>,
    add_change_stateful_writer_receiver: Receiver<CacheChange>,
    is_change_acknowledged_receiver: Receiver<(i64, Sender<bool>)>,
    remove_change_receiver: Receiver<i64>,
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
        message: &messages::overall_structure::RtpsMessageWrite,
        locator_list: &[Locator],
    ) {

        let buf = message.buffer();

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
                            .send_to(buf, UdpLocator(destination_locator))
                            .ok();
                    }
                }
            } else {
                self.socket
                    .send_to(buf, UdpLocator(destination_locator))
                    .ok();
            }
        }
    }

    fn guid_prefix(&self) -> GuidPrefix {
        self.guid_prefix
    }
}

pub struct UdpTransportParticipant {
    guid: Guid,
    message_writer: Arc<MessageWriter>,
    default_unicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    fragment_size: usize,
    add_stateless_reader_sender: Sender<RtpsStatelessReader>,
    add_stateful_reader_sender: Sender<Arc<Mutex<RtpsStatefulReader>>>,
    add_stateful_writer_sender: Sender<StatefulWriterActor>,
}

impl TransportParticipant for UdpTransportParticipant {
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
        &[]
    }
    fn create_stateless_reader(
        &mut self,
        entity_id: EntityId,
        reader_history_cache: Box<dyn HistoryCache>,
    ) -> Box<dyn TransportStatelessReader> {
        struct StatelessReader {
            guid: Guid,
        }
        impl TransportStatelessReader for StatelessReader {
            fn guid(&self) -> Guid {
                self.guid
            }
        }
        let guid = Guid::new(self.guid.prefix(), entity_id);
        self.add_stateless_reader_sender
            .send(RtpsStatelessReader::new(guid, reader_history_cache))
            .expect("receiver alive");
        Box::new(StatelessReader {
            guid: Guid::new(self.guid.prefix(), entity_id),
        })
    }
    fn create_stateless_writer(
        &mut self,
        entity_id: EntityId,
    ) -> Box<dyn TransportStatelessWriter> {
        struct StatelessWriter {
            rtps_writer: RtpsStatelessWriter,
            message_writer: Arc<MessageWriter>,
        }
        impl TransportStatelessWriter for StatelessWriter {
            fn guid(&self) -> Guid {
                self.rtps_writer.guid()
            }
            fn history_cache(&mut self) -> &mut dyn HistoryCache {
                self
            }

            fn add_reader_locator(&mut self, locator: Locator) {
                self.rtps_writer.reader_locator_add(locator);
            }

            fn remove_reader_locator(&mut self, locator: &Locator) {
                self.rtps_writer.reader_locator_remove(*locator);
            }
        }
        impl HistoryCache for StatelessWriter {
            fn add_change(&mut self, cache_change: CacheChange) {
                self.rtps_writer.add_change(cache_change);
                self.rtps_writer.write_message(self.message_writer.as_ref());
            }

            fn remove_change(&mut self, sequence_number: i64) {
                self.rtps_writer.remove_change(sequence_number);
            }
        }
        let guid = Guid::new(self.guid.prefix(), entity_id);
        Box::new(StatelessWriter {
            rtps_writer: RtpsStatelessWriter::new(guid),
            message_writer: self.message_writer.clone(),
        })
    }

    fn create_stateful_reader(
        &mut self,
        entity_id: EntityId,
        _reliability_kind: ReliabilityKind,
        reader_history_cache: Box<dyn HistoryCache>,
    ) -> Box<dyn TransportStatefulReader> {
        struct StatefulReader {
            guid: Guid,
            rtps_stateful_reader: Arc<Mutex<RtpsStatefulReader>>,
        }
        impl TransportStatefulReader for StatefulReader {
            fn guid(&self) -> Guid {
                self.guid
            }
            fn is_historical_data_received(&self) -> bool {
                self.rtps_stateful_reader
                    .lock()
                    .expect("rtps_stateful_reader is valid")
                    .is_historical_data_received()
            }
            fn add_matched_writer(&mut self, writer_proxy: WriterProxy) {
                self.rtps_stateful_reader
                    .lock()
                    .expect("rtps_stateful_reader is valid")
                    .add_matched_writer(&writer_proxy)
            }
            fn remove_matched_writer(&mut self, remote_writer_guid: Guid) {
                self.rtps_stateful_reader
                    .lock()
                    .expect("rtps_stateful_reader is valid")
                    .delete_matched_writer(remote_writer_guid)
            }
        }

        let guid = Guid::new(self.guid.prefix(), entity_id);
        let rtps_stateful_reader = Arc::new(Mutex::new(RtpsStatefulReader::new(
            guid,
            reader_history_cache,
        )));
        self.add_stateful_reader_sender
            .send(rtps_stateful_reader.clone())
            .expect("receiver alive");
        Box::new(StatefulReader {
            guid,
            rtps_stateful_reader,
        })
    }

    fn create_stateful_writer(
        &mut self,
        entity_id: EntityId,
        _reliability_kind: ReliabilityKind,
    ) -> Box<dyn TransportStatefulWriter> {
        struct StatefulWriter {
            guid: Guid,
            add_matched_reader_sender: Sender<ReaderProxy>,
            remove_matched_reader_sender: Sender<Guid>,
            add_change_stateful_writer_sender: Sender<CacheChange>,
            is_change_acknowledged_sender: Sender<(i64, Sender<bool>)>,
            remove_change_sender: Sender<i64>,
            default_unicast_locator_list: Vec<Locator>,
        }
        impl TransportStatefulWriter for StatefulWriter {
            fn guid(&self) -> Guid {
                self.guid
            }
            fn history_cache(&mut self) -> &mut dyn HistoryCache {
                self
            }
            fn is_change_acknowledged(&self, sequence_number: i64) -> bool {
                let (reply_sender, reply_receiver) = channel();
                self.is_change_acknowledged_sender
                    .send((sequence_number, reply_sender))
                    .expect("receiver available");
                reply_receiver.recv().expect("sender alive")
            }
            fn add_matched_reader(&mut self, mut reader_proxy: ReaderProxy) {
                if reader_proxy.unicast_locator_list.is_empty() {
                    reader_proxy
                        .unicast_locator_list
                        .clone_from(&self.default_unicast_locator_list);
                }
                self.add_matched_reader_sender
                    .send(reader_proxy)
                    .expect("receiver alive");
            }
            fn remove_matched_reader(&mut self, remote_reader_guid: Guid) {
                self.remove_matched_reader_sender
                    .send(remote_reader_guid)
                    .expect("receiver alive");
            }
        }
        impl HistoryCache for StatefulWriter {
            fn add_change(&mut self, cache_change: CacheChange) {
                self.add_change_stateful_writer_sender
                    .send(cache_change)
                    .expect("receiver alive");
            }

            fn remove_change(&mut self, sequence_number: i64) {
                self.remove_change_sender
                    .send(sequence_number)
                    .expect("receiver alive");
            }
        }

        let (add_matched_reader_sender, add_matched_reader_receiver) = std::sync::mpsc::channel();
        let (remove_matched_reader_sender, remove_matched_reader_receiver) =
            std::sync::mpsc::channel();
        let (add_change_stateful_writer_sender, add_change_stateful_writer_receiver) =
            std::sync::mpsc::channel();
        let (is_change_acknowledged_sender, is_change_acknowledged_receiver) =
            std::sync::mpsc::channel();
        let (remove_change_sender, remove_change_receiver) = std::sync::mpsc::channel();
        let guid = Guid::new(self.guid.prefix(), entity_id);
        let rtps_stateful_writer = RtpsStatefulWriter::new(guid, self.fragment_size);
        self.add_stateful_writer_sender
            .send(StatefulWriterActor {
                rtps_stateful_writer,
                add_matched_reader_receiver,
                remove_matched_reader_receiver,
                add_change_stateful_writer_receiver,
                is_change_acknowledged_receiver,
                remove_change_receiver,
            })
            .expect("receiver alive");
        Box::new(StatefulWriter {
            guid,
            add_matched_reader_sender,
            remove_matched_reader_sender,
            add_change_stateful_writer_sender,
            is_change_acknowledged_sender,
            remove_change_sender,
            default_unicast_locator_list: self.default_unicast_locator_list.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{sync_channel, SyncSender};

    use dust_dds::transport::{
        history_cache::CacheChange,
        types::{ChangeKind, DurabilityKind, ENTITYID_UNKNOWN},
        writer::ReaderProxy,
    };

    use super::*;

    #[test]
    fn basic_transport_stateful_reader_writer_usage() {
        let guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let domain_id = 0;
        let transport = UdpTransportParticipantFactoryBuilder::new()
            .build()
            .unwrap();
        let mut participant = transport.create_participant(guid_prefix, domain_id);

        struct MockHistoryCache(SyncSender<CacheChange>);

        impl HistoryCache for MockHistoryCache {
            fn add_change(&mut self, cache_change: CacheChange) {
                self.0.send(cache_change).unwrap();
            }
            fn remove_change(&mut self, _sequence_number: i64) {
                unimplemented!()
            }
        }

        let entity_id = EntityId::new([1, 2, 3], 4);
        let reliability_kind = ReliabilityKind::BestEffort;
        let (sender, receiver) = sync_channel(0);
        let reader_history_cache = Box::new(MockHistoryCache(sender));
        let mut reader =
            participant.create_stateful_reader(entity_id, reliability_kind, reader_history_cache);

        let entity_id = EntityId::new([5, 6, 7], 8);
        let mut writer = participant.create_stateful_writer(entity_id, reliability_kind);

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
            .recv_timeout(std::time::Duration::from_secs(3))
            .unwrap();
        assert_eq!(cache_change, received_cache_change);
    }

    #[test]
    fn basic_transport_stateless_reader_writer_usage() {
        let guid_prefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let domain_id = 0;
        let transport = UdpTransportParticipantFactoryBuilder::new()
            .build()
            .unwrap();
        let mut participant = transport.create_participant(guid_prefix, domain_id);

        struct MockHistoryCache(SyncSender<CacheChange>);

        impl HistoryCache for MockHistoryCache {
            fn add_change(&mut self, cache_change: CacheChange) {
                self.0.send(cache_change).unwrap();
            }
            fn remove_change(&mut self, _sequence_number: i64) {
                unimplemented!()
            }
        }

        let entity_id = EntityId::new([1, 2, 3], 4);
        let (sender, receiver) = sync_channel(0);
        let reader_history_cache = Box::new(MockHistoryCache(sender));
        let _reader = participant.create_stateless_reader(entity_id, reader_history_cache);

        let entity_id = EntityId::new([5, 6, 7], 8);
        let mut writer = participant.create_stateless_writer(entity_id);
        for locator in participant.default_unicast_locator_list() {
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
            .recv_timeout(std::time::Duration::from_secs(30))
            .unwrap();
        assert_eq!(cache_change, received_cache_change);
    }
}
