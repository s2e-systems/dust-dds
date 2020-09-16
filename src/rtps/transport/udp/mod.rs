use ipconfig;
use net2::UdpBuilder;

use std::convert::TryInto;
use std::net::{UdpSocket, IpAddr, Ipv4Addr, SocketAddr};

use crate::rtps::types::Locator;
use crate::rtps::types::constants::LOCATOR_KIND_UDPv4;
use crate::rtps::messages::{RtpsMessage};
use crate::rtps::endpoint_types::DomainId;
use super::{Transport, TransportResult};

mod psm_mapping;

const MAX_UDP_DATA_SIZE: usize = 65536;

pub struct UdpTransport {
    socket: UdpSocket,
    unicast_locator: Locator,
    multicast_locator: Option<Locator>,
}

impl UdpTransport {
    const PB : u32 = 7400;  // TODO: Should be configurable
    const DG : u32 = 250;   // TODO: Should be configurable
    const PG : u32 = 2; // TODO: Should be configurable
    const D0 : u32 = 0; // TODO: Should be configurable
    const D1 : u32 = 10;    // TODO: Should be configurable
    const D2 : u32 = 1; // TODO: Should be configurable
    const D3 : u32 = 11;    // TODO: Should be configurable

    pub fn new(
        unicast_locator: Locator,
        multicast_locator: Option<Locator>,
    ) -> TransportResult<Self> {
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
            socket,
            unicast_locator,
            multicast_locator,
        })
    }

    pub fn default_metatraffic_transport(domain_id: DomainId, interface: &str) -> TransportResult<Self> {
        let spdp_well_known_multicast_port = UdpTransport::PB + UdpTransport::DG * domain_id + UdpTransport::D0;

        let metatraffic_unicast_locator = Locator::new(
            LOCATOR_KIND_UDPv4,
            spdp_well_known_multicast_port,
            get_interface_address(interface).unwrap(),
        );

        let metatraffic_multicast_locator = Locator::new(
            LOCATOR_KIND_UDPv4,
            spdp_well_known_multicast_port,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
        );

        UdpTransport::new(metatraffic_unicast_locator, Some(metatraffic_multicast_locator))
    }

    pub fn default_userdata_transport(_domain_id: DomainId, _interface: &str) -> TransportResult<Self> {
        todo!()
    }

}

impl Transport for UdpTransport {
    fn write(&self, message: RtpsMessage, destination_locator_list: &[Locator]) {
        let mut buf =  Vec::new();
        psm_mapping::serialize_rtps_message(&message, &mut buf).unwrap();

        for locator in destination_locator_list {
            let address: [u8;4] = locator.address()[12..16].try_into().unwrap();
            let port: u16 = locator.port() as u16;
            self.socket
                .send_to(
                    &buf,
                    SocketAddr::from((address, port)),
                )
                .unwrap();
        }
    }

    fn read(&self) -> TransportResult<Option<(RtpsMessage, Locator)>> {
        let mut buf = [0_u8; MAX_UDP_DATA_SIZE];
        let recv_result = self.socket.recv_from(&mut buf);
        match recv_result {
            Ok((number_of_bytes, src_addr)) => {
                let message = psm_mapping::deserialize_rtps_message(&buf[..number_of_bytes]).ok();
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

    fn unicast_locator_list(&self) -> Vec<Locator> {
        vec![self.unicast_locator]
    }
    
    fn multicast_locator_list(&self) -> Vec<Locator> {
        match self.multicast_locator {
            Some(multicast_locator) => vec![multicast_locator],
            None => vec![],
        }
    }
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



#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps::messages::{RtpsSubmessage, Endianness};
    use crate::rtps::messages::submessages::Gap;
    use crate::rtps::types::constants::{ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, PROTOCOL_VERSION_2_4, VENDOR_ID};
    use std::collections::BTreeSet;

    #[test]
    fn read_udp_data() {
        let addr = [127, 0, 0, 1];
        let multicast_group = [239, 255, 0, 1];
        let port = 7405;
        let unicast_locator = Locator::new_udpv4(port, addr);
        let multicast_locator = Locator::new_udpv4(0, multicast_group);

        let transport = UdpTransport::new(unicast_locator, Some(multicast_locator)).unwrap();

        let submessages = vec![
            RtpsSubmessage::Gap(Gap::new(Endianness::LittleEndian, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR, 0, BTreeSet::new())),
        ];
        let message = RtpsMessage::new(PROTOCOL_VERSION_2_4, VENDOR_ID, [1,2,3,4,5,6,7,8,9,10,11,12], submessages);
        let mut bytes = Vec::new();
        psm_mapping::serialize_rtps_message(&message, &mut bytes).unwrap();

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
            RtpsSubmessage::Gap(Gap::new(
                Endianness::LittleEndian,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                0,
                BTreeSet::new())),
        ];
        let message = RtpsMessage::new(PROTOCOL_VERSION_2_4, VENDOR_ID, [1,2,3,4,5,6,7,8,9,10,11,12], submessages);
        let mut expected_bytes = Vec::new();
        psm_mapping::serialize_rtps_message(&message, &mut expected_bytes).unwrap();

        let receiver_address: [u8;4] = unicast_locator.address()[12..16].try_into().unwrap();
        let receiver_port = port as u16;
        let receiver = std::net::UdpSocket::bind(SocketAddr::from((receiver_address, receiver_port))).unwrap();

        transport.write(message, &[unicast_locator_sent_to]);

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
