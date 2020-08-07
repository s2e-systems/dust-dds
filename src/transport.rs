use ipconfig;
use net2::UdpBuilder;

use std::convert::TryInto;
use std::net::{UdpSocket, IpAddr, Ipv4Addr, SocketAddr};

use crate::types::Locator;
use crate::messages::{RtpsMessage, UdpPsmMapping};

#[derive(Debug)]
pub enum TransportError {
    IoError(std::io::Error),
    Other,
}

impl From<std::io::Error> for TransportError {
    fn from(error: std::io::Error) -> Self {
        TransportError::IoError(error)
    }
}

pub type Result<T> = std::result::Result<T, TransportError>;

pub trait Transport {
    fn write(&self, message: RtpsMessage, locator: Locator);

    fn read(&self) -> Result<Option<(RtpsMessage, Locator)>>;
}

const MAX_UDP_DATA_SIZE: usize = 65536;

pub struct UdpTransport {
    socket: UdpSocket,
}

pub fn get_interface_address(interface_name: &str) -> Option<[u8; 16]> {
    for adapter in ipconfig::get_adapters().unwrap() {
        if adapter.friendly_name() == interface_name
        {
            for addr in adapter.ip_addresses() {
                match *addr {
                    IpAddr::V4(ipv4addr) => return Some(ipv4addr.to_ipv6_compatible().octets()),
                    _ => (),
                }
            }
        }
    }

    None
}

impl UdpTransport {
    pub fn new(
        unicast_locator: Locator,
        multicast_locator: Option<Locator>,
    ) -> Result<Self> {
        let socket_builder = UdpBuilder::new_v4()?;
        socket_builder.reuse_address(true)?;
        let unicast_address: [u8;4] = unicast_locator.address()[12..16].try_into().unwrap();
        let port: u16 = unicast_locator.port() as u16;

        let socket = socket_builder.bind(SocketAddr::from((unicast_address, port)))?;

        if let Some(multicast_locator) = multicast_locator {
            socket.set_multicast_loop_v4(true)?;
            let multicast_address: [u8;4] = multicast_locator.address()[12..16].try_into().unwrap();
            let multicast_addr = Ipv4Addr::from(multicast_address);
            let multicast_interface = Ipv4Addr::from(unicast_address);
            socket.join_multicast_v4(&multicast_addr, &multicast_interface)?;
        }

        //socket.set_read_timeout(Some(Duration::new(0/*secs*/, 0/*nanos*/))).expect("Error setting timeout");
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket
        })
    }
}

impl Transport for UdpTransport {
    fn write(&self, message: RtpsMessage, unicast_locator: Locator) {
        let mut buf =  Vec::new();
        message.compose(&mut buf).unwrap();
        let address: [u8;4] = unicast_locator.address()[12..16].try_into().unwrap();
        let port: u16 = unicast_locator.port() as u16;
        self.socket
            .send_to(
                &buf,
                SocketAddr::from((address, port)),
            )
            .unwrap();
    }

    fn read(&self) -> Result<Option<(RtpsMessage, Locator)>> {
        let mut buf = [0_u8; MAX_UDP_DATA_SIZE];
        let recv_result = self.socket.recv_from(&mut buf);
        match recv_result {
            Ok((number_of_bytes, src_addr)) => {
                let message = RtpsMessage::parse(&buf[..number_of_bytes]).ok();
                if let Some(message) = message {
                    let src_locator = match src_addr {
                        SocketAddr::V4(socket_addr_v4) => Locator::new_udpv4(socket_addr_v4.port(), socket_addr_v4.ip().octets()),
                        _ => todo!(),
                    };
            
                    Ok(Some((message, src_locator)))
                } else {
                    Ok(None)
                }
            },
            Err(error) => {
                if error.kind() == std::io::ErrorKind::WouldBlock {
                    Ok(None)
                } else {
                    Err(error.into())
                }
            },
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{RtpsSubmessage, Gap};
    use crate::types::constants::{ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR};
    use crate::messages::Endianness;

    #[test]
    fn read_udp_data() {
        let addr = [127, 0, 0, 1];
        let multicast_group = [239, 255, 0, 1];
        let port = 7405;
        let unicast_locator = Locator::new_udpv4(port, addr);
        let multicast_locator = Locator::new_udpv4(0, multicast_group);

        let transport = UdpTransport::new(unicast_locator, Some(multicast_locator)).unwrap();

        let submessages = vec![
            RtpsSubmessage::Gap(Gap::new(ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, 0, Endianness::LittleEndian)),
        ];
        let message = RtpsMessage::new([1,2,3,4,5,6,7,8,9,10,11,12], submessages);
        let mut bytes = Vec::new();
        message.compose(&mut bytes).unwrap();

        let sender = std::net::UdpSocket::bind(SocketAddr::from((addr, 0))).unwrap();
        sender
            .send_to(&bytes, SocketAddr::from((multicast_group, port)))
            .unwrap();

        let result = transport.read().unwrap();

        let (src_port, src_address) = match sender.local_addr().unwrap() {
            SocketAddr::V4(socket_addr_v4) => (socket_addr_v4.port(), socket_addr_v4.ip().octets()),
            _ => panic!("Expected IPv4"),
        };
        let expected_src_locator = Locator::new_udpv4(src_port, src_address);

        assert_eq!(result, Some((message, expected_src_locator)));
    }

    #[test]
    fn read_udp_no_data() {
        let addr = [127, 0, 0, 1];
        let multicast_group = [239, 255, 0, 1];
        let port = 7400;
        let unicast_locator = Locator::new_udpv4(port, addr);
        let multicast_locator = Locator::new_udpv4(0, multicast_group);

        let transport = UdpTransport::new(unicast_locator, Some(multicast_locator)).unwrap();

        let result = transport.read().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn write_udp_data() {
        let addr = [127, 0, 0, 1];
        let multicast_group = [239, 255, 0, 1];
        let port = 7500;
        let unicast_locator = Locator::new_udpv4(0, addr);
        let multicast_locator = Locator::new_udpv4(0, multicast_group);
        let unicast_locator_sent_to = Locator::new_udpv4(port, addr);

        let transport = UdpTransport::new(unicast_locator, Some(multicast_locator)).unwrap();

        let submessages = vec![
            RtpsSubmessage::Gap(Gap::new(ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, 0, Endianness::LittleEndian)),
        ];
        let message = RtpsMessage::new([1,2,3,4,5,6,7,8,9,10,11,12], submessages);
        let mut expected_bytes = Vec::new();
        message.compose(&mut expected_bytes).unwrap();

        let receiver_address: [u8;4] = unicast_locator.address()[12..16].try_into().unwrap();
        let receiver_port = port as u16;
        let receiver = std::net::UdpSocket::bind(SocketAddr::from((receiver_address, receiver_port))).unwrap();

        transport.write(message, unicast_locator_sent_to);

        let mut buf = [0; MAX_UDP_DATA_SIZE];
        let (size, _) = receiver.recv_from(&mut buf).unwrap();
        let result = &buf[..size];
        assert_eq!(expected_bytes, result);
    }

    #[test]
    fn test_list_adapters() {
        for adapter in ipconfig::get_adapters().unwrap() {
            println!("Adapter: {:?}", adapter.friendly_name());
        }
    }

    #[test]
    fn get_address() {
        let interface = "Wi-Fi";
        println!("Interface {:?} address: {:?}", interface, get_interface_address(&interface));

        let interface = "Invalid";
        println!("Interface {:?} address: {:?}", interface, get_interface_address(&interface));

    }
}
