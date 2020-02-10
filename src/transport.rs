use std::net::UdpSocket;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::net::AddrParseError;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::time::Duration;
use ipconfig;
use net2::UdpBuilder;

use crate::parser::{RtpsMessage, parse_rtps_message, SubMessageType, Data, AckNack};
use crate::{Udpv4Locator};

#[derive(Debug)]
pub enum TransportError {
    IoError(std::io::Error),
}

impl From<std::io::Error> for TransportError {
    fn from(error: std::io::Error) -> Self {
        TransportError::IoError(error)
    }
}

type Result<T> = std::result::Result< T, TransportError>;

pub struct Transport {
    socket: UdpSocket,
}

fn get_wifi_adress() -> Option<Ipv4Addr> {
    for adapter in ipconfig::get_adapters().unwrap() {
        if adapter.friendly_name() == "Loopback Pseudo-Interface 1"/*"Wi-Fi"*/ {
            for addr in adapter.ip_addresses() {
                match *addr {
                    IpAddr::V4(ip4) => return Some(ip4),
                    _ => (),
                }
            }
        }
    }
    
    None
}

impl Transport {

    pub fn new(unicast_locator: Udpv4Locator, multicast_locator: Option<Udpv4Locator>) -> Self {

        let socket_builder = UdpBuilder::new_v4().expect("Couldn't create socket");
        socket_builder.reuse_address(true).expect("Error setting reuse address");
        let socket = socket_builder.bind(SocketAddr::from((unicast_locator.address, unicast_locator.port))).expect("couldn't bind to address");

        if let Some(multicast_locator) = multicast_locator {            
            socket.set_multicast_loop_v4(true).expect("Error setting multicast loop");
            let multicast_addr = Ipv4Addr::from(multicast_locator.address);
            let multicast_interface = Ipv4Addr::from(unicast_locator.address);
            socket.join_multicast_v4(&multicast_addr, &multicast_interface).expect("Error joining multicast group");
        }

        socket.set_read_timeout(Some(Duration::new(1,0))).expect("Error setting timeout");

        Transport {
            socket,
        }
    }

    pub fn write(&self, buf: &[u8], unicast_locator: Udpv4Locator) -> () {
        self.socket.send_to(buf, SocketAddr::from((unicast_locator.address, unicast_locator.port))).unwrap();
    }

    pub fn read(&self) -> Result<Vec<u8>> {
        let mut buf : Vec<u8> = Vec::with_capacity(65536);
        // Resizing to capacity required here since the recv_from() functions takes the size inforation rather 
        // the capacity
        buf.resize(buf.capacity(), 0);
        let (number_of_bytes, src_addr) = self.socket.recv_from(buf.as_mut_slice()).expect("Didn't receive data");
        buf.resize(number_of_bytes, 0);
        Ok(buf)
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn read_data_from_endpoint() {
        let addr = [127,0,0,1];
        let multicast_group = [239,255,0,1];
        let port = 7400;
        let unicast_locator = Udpv4Locator::new_udpv4(&addr, &port);
        let multicast_locator = Udpv4Locator::new_udpv4(&multicast_group, &0);

        let transport = Transport::new(unicast_locator, Some(multicast_locator));

        let expected : Vec<u8> = vec![1, 2, 3];

        let sender = std::net::UdpSocket::bind(SocketAddr::from((addr, 0))).unwrap();
        sender.send_to(&expected, SocketAddr::from((multicast_group, port))).unwrap();

        let result = transport.read().unwrap();
        assert_eq!(expected, result);
        // println!("{:?}", result);
    }

    #[test]
    fn test_list_adapters() {
        for adapter in ipconfig::get_adapters().unwrap() {
            println!("Adapter: {:?}", adapter.friendly_name());
        }

    }
}

