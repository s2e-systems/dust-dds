use crate::implementation::rtps::{
    messages::overall_structure::{RtpsMessageRead, RtpsMessageWrite},
    transport::TransportWrite,
    types::{Locator, LocatorAddress, LocatorPort, LOCATOR_KIND_UDP_V4, LOCATOR_KIND_UDP_V6},
};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};

pub struct UdpTransportRead {
    socket: tokio::net::UdpSocket,
}

impl UdpTransportRead {
    pub fn new(socket: tokio::net::UdpSocket) -> Self {
        Self { socket }
    }

    pub async fn read(&mut self) -> Option<(Locator, RtpsMessageRead)> {
        let mut message = RtpsMessageRead::new();
        match self.socket.recv_from(&mut message.data).await {
            Ok((bytes, source_address)) => {
                if bytes > 0 {
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

pub struct UdpTransportWrite {
    socket: std::net::UdpSocket,
}

impl UdpTransportWrite {
    pub fn new(socket: std::net::UdpSocket) -> Self {
        Self { socket }
    }
}

impl TransportWrite for UdpTransportWrite {
    fn write(&self, message: &RtpsMessageWrite, destination_locator_list: &[Locator]) {
        let buf = message.buffer();

        for &destination_locator in destination_locator_list {
            if UdpLocator(destination_locator).is_multicast() {
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
}

struct UdpLocator(Locator);

impl ToSocketAddrs for UdpLocator {
    type Iter = std::option::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        let locator_address = <[u8; 16]>::from(self.0.address());
        match self.0.kind() {
            LOCATOR_KIND_UDP_V4 => {
                let address = SocketAddrV4::new(
                    Ipv4Addr::new(
                        locator_address[12],
                        locator_address[13],
                        locator_address[14],
                        locator_address[15],
                    ),
                    <u32>::from(self.0.port()) as u16,
                );
                Ok(Some(SocketAddr::V4(address)).into_iter())
            }
            LOCATOR_KIND_UDP_V6 => todo!(),
            _ => Err(std::io::ErrorKind::InvalidInput.into()),
        }
    }
}

impl From<SocketAddr> for UdpLocator {
    fn from(socket_addr: SocketAddr) -> Self {
        match socket_addr {
            SocketAddr::V4(socket_addr) => {
                let port = LocatorPort::new(socket_addr.port() as u32);
                let address = socket_addr.ip().octets();
                let locator = Locator::new(
                    LOCATOR_KIND_UDP_V4,
                    port,
                    LocatorAddress::new([
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, address[0], address[1], address[2],
                        address[3],
                    ]),
                );
                UdpLocator(locator)
            }
            SocketAddr::V6(_) => todo!(),
        }
    }
}

impl UdpLocator {
    fn is_multicast(&self) -> bool {
        let locator_address = <[u8; 16]>::from(self.0.address());
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
    use std::str::FromStr;

    use crate::implementation::rtps::types::{LocatorAddress, LocatorPort, LOCATOR_INVALID};

    use super::*;

    #[test]
    fn udpv4_locator_conversion_address1() {
        let locator = Locator::new(
            LOCATOR_KIND_UDP_V4,
            LocatorPort::new(7400),
            LocatorAddress::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1]),
        );

        let mut socket_addrs = UdpLocator(locator).to_socket_addrs().unwrap();
        let expected_socket_addr = SocketAddr::from_str("127.0.0.1:7400").unwrap();
        assert_eq!(socket_addrs.next(), Some(expected_socket_addr));
    }

    #[test]
    fn udpv4_locator_conversion_address2() {
        let locator = Locator::new(
            LOCATOR_KIND_UDP_V4,
            LocatorPort::new(7500),
            LocatorAddress::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 25]),
        );

        let mut socket_addrs = UdpLocator(locator).to_socket_addrs().unwrap();
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
        assert_eq!(locator.kind(), LOCATOR_KIND_UDP_V4);
        assert_eq!(locator.port(), LocatorPort::new(7400));
        assert_eq!(
            locator.address(),
            LocatorAddress::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1])
        );
    }
}
