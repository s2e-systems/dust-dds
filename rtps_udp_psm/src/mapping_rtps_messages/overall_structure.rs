use std::io::{BufRead, Write};

use rust_rtps_pim::{
    messages::{
        submessage_elements::Parameter, submessages::RtpsSubmessageType, RtpsMessage,
        RtpsSubmessageHeader,
    },
    structure::types::SequenceNumber,
};

use crate::{
    deserialize::{self, MappingRead},
    serialize::MappingWrite,
};

use super::submessages::submessage_header::{
    ACKNACK, DATA, DATA_FRAG, GAP, HEARTBEAT, HEARTBEAT_FRAG, INFO_DST, INFO_REPLY, INFO_SRC,
    INFO_TS, NACK_FRAG, PAD,
};

type RtpsSubmessageWrite<'a> =
    RtpsSubmessageType<'a, Vec<SequenceNumber>, &'a [Parameter<'a>], (), ()>;
type RtpsSubmessageRead<'a> =
    RtpsSubmessageType<'a, Vec<SequenceNumber>, Vec<Parameter<'a>>, (), ()>;

pub type RtpsMessageWrite<'a> = RtpsMessage<Vec<RtpsSubmessageWrite<'a>>>;
pub type RtpsMessageRead<'a> = RtpsMessage<Vec<RtpsSubmessageRead<'a>>>;

impl MappingWrite for RtpsSubmessageWrite<'_> {
    fn write<W: Write>(&self, mut writer: W) -> crate::serialize::Result {
        match self {
            RtpsSubmessageType::AckNack(s) => s.write(&mut writer)?,
            RtpsSubmessageType::Data(s) => s.write(&mut writer)?,
            RtpsSubmessageType::DataFrag(s) => s.write(&mut writer)?,
            RtpsSubmessageType::Gap(s) => s.write(&mut writer)?,
            RtpsSubmessageType::Heartbeat(s) => s.write(&mut writer)?,
            RtpsSubmessageType::HeartbeatFrag(s) => s.write(&mut writer)?,
            RtpsSubmessageType::InfoDestination(s) => s.write(&mut writer)?,
            RtpsSubmessageType::InfoReply(s) => s.write(&mut writer)?,
            RtpsSubmessageType::InfoSource(s) => s.write(&mut writer)?,
            RtpsSubmessageType::InfoTimestamp(s) => s.write(&mut writer)?,
            RtpsSubmessageType::NackFrag(s) => s.write(&mut writer)?,
            RtpsSubmessageType::Pad(s) => s.write(&mut writer)?,
        };
        Ok(())
    }
}

impl MappingWrite for RtpsMessageWrite<'_> {
    fn write<W: Write>(&self, mut writer: W) -> crate::serialize::Result {
        self.header.write(&mut writer)?;
        for submessage in &self.submessages {
            submessage.write(&mut writer)?;
        }
        Ok(())
    }
}

impl<'a, 'de: 'a> MappingRead<'de> for RtpsMessageRead<'a> {
    fn read(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        let header = MappingRead::read(buf)?;
        const MAX_SUBMESSAGES: usize = 2_usize.pow(16);
        let mut submessages = vec![];
        for _ in 0..MAX_SUBMESSAGES {
            if buf.len() < 4 {
                break;
            }
            // Preview byte only (to allow full deserialization of submessage header)
            let submessage_id = buf[0];
            let submessage = match submessage_id {
                ACKNACK => RtpsSubmessageType::AckNack(MappingRead::read(buf)?),
                DATA => RtpsSubmessageType::Data(MappingRead::read(buf)?),
                DATA_FRAG => RtpsSubmessageType::DataFrag(MappingRead::read(buf)?),
                GAP => RtpsSubmessageType::Gap(MappingRead::read(buf)?),
                HEARTBEAT => RtpsSubmessageType::Heartbeat(MappingRead::read(buf)?),
                HEARTBEAT_FRAG => RtpsSubmessageType::HeartbeatFrag(MappingRead::read(buf)?),
                INFO_DST => RtpsSubmessageType::InfoDestination(MappingRead::read(buf)?),
                INFO_REPLY => RtpsSubmessageType::InfoReply(MappingRead::read(buf)?),
                INFO_SRC => RtpsSubmessageType::InfoSource(MappingRead::read(buf)?),
                INFO_TS => RtpsSubmessageType::InfoTimestamp(MappingRead::read(buf)?),
                NACK_FRAG => RtpsSubmessageType::NackFrag(MappingRead::read(buf)?),
                PAD => RtpsSubmessageType::Pad(MappingRead::read(buf)?),
                _ => {
                    let submessage_header: RtpsSubmessageHeader = MappingRead::read(buf)?;
                    buf.consume(submessage_header.submessage_length as usize);
                    continue;
                }
            };
            submessages.push(submessage);
        }
        Ok(Self {
            header,
            submessages,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::deserialize::from_bytes;
    use crate::serialize::to_bytes;
    use rust_rtps_pim::messages::submessage_elements::{
        EntityIdSubmessageElement, Parameter, ParameterListSubmessageElement,
        SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
    };

    use rust_rtps_pim::messages::submessages::DataSubmessage;
    use rust_rtps_pim::messages::types::ParameterId;
    use rust_rtps_pim::messages::{types::ProtocolId, RtpsMessageHeader};
    use rust_rtps_pim::structure::types::{EntityId, EntityKind, ProtocolVersion};

    #[test]
    fn serialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
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
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], EntityKind::UserDefinedReaderNoKey),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], EntityKind::UserDefinedReaderGroup),
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let parameter_1 = Parameter::new(ParameterId(6), &[10, 11, 12, 13]);
        let parameter_2 = Parameter::new(ParameterId(7), &[20, 21, 22, 23]);
        let parameter_list = [parameter_1, parameter_2];
        let inline_qos = ParameterListSubmessageElement {
            parameter: parameter_list.as_ref(),
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[] };

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
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };

        let expected = RtpsMessage {
            header,
            submessages: Vec::new(),
        };
        #[rustfmt::skip]
        let result: RtpsMessageRead = from_bytes(&[
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
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], EntityKind::UserDefinedReaderNoKey),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], EntityKind::UserDefinedReaderGroup),
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let parameter_1 = Parameter::new(ParameterId(6), &[10, 11, 12, 13]);
        let parameter_2 = Parameter::new(ParameterId(7), &[20, 21, 22, 23]);
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![parameter_1, parameter_2],
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[] };

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
        let result: RtpsMessageRead = from_bytes(&[
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
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], EntityKind::UserDefinedReaderNoKey),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], EntityKind::UserDefinedReaderGroup),
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let parameter_1 = Parameter::new(ParameterId(6), &[10, 11, 12, 13]);
        let parameter_2 = Parameter::new(ParameterId(7), &[20, 21, 22, 23]);
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![parameter_1, parameter_2],
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[] };

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
        let result: RtpsMessageRead = from_bytes(&[
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
