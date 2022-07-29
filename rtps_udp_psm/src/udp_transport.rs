use std::{
    io::{self, ErrorKind},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket},
    str::FromStr,
};

use crate::mapping_traits::{from_bytes, to_bytes};
use mac_address::MacAddress;
use rtps_pim::structure::types::{LOCATOR_KIND_UDPv4, LOCATOR_KIND_UDPv6, Locator};
use socket2::Socket;

use dds_transport::{RtpsMessage, TransportRead, TransportWrite};

// As of 9.6.1.4.1  Default multicast address
const DEFAULT_MULTICAST_LOCATOR_ADDRESS: [u8; 16] =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1];

const PB: u16 = 7400;
const DG: u16 = 250;
const PG: u16 = 2;
#[allow(non_upper_case_globals)]
const d0: u16 = 0;
#[allow(non_upper_case_globals)]
const d1: u16 = 10;
#[allow(non_upper_case_globals)]
const _d2: u16 = 1;
#[allow(non_upper_case_globals)]
const d3: u16 = 11;

pub fn port_builtin_multicast(domain_id: u16) -> u16 {
    PB + DG * domain_id + d0
}

pub fn port_builtin_unicast(domain_id: u16, participant_id: u16) -> u16 {
    PB + DG * domain_id + d1 + PG * participant_id
}

pub fn port_user_unicast(domain_id: u16, participant_id: u16) -> u16 {
    PB + DG * domain_id + d3 + PG * participant_id
}

pub fn get_multicast_socket(multicast_address: Ipv4Addr, port: u16) -> io::Result<UdpSocket> {
    let socket_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;

    socket.set_reuse_address(true)?;

    //socket.set_nonblocking(true).ok()?;
    socket.set_read_timeout(Some(std::time::Duration::from_millis(50)))?;

    socket.bind(&socket_addr.into())?;

    socket.join_multicast_v4(&multicast_address, &Ipv4Addr::UNSPECIFIED)?;
    socket.set_multicast_loop_v4(true)?;

    Ok(socket.into())
}

pub fn get_unicast_socket(port: u16) -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))?;
    socket.set_nonblocking(true)?;

    Ok(socket)
}

fn ipv4_from_locator(address: &[u8; 16]) -> Ipv4Addr {
    [address[12], address[13], address[14], address[15]].into()
}

#[rustfmt::skip]
fn locator_from_ipv4(address: Ipv4Addr) -> [u8; 16] {
    [0, 0, 0, 0,
     0, 0, 0, 0,
     0, 0, 0, 0,
     address.octets()[0], address.octets()[1], address.octets()[2], address.octets()[3]]
}

type DomainId = i32;

pub struct RtpsUdpPsm {
    domain_id: DomainId,
    participant_id: usize,
    guid_prefix: [u8; 12],
    unicast_address_list: Vec<Ipv4Addr>,
    multicast_address: Ipv4Addr,
    metatraffic_multicast: Option<UdpMulticastTransport>,
    metatraffic_unicast: Option<UdpUnicastTransport>,
    default_unicast: Option<UdpUnicastTransport>,
}

