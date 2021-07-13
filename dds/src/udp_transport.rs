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

    pub fn write(&mut self, message: &RTPSMessageUdp<'_>, _destination_locator: &Locator) {
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

    pub fn read(&mut self) -> Option<(Locator, RTPSMessageUdp<'_>)>{
        let received = self.socket.recv_from(&mut self.revceive_buffer);

        match received {
            Ok((_bytes, _source_address)) => {
                let mut deserializer = RtpsMessageDeserializer{reader: &self.revceive_buffer};
                let message: RTPSMessageUdp = serde::de::Deserialize::deserialize(&mut deserializer).unwrap();
                let locator = LOCATOR_INVALID;
                Some((locator, message))
            },
            Err(_) => None,
        }
    }
}


#[cfg(test)]
mod tests{
    use rust_rtps_pim::{messages::{RTPSMessage, RtpsMessageHeader, submessage_elements::SequenceNumberSubmessageElementType, submessages::{DataSubmessage, RtpsSubmessageType}}, structure::types::{LOCATOR_INVALID, PROTOCOLVERSION_2_4, VENDOR_ID_S2E}};
    use rust_rtps_udp_psm::{message::RTPSMessageUdp, parameter_list::ParameterListUdp, submessage_elements::{EntityIdUdp, Octet, SequenceNumberUdp, SerializedDataUdp}, submessages::data::DataSubmesageUdp};

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

        let message1: RTPSMessageUdp = RTPSMessageUdp::new(&header, vec![]);
        transport.write(&message1, &destination_locator);
        let (_locator, received_message1) = transport.read().unwrap();
        assert_eq!(message1, received_message1);


        let endianness_flag = true;
        let inline_qos_flag = false;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdUdp {
            entity_key: [Octet(1), Octet(2), Octet(3)],
            entity_kind: Octet(4),
        };
        let writer_id = EntityIdUdp {
            entity_key: [Octet(6), Octet(7), Octet(8)],
            entity_kind: Octet(9),
        };
        let writer_sn = SequenceNumberUdp::new(&5);
        let inline_qos = ParameterListUdp {
            parameter: vec![].into(),
        };
        let data = [];
        let serialized_payload = SerializedDataUdp(data[..].into());
        let submessage = DataSubmesageUdp::new(
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        );
        let message2: RTPSMessageUdp = RTPSMessageUdp::new(&header, vec![RtpsSubmessageType::Data(submessage)]);
        transport.write(&message2, &destination_locator);
        let (_locator, received_message2) = transport.read().unwrap();
        assert_eq!(message2, received_message2);

    }
}
