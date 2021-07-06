use std::net::UdpSocket;

use rust_dds_rtps_implementation::transport::{TransportLocator, TransportRead, TransportWrite};
use rust_rtps_pim::{messages::RTPSMessagePIM, structure::types::Locator};
use rust_rtps_udp_psm::{message::RTPSMessageC, psm::RtpsUdpPsm};
use rust_serde_cdr::serializer::RtpsMessageSerializer;
use serde::ser::Serialize;

pub struct UdpTransport {
    socket: UdpSocket,
}

impl UdpTransport {
    pub fn new() -> Self {
        Self {
            socket: UdpSocket::bind("192.168.1.12:32454").unwrap(),
        }
    }
}



// fn serialize<T: serde::Serialize>(value: T) -> Vec<u8> {
//     let mut serializer = RtpsMessageSerializer {
//         writer: Vec::<u8>::new(),
//     };
//     value.serialize(&mut serializer).unwrap();
//     serializer.writer
// }


impl<'a> TransportWrite<'a> for UdpTransport {
    type RTPSMessageType = RTPSMessageC<'a>;

    fn write(&mut self, message: &Self::RTPSMessageType, destination_locator: &Locator) {
        let json_vec = serde_json::ser::to_string(message).unwrap();
        let json_string = std::str::from_utf8(json_vec.as_ref()).unwrap();
        println!("{:?}", json_string);

        let writer = Vec::<u8>::new();
        let mut serializer = RtpsMessageSerializer {
                writer,
            };
        message.serialize(&mut serializer).unwrap();
        self.socket.send_to(serializer.writer.as_slice(), "192.168.1.1:7400").unwrap();
    }
}

impl TransportRead<RtpsUdpPsm> for UdpTransport {
    fn read<'a>(&self) -> Option<(<RtpsUdpPsm as RTPSMessagePIM<'a>>::RTPSMessageType, Locator)>
    where
        RtpsUdpPsm: RTPSMessagePIM<'a>,
    {
        todo!()
    }
}

impl TransportLocator for UdpTransport {
    fn unicast_locator_list(&self) -> &[Locator] {
        todo!()
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        todo!()
    }
}