impl RtpsUdpPsm {
    pub fn new(domain_id: DomainId) -> Result<Self, String> {
        let unicast_address_list: Vec<_> = ifcfg::IfCfg::get()
            .expect("Could not scan interfaces")
            .into_iter()
            .flat_map(|i| {
                i.addresses.into_iter().filter_map(|a| match a.address? {
                    SocketAddr::V4(v4) if !v4.ip().is_loopback() => Some(*v4.ip()),
                    _ => None,
                })
            })
            .collect();

        assert!(
            !unicast_address_list.is_empty(),
            "Could not find any IPv4 address"
        );

        let mac_address = ifcfg::IfCfg::get()
            .expect("Could not scan interfaces")
            .into_iter()
            .filter_map(|i| MacAddress::from_str(&i.mac).ok())
            .find(|&mac| mac != MacAddress::new([0, 0, 0, 0, 0, 0]))
            .expect("Could not find any mac address")
            .bytes();

        let multicast_address = ipv4_from_locator(&DEFAULT_MULTICAST_LOCATOR_ADDRESS);
        let metatraffic_multicast_socket =
            get_multicast_socket(multicast_address, port_builtin_multicast(domain_id as u16))
                .map_err(|e| format!("{}", e))?;

        let (participant_id, metatraffic_unicast_socket, default_unicast_socket) = (0..)
            .map(
                |participant_id| -> io::Result<(usize, UdpSocket, UdpSocket)> {
                    Ok((
                        participant_id,
                        get_unicast_socket(port_builtin_unicast(
                            domain_id as u16,
                            participant_id as u16,
                        ))?,
                        get_unicast_socket(port_user_unicast(
                            domain_id as u16,
                            participant_id as u16,
                        ))?,
                    ))
                },
            )
            .find(|result| match result {
                Err(e) => e.kind() != ErrorKind::AddrInUse,
                _ => true,
            })
            .unwrap()
            .map_err(|e| format!("{}", e))?;

        #[rustfmt::skip]
        let guid_prefix = [
            mac_address[0], mac_address[1], mac_address[2],
            mac_address[3], mac_address[4], mac_address[5],
            domain_id as u8, participant_id as u8, 0, 0, 0, 0
        ];

        Ok(Self {
            domain_id,
            participant_id,
            guid_prefix,
            unicast_address_list,
            multicast_address,
            metatraffic_multicast: Some(UdpMulticastTransport::new(metatraffic_multicast_socket)),
            metatraffic_unicast: Some(UdpUnicastTransport::new(metatraffic_unicast_socket)),
            default_unicast: Some(UdpUnicastTransport::new(default_unicast_socket)),
        })
    }

    pub fn metatraffic_multicast_locator_list(&self) -> Vec<Locator> {
        vec![Locator::new(
            LOCATOR_KIND_UDPv4,
            port_builtin_multicast(self.domain_id as u16) as u32,
            locator_from_ipv4(self.multicast_address),
        )]
    }

    pub fn metatraffic_unicast_locator_list(&self) -> Vec<Locator> {
        self.unicast_address_list
            .iter()
            .map(|&address| {
                Locator::new(
                    LOCATOR_KIND_UDPv4,
                    port_builtin_unicast(self.domain_id as u16, self.participant_id as u16) as u32,
                    locator_from_ipv4(address),
                )
            })
            .collect()
    }

    pub fn default_unicast_locator_list(&self) -> Vec<Locator> {
        self.unicast_address_list
            .iter()
            .map(|&address| {
                Locator::new(
                    LOCATOR_KIND_UDPv4,
                    port_user_unicast(self.domain_id as u16, self.participant_id as u16) as u32,
                    locator_from_ipv4(address),
                )
            })
            .collect()
    }

    pub fn default_multicast_locator_list(&self) -> &[Locator] {
        &[]
    }

    pub fn guid_prefix(&self) -> [u8; 12] {
        self.guid_prefix
    }

    pub fn metatraffic_multicast_transport(&mut self) -> Option<UdpMulticastTransport> {
        self.metatraffic_multicast.take()
    }

    pub fn metatraffic_unicast_transport(&mut self) -> Option<UdpUnicastTransport> {
        self.metatraffic_unicast.take()
    }

    pub fn default_unicast_transport(&mut self) -> Option<UdpUnicastTransport> {
        self.default_unicast.take()
    }
}

const BUFFER_SIZE: usize = 32000;
pub struct UdpUnicastTransport {
    socket: UdpSocket,
    receive_buffer: Box<[u8; BUFFER_SIZE]>,
}

impl UdpUnicastTransport {
    pub fn new(socket: UdpSocket) -> Self {
        Self {
            socket,
            receive_buffer: Box::new([0; BUFFER_SIZE]),
        }
    }
}

impl TransportWrite for UdpUnicastTransport {
    fn write(&mut self, message: &RtpsMessage<'_>, destination_locator: Locator) {
        let buf = to_bytes(message).unwrap();
        self.socket
            .send_to(buf.as_slice(), UdpLocator(destination_locator))
            .ok();
    }
}

