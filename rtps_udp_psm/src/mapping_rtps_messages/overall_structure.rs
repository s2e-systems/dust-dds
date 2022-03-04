use std::io::{BufRead, Error, Write};

use rust_rtps_pim::messages::overall_structure::RtpsSubmessageHeader;

use crate::{
    mapping_traits::{MappingRead, MappingWrite},
    messages::overall_structure::{
        RtpsMessageRead, RtpsMessageWrite, RtpsSubmessageTypeRead, RtpsSubmessageTypeWrite,
    },
};

use super::submessages::submessage_header::{
    ACKNACK, DATA, DATA_FRAG, GAP, HEARTBEAT, HEARTBEAT_FRAG, INFO_DST, INFO_REPLY, INFO_SRC,
    INFO_TS, NACK_FRAG, PAD,
};

impl MappingWrite for RtpsSubmessageTypeWrite<'_> {
    fn mapping_write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        match self {
            RtpsSubmessageTypeWrite::AckNack(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageTypeWrite::Data(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageTypeWrite::DataFrag(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageTypeWrite::Gap(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageTypeWrite::Heartbeat(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageTypeWrite::HeartbeatFrag(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageTypeWrite::InfoDestination(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageTypeWrite::InfoReply(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageTypeWrite::InfoSource(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageTypeWrite::InfoTimestamp(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageTypeWrite::NackFrag(s) => s.mapping_write(&mut writer)?,
            RtpsSubmessageTypeWrite::Pad(s) => s.mapping_write(&mut writer)?,
        };
        Ok(())
    }
}

impl MappingWrite for RtpsMessageWrite<'_> {
    fn mapping_write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        self.header.mapping_write(&mut writer)?;
        for submessage in &self.submessages {
            submessage.mapping_write(&mut writer)?;
        }
        Ok(())
    }
}

impl<'a, 'de: 'a> MappingRead<'de> for RtpsMessageRead<'a> {
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
                ACKNACK => RtpsSubmessageTypeRead::AckNack(MappingRead::mapping_read(buf)?),
                DATA => RtpsSubmessageTypeRead::Data(MappingRead::mapping_read(buf)?),
                DATA_FRAG => RtpsSubmessageTypeRead::DataFrag(MappingRead::mapping_read(buf)?),
                GAP => RtpsSubmessageTypeRead::Gap(MappingRead::mapping_read(buf)?),
                HEARTBEAT => RtpsSubmessageTypeRead::Heartbeat(MappingRead::mapping_read(buf)?),
                HEARTBEAT_FRAG => {
                    RtpsSubmessageTypeRead::HeartbeatFrag(MappingRead::mapping_read(buf)?)
                }
                INFO_DST => {
                    RtpsSubmessageTypeRead::InfoDestination(MappingRead::mapping_read(buf)?)
                }
                INFO_REPLY => RtpsSubmessageTypeRead::InfoReply(MappingRead::mapping_read(buf)?),
                INFO_SRC => RtpsSubmessageTypeRead::InfoSource(MappingRead::mapping_read(buf)?),
                INFO_TS => RtpsSubmessageTypeRead::InfoTimestamp(MappingRead::mapping_read(buf)?),
                NACK_FRAG => RtpsSubmessageTypeRead::NackFrag(MappingRead::mapping_read(buf)?),
                PAD => RtpsSubmessageTypeRead::Pad(MappingRead::mapping_read(buf)?),
                _ => {
                    let submessage_header: RtpsSubmessageHeader = MappingRead::mapping_read(buf)?;
                    buf.consume(submessage_header.submessage_length as usize);
                    continue;
                }
            };
            submessages.push(submessage);
        }
        Ok(Self::new(header, submessages))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::mapping_traits::{from_bytes, to_bytes};
    use crate::messages::submessage_elements::{
        EntityIdSubmessageElementPsm, ParameterListSubmessageElementRead,
        ParameterListSubmessageElementWrite, SequenceNumberSubmessageElementPsm,
        SerializedDataSubmessageElementPsm,
    };
    use crate::messages::submessages::{DataSubmessageRead, DataSubmessageWrite};
    use rust_rtps_pim::messages::overall_structure::RtpsMessageHeader;
    use rust_rtps_pim::messages::submessage_elements::{
        EntityIdSubmessageElementConstructor, Parameter, ParameterListSubmessageElement,
        ParameterListSubmessageElementConstructor, SerializedDataSubmessageElementConstructor,
    };

    use rust_rtps_pim::messages::submessage_elements::SequenceNumberSubmessageElementConstructor;
    use rust_rtps_pim::messages::submessages::DataSubmessageConstructor;
    use rust_rtps_pim::messages::types::ParameterId;
    use rust_rtps_pim::messages::types::ProtocolId;
    use rust_rtps_pim::structure::types::{
        EntityId, GuidPrefix, ProtocolVersion, USER_DEFINED_READER_GROUP,
        USER_DEFINED_READER_NO_KEY,
    };

    #[test]
    fn serialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: [9, 8],
            guid_prefix: GuidPrefix([3; 12]),
        };
        let value = RtpsMessageWrite::new(header, Vec::new());
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
            guid_prefix: GuidPrefix([3; 12]),
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id =
            EntityIdSubmessageElementPsm::new(EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY));
        let writer_id =
            EntityIdSubmessageElementPsm::new(EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP));
        let writer_sn = SequenceNumberSubmessageElementPsm::new(5);
        let parameter_1 = Parameter {
            parameter_id: ParameterId(6),
            length: 4,
            value: &[10, 11, 12, 13],
        };
        let parameter_2 = Parameter {
            parameter_id: ParameterId(7),
            length: 4,
            value: &[20, 21, 22, 23],
        };
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![parameter_1, parameter_2],
        };
        let serialized_payload = SerializedDataSubmessageElementPsm::new(&[][..]);

        let submessage = RtpsSubmessageTypeWrite::Data(DataSubmessageWrite::new(
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
        ));
        let value = RtpsMessageWrite::new(header, vec![submessage]);
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
            guid_prefix: GuidPrefix([3; 12]),
        };

        let expected = RtpsMessageRead::new(header, Vec::new());
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
            guid_prefix: GuidPrefix([3; 12]),
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdSubmessageElementPsm {
            value: EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
        };
        let writer_id = EntityIdSubmessageElementPsm {
            value: EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP),
        };
        let writer_sn = SequenceNumberSubmessageElementPsm { value: 5 };
        let parameter_1 = Parameter {
            parameter_id: ParameterId(6),
            length: 4,
            value: &[10, 11, 12, 13],
        };
        let parameter_2 = Parameter {
            parameter_id: ParameterId(7),
            length: 4,
            value: &[20, 21, 22, 23],
        };
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![parameter_1, parameter_2],
        };
        let serialized_payload = SerializedDataSubmessageElementPsm { value: &[][..] };

        let submessage = RtpsSubmessageTypeRead::Data(DataSubmessageRead::new(
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
        ));
        let expected = RtpsMessageRead::new(header, vec![submessage]);
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
            guid_prefix: GuidPrefix([3; 12]),
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdSubmessageElementPsm {
            value: EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY),
        };
        let writer_id = EntityIdSubmessageElementPsm {
            value: EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP),
        };
        let writer_sn = SequenceNumberSubmessageElementPsm { value: 5 };
        let parameter_1 = Parameter {
            parameter_id: ParameterId(6),
            length: 4,
            value: &[10, 11, 12, 13],
        };
        let parameter_2 = Parameter {
            parameter_id: ParameterId(7),
            length: 4,
            value: &[20, 21, 22, 23],
        };
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![parameter_1, parameter_2],
        };
        let serialized_payload = SerializedDataSubmessageElementPsm { value: &[][..] };

        let submessage = RtpsSubmessageTypeRead::Data(DataSubmessageRead::new(
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
        ));
        let expected = RtpsMessageRead::new(header, vec![submessage]);
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
