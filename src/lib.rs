#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::time::Duration;

mod types;
pub mod parser;
mod cache;
mod entity;
mod guid;
mod endpoint;
mod message;
mod participant_proxy;

use types::EntityId;

pub enum LocatorKind {
    LocatorInvalid,
    Invalid,
    Reserved,
    Udpv4,
    Udpv6,
    LocatorAddressInvalid,
    LocatorPortInvalid,
}

trait Locator {

}

pub struct Udpv4Locator {
    pub kind: LocatorKind,
    pub address: [u8;4],
    pub port: u16,
}

impl Udpv4Locator {
    pub fn new_udpv4(address: &[u8;4], port: &u16 ) -> Udpv4Locator {
        Udpv4Locator{
            kind: LocatorKind::Udpv4,
            address: *address,
            port: *port,
        }
    }
}

impl Locator for Udpv4Locator{ }

// pub struct RTPSReader {
//     pub endpoint: RTPSEndpoint,
//     pub sockets: Vec<UdpSocket>
// }

// impl RTPSReader {
//     pub fn new(multicast_locator_list: Vec<Udpv4Locator>) -> RTPSReader {
//         let mut sockets = Vec::with_capacity(1);

//         for _i in &multicast_locator_list {
//             let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 7400))).expect("couldn't bind to address");
//             let multicast_addr = Ipv4Addr::from_str("239.255.0.1").unwrap();
//             let multicast_interface = Ipv4Addr::from_str("192.168.2.5").expect("Error resolving multicast interface address");
//             socket.join_multicast_v4(&multicast_addr, &multicast_interface).expect("Error joining multicast group");
//             socket.set_read_timeout(Some(Duration::new(1,0))).expect("Error setting timeout");
//             sockets.push(socket);
//         }

//         RTPSReader{ 
//             endpoint: RTPSEndpoint{
//                 topic_kind: 0,
//                 reliability_level: 0,
//                 unicast_locator_list: Vec::new(),
//                 multicast_locator_list: Vec::new(),
//                 endpoint_id: [0x00,0x00,0x00,0x00],},
//             sockets: sockets,}
//     }

//     
// }


