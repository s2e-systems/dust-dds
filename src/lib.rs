use std::net::UdpSocket;
use std::net::Ipv4Addr;
use std::time::Duration;
use std::str::FromStr;
use std::net::AddrParseError;
use std::net::SocketAddr;
use std::net::SocketAddrV4;

pub mod parser;

type TopicKindT = u32;
type ReliabilityLevelT = u32;
type LocatorT = u32;
type EntityIdT = u32;


enum LocatorKind {
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
    kind: LocatorKind,
    address: [u8;4],
    port: u16,
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

pub struct RTPSEndpoint{
    topic_kind: TopicKindT,
    reliability_level: ReliabilityLevelT,
    unicast_locator_list: Vec<Udpv4Locator>, 
    multicast_locator_list: Vec<Udpv4Locator>,
    endpoint_id: EntityIdT,
}

pub struct RTPSReader {
    endpoint: RTPSEndpoint,
    sockets: Vec<UdpSocket>
}

impl RTPSReader {
    pub fn new(multicast_locator_list: Vec<Udpv4Locator>) -> RTPSReader {
        let mut sockets = Vec::with_capacity(1);

        for i in &multicast_locator_list {
            let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 7400))).expect("couldn't bind to address");
            let multicast_addr = Ipv4Addr::from_str("239.255.0.1").unwrap();
            let multicast_interface = Ipv4Addr::from_str("192.168.2.5").expect("Error resolving multicast interface address");
            socket.join_multicast_v4(&multicast_addr, &multicast_interface).expect("Error joining multicast group");
            socket.set_read_timeout(Some(Duration::new(1,0))).expect("Error setting timeout");
            sockets.push(socket);
        }

        RTPSReader{ 
            endpoint: RTPSEndpoint{
                topic_kind: 0,
                reliability_level: 0,
                unicast_locator_list: Vec::new(),
                multicast_locator_list: Vec::new(),
                endpoint_id: 0,},
            sockets: sockets,}
    }

    pub fn read_data(&self) -> () {
        let mut buf = [0;512];

        for i in &self.sockets {
            i.recv_from(&mut buf);
            println!("Received {:?}", &buf[0 .. 30]);
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let discovery_locator = Udpv4Locator::new_udpv4(&[239,255,0,1],&7400);
//         let reader = RTPSReader::new(vec!(discovery_locator));
//         for _i in 1..=120 {
//             println!("Reading data");
//             reader.read_data();
//         }
//         assert_eq!(2 + 2, 4);
//     }
// }
