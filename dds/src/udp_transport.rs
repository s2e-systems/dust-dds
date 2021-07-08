use std::{marker::PhantomData, net::UdpSocket};

use rust_dds_rtps_implementation::transport::{TransportLocator, TransportWrite};
use rust_rtps_pim::{messages::{RTPSMessagePIM, RtpsMessageHeaderType}, structure::types::Locator};
use rust_rtps_udp_psm::{message::RTPSMessageUdp, message_header::RTPSMessageHeaderUdp, psm::RtpsUdpPsm};
use rust_serde_cdr::serializer::RtpsMessageSerializer;
use serde::ser::Serialize;

pub struct UdpTransport<'a> {
    socket: UdpSocket,
    phantom: std::marker::PhantomData<&'a()>
}

impl<'a> UdpTransport<'a> {
    pub fn new() -> Self {
        Self {
            socket: UdpSocket::bind("192.168.1.142:32454").unwrap(),
            phantom: PhantomData
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


impl<'a> TransportWrite for UdpTransport<'a> {
    type RtpsMessageHeaderType = RTPSMessageHeaderUdp;
    type RTPSMessageType = RTPSMessageUdp<'a>;

    fn write<'b>(&mut self, message: &RTPSMessageUdp<'b>, destination_locator: &Locator) {
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

// impl<'a> TransportRead<RtpsUdpPsm<'a>> for UdpTransport<'a> {
//     fn read<'b>(&self) -> Option<(<RtpsUdpPsm as RTPSMessagePIM<'b>>::RTPSMessageType, Locator)>
//     where
//         RtpsUdpPsm: RTPSMessagePIM<'b>,
//     {
//         todo!()
//     }
// }

impl<'a> TransportLocator for UdpTransport<'a> {
    fn unicast_locator_list(&self) -> &[Locator] {
        todo!()
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        todo!()
    }
}
