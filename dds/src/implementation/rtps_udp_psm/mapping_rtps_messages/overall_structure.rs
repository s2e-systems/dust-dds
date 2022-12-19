use std::io::{BufRead, Error, Write};

use byteorder::LittleEndian;

use crate::implementation::{
    rtps::messages::{overall_structure::RtpsSubmessageHeader, RtpsMessage, RtpsSubmessageKind},
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrderInfoInData, MappingReadByteOrdered, MappingWriteByteOrderInfoInData,
        MappingWriteByteOrdered,
    },
};

use super::submessages::submessage_header::{
    ACKNACK, DATA, DATA_FRAG, GAP, HEARTBEAT, HEARTBEAT_FRAG, INFO_DST, INFO_REPLY, INFO_SRC,
    INFO_TS, NACK_FRAG, PAD,
};

impl MappingWriteByteOrderInfoInData for RtpsSubmessageKind<'_> {
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        match self {
            RtpsSubmessageKind::AckNack(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageKind::Data(s) => s.mapping_write_byte_order_info_in_data(&mut writer)?,
            RtpsSubmessageKind::DataFrag(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageKind::Gap(s) => s.mapping_write_byte_order_info_in_data(&mut writer)?,
            RtpsSubmessageKind::Heartbeat(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageKind::HeartbeatFrag(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageKind::InfoDestination(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageKind::InfoReply(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageKind::InfoSource(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageKind::InfoTimestamp(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageKind::NackFrag(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageKind::Pad(s) => s.mapping_write_byte_order_info_in_data(&mut writer)?,
        };
        Ok(())
    }
}

impl MappingWriteByteOrderInfoInData for RtpsMessage<'_> {
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        // The byteorder is determined by each submessage individually. Hence
        // decide here for a byteorder for the header
        self.header
            .mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)?;
        for submessage in &self.submessages {
            submessage.mapping_write_byte_order_info_in_data(&mut writer)?;
        }
        Ok(())
    }
}

impl<'a, 'de: 'a> MappingReadByteOrderInfoInData<'de> for RtpsMessage<'a> {
    fn mapping_read_byte_order_info_in_data(buf: &mut &'de [u8]) -> Result<Self, Error> {
        // The byteorder is determined by each submessage individually. Hence
        // decide here for a byteorder for the header
        let header = MappingReadByteOrdered::mapping_read_byte_ordered::<LittleEndian>(buf)?;
        const MAX_SUBMESSAGES: usize = 2_usize.pow(16);
        let mut submessages = vec![];
        for _ in 0..MAX_SUBMESSAGES {
            if buf.len() < 4 {
                break;
            }
            // Preview byte only (to allow full deserialization of submessage header)
            let submessage_id = buf[0];
            let submessage = match submessage_id {
                ACKNACK => RtpsSubmessageKind::AckNack(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                DATA => RtpsSubmessageKind::Data(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                DATA_FRAG => RtpsSubmessageKind::DataFrag(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                GAP => RtpsSubmessageKind::Gap(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                HEARTBEAT => RtpsSubmessageKind::Heartbeat(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                HEARTBEAT_FRAG => RtpsSubmessageKind::HeartbeatFrag(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                INFO_DST => RtpsSubmessageKind::InfoDestination(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                INFO_REPLY => RtpsSubmessageKind::InfoReply(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                INFO_SRC => RtpsSubmessageKind::InfoSource(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                INFO_TS => RtpsSubmessageKind::InfoTimestamp(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                NACK_FRAG => RtpsSubmessageKind::NackFrag(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                PAD => RtpsSubmessageKind::Pad(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                _ => {
                    let submessage_header: RtpsSubmessageHeader =
                        MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?;
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

    use crate::implementation::{
        rtps::{
            messages::{
                overall_structure::RtpsMessageHeader,
                submessage_elements::{Parameter, ParameterList},
                submessages::DataSubmessage,
                types::{ProtocolId, SerializedPayload},
            },
            types::{
                EntityId, EntityKey, GuidPrefix, ProtocolVersion, SequenceNumber, VendorId,
                USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY,
            },
        },
        rtps_udp_psm::mapping_traits::{from_bytes, to_bytes},
    };

    use super::*;

    #[test]
    fn serialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
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
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
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
        let inline_qos = ParameterList {
            parameter: vec![parameter_1, parameter_2],
        };
        let serialized_payload = SerializedPayload::new(&[]);

        let submessage = RtpsSubmessageKind::Data(DataSubmessage {
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
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
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
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
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
        let inline_qos = ParameterList {
            parameter: vec![parameter_1, parameter_2],
        };
        let serialized_payload = SerializedPayload::new(&[]);

        let submessage = RtpsSubmessageKind::Data(DataSubmessage {
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
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
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
        let inline_qos = ParameterList {
            parameter: vec![parameter_1, parameter_2],
        };
        let serialized_payload = SerializedPayload::new(&[]);

        let submessage = RtpsSubmessageKind::Data(DataSubmessage {
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