impl<'a> TransportRead<'a> for UdpUnicastTransport {
    fn read(&'a mut self) -> Option<(Locator, RtpsMessage<'a>)> {
        match self.socket.recv_from(self.receive_buffer.as_mut()) {
            Ok((bytes, source_address)) => {
                if bytes > 0 {
                    let message =
                        from_bytes(&self.receive_buffer[0..bytes]).expect("Failed to deserialize");
                    let udp_locator: UdpLocator = source_address.into();
                    Some((udp_locator.0, message))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

pub struct UdpMulticastTransport {
    socket: UdpSocket,
    receive_buffer: [u8; BUFFER_SIZE],
}

impl UdpMulticastTransport {
    pub fn new(socket: UdpSocket) -> Self {
        Self {
            socket,
            receive_buffer: [0; BUFFER_SIZE],
        }
    }
}

impl TransportWrite for UdpMulticastTransport {
    fn write(&mut self, message: &RtpsMessage<'_>, destination_locator: Locator) {
        let buf = to_bytes(message).unwrap();
        let socket2: socket2::Socket = self.socket.try_clone().unwrap().into();
        let interface_addresses: Vec<_> = ifcfg::IfCfg::get()
            .expect("Could not scan interfaces")
            .into_iter()
            .flat_map(|i| {
                i.addresses.into_iter().filter_map(|a| match a.address? {
                    SocketAddr::V4(v4) => Some(*v4.ip()),
                    _ => None,
                })
            })
            .collect();

        for address in interface_addresses {
            socket2.set_multicast_if_v4(&address).unwrap();
            self.socket
                .send_to(buf.as_slice(), UdpLocator(destination_locator))
                .ok();
        }
    }
}

impl<'a> TransportRead<'a> for UdpMulticastTransport {
    fn read(&'a mut self) -> Option<(Locator, RtpsMessage<'a>)> {
        match self.socket.recv_from(&mut self.receive_buffer) {
            Ok((bytes, source_address)) => {
                if bytes > 0 {
                    let message =
                        from_bytes(&self.receive_buffer[0..bytes]).expect("Failed to deserialize");
                    let udp_locator: UdpLocator = source_address.into();
                    Some((udp_locator.0, message))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

struct UdpLocator(Locator);

impl ToSocketAddrs for UdpLocator {
    type Iter = std::option::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        #[allow(non_upper_case_globals)]
        match *self.0.kind() {
            LOCATOR_KIND_UDPv4 => {
                let locator_address = self.0.address();
                let address = SocketAddrV4::new(
                    Ipv4Addr::new(
                        locator_address[12],
                        locator_address[13],
                        locator_address[14],
                        locator_address[15],
                    ),
                    *self.0.port() as u16,
                );
                Ok(Some(SocketAddr::V4(address)).into_iter())
            }
            LOCATOR_KIND_UDPv6 => todo!(),
            _ => Err(std::io::ErrorKind::InvalidInput.into()),
        }
    }
}

impl From<SocketAddr> for UdpLocator {
    fn from(socket_addr: SocketAddr) -> Self {
        match socket_addr {
            SocketAddr::V4(socket_addr) => {
                let port = socket_addr.port() as u32;
                let address = socket_addr.ip().octets();
                let locator = Locator::new(
                    LOCATOR_KIND_UDPv4,
                    port,
                    [
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, address[0], address[1], address[2],
                        address[3],
                    ],
                );
                UdpLocator(locator)
            }
            SocketAddr::V6(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    use rtps_pim::structure::types::{LOCATOR_KIND_UDPv4, Locator, LOCATOR_INVALID};

    #[test]
    fn udpv4_locator_conversion_address1() {
        let locator = Locator::new(
            LOCATOR_KIND_UDPv4,
            7400,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
        );

        let mut socket_addrs = UdpLocator(locator).to_socket_addrs().unwrap().into_iter();
        let expected_socket_addr = SocketAddr::from_str("127.0.0.1:7400").unwrap();
        assert_eq!(socket_addrs.next(), Some(expected_socket_addr));
    }

    #[test]
    fn udpv4_locator_conversion_address2() {
        let locator = Locator::new(
            LOCATOR_KIND_UDPv4,
            7500,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 25],
        );

        let mut socket_addrs = UdpLocator(locator).to_socket_addrs().unwrap().into_iter();
        let expected_socket_addr = SocketAddr::from_str("192.168.1.25:7500").unwrap();
        assert_eq!(socket_addrs.next(), Some(expected_socket_addr));
    }

    #[test]
    fn locator_conversion_invalid_locator() {
        assert!(UdpLocator(LOCATOR_INVALID).to_socket_addrs().is_err())
    }

    #[test]
    fn socket_addr_to_locator_conversion() {
        let socket_addr = SocketAddr::from_str("127.0.0.1:7400").unwrap();
        let locator = UdpLocator::from(socket_addr).0;
        assert_eq!(locator.kind(), &LOCATOR_KIND_UDPv4);
        assert_eq!(locator.port(), &7400);
        assert_eq!(
            locator.address(),
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1]
        );
    }

    #[test]
    fn new_transport_makes_different_guid() {
        let comm1 = RtpsUdpPsm::new(0).unwrap();
        let comm2 = RtpsUdpPsm::new(0).unwrap();

        assert_ne!(comm1.guid_prefix, comm2.guid_prefix);
    }

    // #[test]
    // fn multicast_write() {
    //     let socket_port = 17400;
    //     let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], socket_port))).unwrap();
    //     socket
    //         .join_multicast_v4(&Ipv4Addr::new(239, 255, 0, 1), &Ipv4Addr::new(127, 0, 0, 1))
    //         .unwrap();
    //     let mut transport = UdpTransport::new(socket);
    //     let header = RtpsMessageHeader {
    //         protocol: rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
    //         version: PROTOCOLVERSION_2_4,
    //         vendor_id: VENDOR_ID_S2E,
    //         guid_prefix: [3; 12],
    //     };
    //     let destination_locator = Locator::new(
    //         LOCATOR_KIND_UDPv4,
    //         socket_port as u32,
    //         [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
    //     );
    //     let message1 = RtpsMessage {
    //         header,
    //         submessages: vec![],
    //     };

    //     transport.write(&message1, &destination_locator);
    //     let (_locator, received_message1) = transport.read().unwrap();
    //     assert_eq!(message1, received_message1);
    // }

    // #[test]
    // fn roundtrip() {
    //     let header = RtpsMessageHeader {
    //         protocol: rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
    //         version: PROTOCOLVERSION_2_4,
    //         vendor_id: VENDOR_ID_S2E,
    //         guid_prefix: [3; 12],
    //     };

    //     let socket_port = 17405;
    //     let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], socket_port))).unwrap();
    //     let mut transport = UdpTransport::new(socket);
    //     let destination_locator = Locator::new(
    //         LOCATOR_KIND_UDPv4,
    //         socket_port as u32,
    //         [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
    //     );

    //     let message1: RtpsMessageUdp = RtpsMessageUdp::new(&header, vec![]);
    //     transport.write(&message1, &destination_locator);
    //     let (_locator, received_message1) = transport.read().unwrap();
    //     assert_eq!(message1, received_message1);

    //     let endianness_flag = true;
    //     let inline_qos_flag = false;
    //     let data_flag = false;
    //     let key_flag = false;
    //     let non_standard_payload_flag = false;
    //     let reader_id = EntityIdUdp {
    //         entity_key: [1, 2, 3],
    //         entity_kind: 4,
    //     };
    //     let writer_id = EntityIdUdp {
    //         entity_key: [6, 7, 8],
    //         entity_kind: 9,
    //     };
    //     let writer_sn = SequenceNumberUdp::new(&5);
    //     let inline_qos = ParameterListUdp {
    //         parameter: vec![].into(),
    //     };
    //     let data = [];
    //     let serialized_payload = SerializedDataUdp(data[..].into());
    //     let submessage = DataSubmessage{
    //         endianness_flag,
    //         inline_qos_flag,
    //         data_flag,
    //         key_flag,
    //         non_standard_payload_flag,
    //         reader_id,
    //         writer_id,
    //         writer_sn,
    //         inline_qos,
    //         serialized_payload,
    //     };
    //     let message2: RtpsMessageUdp =
    //         RtpsMessageUdp::new(&header, vec![RtpsSubmessageType::Data(submessage)]);
    //     transport.write(&message2, &destination_locator);
    //     let (_locator, received_message2) = transport.read().unwrap();
    //     assert_eq!(message2, received_message2);
    // }
}
