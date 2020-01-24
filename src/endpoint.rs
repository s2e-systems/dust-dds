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

pub struct Endpoint {
    socket: UdpSocket,
}

pub trait RTPSEndpoint {
    fn read_data(&mut self) -> Result<Option<RtpsMessage>, ()>;
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

impl Endpoint {
    pub fn new(/*multicast_locator: Udpv4Locator*/) -> Endpoint {

        let socket_builder = UdpBuilder::new_v4().expect("Couldn't create socket");
        socket_builder.reuse_address(true).expect("Error setting reuse address");
        let socket = socket_builder.bind(SocketAddr::from(([0, 0, 0, 0], 7400))).expect("couldn't bind to address");
        socket.set_multicast_loop_v4(true).expect("Error setting multicast loop");
        let multicast_addr = Ipv4Addr::from_str("239.255.0.1").unwrap();
        let multicast_interface = get_wifi_adress().expect("Error retrieving multicast interface address");
        println!("Wi-fi address {:?}", multicast_interface);
        socket.join_multicast_v4(&multicast_addr, &multicast_interface).expect("Error joining multicast group");
        socket.set_read_timeout(Some(Duration::new(1,0))).expect("Error setting timeout");

        Endpoint {
            socket,
        }
    }

    pub fn read(&self) -> () {
        let mut buf = [0;65536];

        let received = self.socket.recv_from(&mut buf);
        if received.is_ok() {
            let (bytes_received, _src) = received.unwrap();
            let received_message = parse_rtps_message(&buf[0..bytes_received]).unwrap();
            println!("Received message with protocol version {:?}", received_message.get_protocol_version());
            println!("Vendor id {:?}", received_message.get_vendor_id());
            println!("GUID prefix {:?}", received_message.get_guid_prefix());
            println!("Total submessages {:?}", received_message.get_submessages().len());
            for i in received_message.get_submessages().iter() {
                println!("Submessage: {:?}", i);
            }
        } else {
            println!("Didn't receive data within timeout");
        }
    }

}

impl RTPSEndpoint for Endpoint {
    fn read_data(&mut self) -> Result<Option<RtpsMessage>, ()> {
        let mut buf = [0;65536];

        let received = self.socket.recv_from(&mut buf);
        if received.is_ok() {
            let (bytes_received, _src) = received.unwrap();
            let received_message = parse_rtps_message(&buf[0..bytes_received]).unwrap();
            return Ok(Some(received_message));
        } else {
            println!("Didn't receive data within timeout");
            return Ok(None);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_data_from_endpoint() {
        let mut udp_discovery_endpoint = Endpoint::new();

        for _i in 1..=120 {
            println!("Reading data");
            udp_discovery_endpoint.read_data();
        }
    }

    #[test]
    fn test_list_adapters() {
        for adapter in ipconfig::get_adapters().unwrap() {
            println!("Adapter: {:?}", adapter.friendly_name());
        }

    }
}

