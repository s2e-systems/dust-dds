use std::io::{BufRead, Error, Write};

use dds_transport::messages::{
    overall_structure::RtpsSubmessageHeader, RtpsMessage, RtpsSubmessageType,
};

use crate::mapping_traits::{MappingRead, MappingWrite};

use super::submessages::submessage_header::{
    ACKNACK, DATA, DATA_FRAG, GAP, HEARTBEAT, HEARTBEAT_FRAG, INFO_DST, INFO_REPLY, INFO_SRC,
    INFO_TS, NACK_FRAG, PAD,
};

impl MappingWrite for RtpsSubmessageType<'_> {
    fn mapping_write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        match self {
            RtpsSubmessageType::AckNack(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageType::Data(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageType::DataFrag(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageType::Gap(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageType::Heartbeat(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageType::HeartbeatFrag(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageType::InfoDestination(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageType::InfoReply(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageType::InfoSource(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageType::InfoTimestamp(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageType::NackFrag(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageType::Pad(s) => s.mapping_write(&mut writer)?,
        };
        Ok(())
    }
}

impl MappingWrite for RtpsMessage<'_> {
    fn mapping_write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        self.header.mapping_write(&mut writer)?;
        for submessage in &self.submessages {
            submessage.mapping_write(&mut writer)?;
        }
        Ok(())
    }
}

impl<'a, 'de: 'a> MappingRead<'de> for RtpsMessage<'a> {
    fn mapping_read(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let header = MappingRead::mapping_read(buf)?;
        const MAX_SUBMESSAGES: usize = 2_usize.pow(16);
        let mut submessages = vec![];
        for _ in 0..MAX_SUBMESSAGES {
            if buf.len() < 4 {
                break;
            }
            // Preview byte only (to allow full deserialization of submessage header)
            let submessage_id = buf[0];
            let submessage = match submessage_id {
                ACKNACK => RtpsSubmessageType::AckNack(MappingRead::mapping_read(buf)?),
                DATA => RtpsSubmessageType::Data(MappingRead::mapping_read(buf)?),
                DATA_FRAG => RtpsSubmessageType::DataFrag(MappingRead::mapping_read(buf)?),
                GAP => RtpsSubmessageType::Gap(MappingRead::mapping_read(buf)?),
                HEARTBEAT => RtpsSubmessageType::Heartbeat(MappingRead::mapping_read(buf)?),
                HEARTBEAT_FRAG => {
                    RtpsSubmessageType::HeartbeatFrag(MappingRead::mapping_read(buf)?)
                }
                INFO_DST => RtpsSubmessageType::InfoDestination(MappingRead::mapping_read(buf)?),
                INFO_REPLY => RtpsSubmessageType::InfoReply(MappingRead::mapping_read(buf)?),
                INFO_SRC => RtpsSubmessageType::InfoSource(MappingRead::mapping_read(buf)?),
                INFO_TS => RtpsSubmessageType::InfoTimestamp(MappingRead::mapping_read(buf)?),
                NACK_FRAG => RtpsSubmessageType::NackFrag(MappingRead::mapping_read(buf)?),
                PAD => RtpsSubmessageType::Pad(MappingRead::mapping_read(buf)?),
                _ => {
                    let submessage_header: RtpsSubmessageHeader = MappingRead::mapping_read(buf)?;
                    buf.consume(submessage_header.submessage_length as usize);
                    continue;
                }
            };
            submessages.push(submessage);
        }
        Ok(RtpsMessage {
            header,
            submessages,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::mapping_traits::{from_bytes, to_bytes};
    use dds_transport::messages::overall_structure::RtpsMessageHeader;
    use dds_transport::messages::submessage_elements::{
        EntityIdSubmessageElement, GuidPrefixSubmessageElement, Parameter,
        ParameterListSubmessageElement, ProtocolVersionSubmessageElement,
        SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
        VendorIdSubmessageElement,
    };
    use dds_transport::messages::submessages::DataSubmessage;
    use dds_transport::messages::types::ProtocolId;

    #[test]
    fn serialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersionSubmessageElement { value: [2, 3] },
            vendor_id: VendorIdSubmessageElement { value: [9, 8] },
            guid_prefix: GuidPrefixSubmessageElement { value: [3; 12] },
        };
        let value = RtpsMessage {
            header,
            submessages: Vec::new(),
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&value).unwrap(), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]);
    }

    #[test]
    fn serialize_rtps_message() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersionSubmessageElement { value: [2, 3] },
            vendor_id: VendorIdSubmessageElement { value: [9, 8] },
            guid_prefix: GuidPrefixSubmessageElement { value: [3; 12] },
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: [1, 2, 3, 0x04],
        };
        let writer_id = EntityIdSubmessageElement {
            value: [6, 7, 8, 0x09],
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let parameter_1 = Parameter {
            parameter_id: 6,
            length: 4,
            value: &[10, 11, 12, 13],
        };
        let parameter_2 = Parameter {
            parameter_id: 7,
            length: 4,
            value: &[20, 21, 22, 23],
        };
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![parameter_1, parameter_2],
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[][..] };

        let submessage = RtpsSubmessageType::Data(DataSubmessage {
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
        });
        let value = RtpsMessage {
            header,
            submessages: vec![submessage],
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&value).unwrap(), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 0, 0, // inlineQos: Sentinel
        ]);
    }

    #[test]
    fn deserialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersionSubmessageElement { value: [2, 3] },
            vendor_id: VendorIdSubmessageElement { value: [9, 8] },
            guid_prefix: GuidPrefixSubmessageElement { value: [3; 12] },
        };

        let expected = RtpsMessage {
            header,
            submessages: Vec::new(),
        };
        #[rustfmt::skip]
        let result: RtpsMessage = from_bytes(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_rtps_message() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersionSubmessageElement { value: [2, 3] },
            vendor_id: VendorIdSubmessageElement { value: [9, 8] },
            guid_prefix: GuidPrefixSubmessageElement { value: [3; 12] },
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: [1, 2, 3, 0x04],
        };
        let writer_id = EntityIdSubmessageElement {
            value: [6, 7, 8, 0x09],
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let parameter_1 = Parameter {
            parameter_id: 6,
            length: 4,
            value: &[10, 11, 12, 13],
        };
        let parameter_2 = Parameter {
            parameter_id: 7,
            length: 4,
            value: &[20, 21, 22, 23],
        };
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![parameter_1, parameter_2],
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[][..] };

        let submessage = RtpsSubmessageType::Data(DataSubmessage {
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
        });
        let expected = RtpsMessage {
            header,
            submessages: vec![submessage],
        };
        #[rustfmt::skip]
        let result: RtpsMessage = from_bytes(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 0, 0, // inlineQos: Sentinel
        ]).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_rtps_message_unknown_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersionSubmessageElement { value: [2, 3] },
            vendor_id: VendorIdSubmessageElement { value: [9, 8] },
            guid_prefix: GuidPrefixSubmessageElement { value: [3; 12] },
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: [1, 2, 3, 0x04],
        };
        let writer_id = EntityIdSubmessageElement {
            value: [6, 7, 8, 0x09],
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let parameter_1 = Parameter {
            parameter_id: 6,
            length: 4,
            value: &[10, 11, 12, 13],
        };
        let parameter_2 = Parameter {
            parameter_id: 7,
            length: 4,
            value: &[20, 21, 22, 23],
        };
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![parameter_1, parameter_2],
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[][..] };

        let submessage = RtpsSubmessageType::Data(DataSubmessage {
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
        });
        let expected = RtpsMessage {
            header,
            submessages: vec![submessage],
        };
        #[rustfmt::skip]
        let result: RtpsMessage = from_bytes(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x99, 0b_0101_0011, 4, 0, // Submessage header
            9, 9, 9, 9, // Unkown data
            0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 0, 0, // inlineQos: Sentinel
        ]).unwrap();
        assert_eq!(result, expected);
    }
}
