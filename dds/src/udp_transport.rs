use std::net::UdpSocket;

use rust_rtps_pim::structure::types::Locator;
use rust_rtps_udp_psm::message::RTPSMessageUdp;
use rust_serde_cdr::serializer::RtpsMessageSerializer;
use serde::ser::Serialize;

pub struct UdpTransport {
    socket: UdpSocket,
}

impl UdpTransport {
    pub fn new() -> Self {
        Self {
            socket: UdpSocket::bind("localhost:52344").unwrap(),
        }
    }

    pub fn write<'a>(&mut self, message: &RTPSMessageUdp<'a>, _destination_locator: &Locator) {
        let json_vec = serde_json::ser::to_string(message).unwrap();
        let json_string = std::str::from_utf8(json_vec.as_ref()).unwrap();
        println!("{:?}", json_string);

        let writer = Vec::<u8>::new();
        let mut serializer = RtpsMessageSerializer { writer };
        message.serialize(&mut serializer).unwrap();
        self.socket
            .send_to(serializer.writer.as_slice(), "localhost:7400")
            .unwrap();
    }
}
