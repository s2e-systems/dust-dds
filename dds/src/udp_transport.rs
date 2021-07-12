use std::{net::UdpSocket};

use rust_rtps_pim::structure::types::{LOCATOR_INVALID, Locator};
use rust_rtps_udp_psm::message::RTPSMessageUdp;
use rust_serde_cdr::{deserializer::RtpsMessageDeserializer, serializer::RtpsMessageSerializer};
use serde::ser::Serialize;


const BUFFER_SIZE:usize = 32000;
pub struct UdpTransport {
    socket: UdpSocket,
    revceive_buffer: [u8; BUFFER_SIZE],
}

impl UdpTransport {
    pub fn new() -> Self {
        Self {
            socket: UdpSocket::bind("localhost:7400").unwrap(),
            revceive_buffer: [0; BUFFER_SIZE],
        }
    }

    pub fn write<'a>(&mut self, message: &RTPSMessageUdp<'a>, _destination_locator: &Locator) {
        let json_vec = serde_json::ser::to_string(message).unwrap();
        let json_string = std::str::from_utf8(json_vec.as_ref()).unwrap();
        println!("{:?}", json_string);

        // let writer = &mut [0_u8; BUFFER_SIZE][..];
        let writer = Vec::<u8>::new();
        let mut serializer = RtpsMessageSerializer { writer };
        message.serialize(&mut serializer).unwrap();
        self.socket
            .send_to(serializer.writer.as_slice(), "localhost:7400")
            .unwrap();
    }

    pub fn read<'a>(&'a mut self/* , mut buffer: &'a mut [u8]*/) -> (Locator, RTPSMessageUdp<'a>){
        let received = self.socket.recv_from(&mut self.revceive_buffer);

        let  (locator, message) = match received {
            Ok((_bytes, _source_address)) => {
                let mut deserializer = RtpsMessageDeserializer{reader: &self.revceive_buffer};
                let message: RTPSMessageUdp = serde::de::Deserialize::deserialize(&mut deserializer).unwrap();
                let locator = LOCATOR_INVALID;
                (locator, message)
            },
            Err(_) => todo!(),
        };
        (locator, message)
    }
}


#[cfg(test)]
mod tests{
    use rust_rtps_pim::{messages::{RTPSMessage, RtpsMessageHeader}, structure::types::{LOCATOR_INVALID, PROTOCOLVERSION_2_4, VENDOR_ID_S2E}};
    use rust_rtps_udp_psm::{message::RTPSMessageUdp};

    use crate::udp_transport::UdpTransport;

    #[test]
    fn roundtrip() {
        let header = RtpsMessageHeader {
            protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
            version: PROTOCOLVERSION_2_4,
            vendor_id: VENDOR_ID_S2E,
            guid_prefix: [3; 12],
        };

        let mut transport = UdpTransport::new();
        let destination_locator = LOCATOR_INVALID;

        let message: RTPSMessageUdp = RTPSMessageUdp::new(&header, vec![]);
        transport.write(&message, &destination_locator);
        let (_locator, received_message1) = transport.read();
        assert_eq!(message, received_message1);

        let message: RTPSMessageUdp = RTPSMessageUdp::new(&header, vec![]);
        transport.write(&message, &destination_locator);
        let (_locator, received_message2) = transport.read();
        assert_eq!(message, received_message2);

    }
}
