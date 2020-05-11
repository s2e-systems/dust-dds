use crate::types::{EntityId, SequenceNumber, Ushort};
use crate::inline_qos::InlineQosParameterList;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, EndianessFlag, RtpsSerdesResult, PrimitiveSerdes};

use super::SubmessageKind;

#[derive(PartialEq, Debug, Clone, Copy)]
struct SubmessageFlag(bool);

impl SubmessageFlag {
    pub fn is_set(&self) -> bool {
         self.0
    }
}

impl RtpsSerialize for [SubmessageFlag; 8] {
    fn serialize(&self, writer: &mut impl std::io::Write, _endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        let mut flags = 0u8;
        for i in 0..8 {
            if self[i].0 {
                flags |= 0b10000000 >> i;
            }
        }
        writer.write(&[flags])?;
        Ok(())
    }
    fn octets(&self) -> usize { todo!() }
}

#[test]
fn test_rtps_serialize_for_submessage_flags() {
    let f = SubmessageFlag(false);
    let t = SubmessageFlag(true);
    let mut writer = Vec::new();

    writer.clear();
    let flags: [SubmessageFlag; 8] = [f, f, f, f, f, f, f, t];
    flags.serialize(&mut writer, EndianessFlag::LittleEndian).unwrap();
    assert_eq!(writer, vec![0b00000001]);
    
    writer.clear();
    let flags: [SubmessageFlag; 8] = [f; 8];
    flags.serialize(&mut writer, EndianessFlag::LittleEndian).unwrap();
    assert_eq!(writer, vec![0b00000000]);
    
    writer.clear();
    let flags: [SubmessageFlag; 8] = [t; 8];
    flags.serialize(&mut writer, EndianessFlag::LittleEndian).unwrap();
    assert_eq!(writer, vec![0b11111111]);
    
    writer.clear();
    let flags: [SubmessageFlag; 8] = [t, f, t, t, f, f, t, f];
    flags.serialize(&mut writer, EndianessFlag::LittleEndian).unwrap();
    assert_eq!(writer, vec![0b10110010]);
}

#[derive(PartialEq, Debug)]
pub enum Payload {
    None,
    Data(Vec<u8>),
    Key(Vec<u8>),
    NonStandard(Vec<u8>),
}

impl RtpsSerialize for Payload {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> { todo!() }
    fn octets(&self) -> usize { todo!() }
}


struct SubmessageHeader {
    submessage_id: SubmessageKind,
    flags: [SubmessageFlag; 8],
    submessage_length: Ushort,
}

impl SubmessageHeader {
    pub fn submessage_id(&self) -> SubmessageKind {self.submessage_id}
    pub fn flags(&self) -> &[SubmessageFlag; 8] {&self.flags}
    pub fn submessage_length(&self) -> Ushort {self.submessage_length}
}

impl RtpsSerialize for SubmessageHeader
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        self.submessage_id.serialize(writer, endianness)?;
        self.flags.serialize(writer, endianness)?;
        self.submessage_length.serialize(writer, endianness)?;
        Ok(())
    }
    
    fn octets(&self) -> usize { todo!() }
}

#[test]
fn test_rtps_serialize_for_submessage_header() {
    let mut result = Vec::new();

    let f = SubmessageFlag(false);
    let t = SubmessageFlag(true);
    let header = SubmessageHeader {
        submessage_id: SubmessageKind::Data,
        flags: [f, f, f, f, f, f, t, f],
        submessage_length: Ushort(16),
    };
    let expected = vec![0x15, 0b00000010, 16, 0x0];
    header.serialize(&mut result, EndianessFlag::LittleEndian).unwrap();
    assert_eq!(result, expected);
}

#[derive(PartialEq, Debug)]
pub struct Data {
    endianness_flag: SubmessageFlag,
    inline_qos_flag: SubmessageFlag,    
    data_flag: SubmessageFlag, 
    key_flag: SubmessageFlag,
    non_standard_payload_flag: SubmessageFlag,
    writer_id: EntityId,
    reader_id: EntityId,
    writer_sn: SequenceNumber,
    inline_qos: Option<InlineQosParameterList>,
    serialized_payload: Option<Payload>,
}

impl Data {
    fn submessage_header(&self) -> SubmessageHeader {
        let x = SubmessageFlag(false);
        let e = self.endianness_flag; // Indicates endianness.
        let q = self.inline_qos_flag; //Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
        let d = self.data_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
        let k = self.key_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the key of the data-object. 
        let n = self.non_standard_payload_flag; //Indicates to the Reader that the serializedPayload submessage element is not formatted according to Section 10.
        // X|X|X|N|K|D|Q|E
        let flags = [x, x, x, n, k, d, q, e];

        let mut octets_to_next_header = 4 /*extra_flags and octetsToInlineQos*/  + self.reader_id.octets() + self.writer_id.octets() + self.writer_sn.octets();
        if let Some(inline_qos) = &self.inline_qos {
            octets_to_next_header += inline_qos.octets();
        }
        if let Some(serialized_payload) = &self.serialized_payload {
            octets_to_next_header += serialized_payload.octets();
        }

        SubmessageHeader { 
            submessage_id: SubmessageKind::Data,
            flags,
            submessage_length: Ushort(octets_to_next_header as u16), //todo
        }
    }

    fn is_valid(&self) -> bool { 
        let sequencenumber_unknown: SequenceNumber = SequenceNumber::new(-1); // Todo: should be "global" constant      
        let submessage_header_is_too_small = false; //self.submessage_header().submessage_length() < self.length(); // Todo!
        let writer_sn_value_is_not_strictly_positive = self.writer_sn < SequenceNumber::new(1) || self.writer_sn == sequencenumber_unknown;
        let inline_qos_is_invalid = match &self.inline_qos {
            None => false,
            Some(inline_qos) => inline_qos.is_valid()
        };
        submessage_header_is_too_small && writer_sn_value_is_not_strictly_positive && inline_qos_is_invalid
    }
}

impl RtpsSerialize for Data {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        let extra_flags = Ushort(0);
        let octecs_to_inline_qos_size = self.reader_id.octets() + self.writer_id.octets() + self.writer_sn.octets();
        let octecs_to_inline_qos = Ushort(octecs_to_inline_qos_size as u16);
        self.submessage_header().serialize(writer, endianness)?;
        extra_flags.serialize(writer, endianness)?;
        octecs_to_inline_qos.serialize(writer, endianness)?;
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.writer_sn.serialize(writer, endianness)?;
        if self.inline_qos_flag.is_set() {
            if let Some(inline_qos) = &self.inline_qos {
                inline_qos.serialize(writer, endianness)?;
            }
        }
        if self.data_flag.is_set() || self.key_flag.is_set() {
            if let Some(serialized_payload) = &self.serialized_payload {
                serialized_payload.serialize(writer, endianness)?;
            }
        }

        Ok(())
    }
    
    fn octets(&self) -> usize { todo!() }
}

// impl RtpsDeserialize for Data {
//     const INLINE_QOS_FLAG_MASK: u8 = 0x02;
//     const DATA_FLAG_MASK: u8 = 0x04;
//     const KEY_FLAG_MASK: u8 = 0x08;
//     const NON_STANDARD_PAYLOAD_FLAG_MASK: u8 = 0x10;
// }


#[cfg(test)]
mod tests {
    use super::*;
    // use crate::messages::InlineQosParameter;
    use crate::types::constants::{ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER};
    // use cdr::{Infinite, LittleEndian};

    #[test]
    fn test_serialize_whole_simple_data_message() {

        // E: EndiannessFlag - Indicates endianness.
        // Q: InlineQosFlag - Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
        // D: DataFlag - Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
        // K: KeyFlag - Indicates to the Reader that the dataPayload submessage element contains the serialized value of the key of the data-object. 
        // N: NonStandardPayloadFlag  -Indicates to the Reader that the serializedPayload submessage element is not formatted according to Section 10.
        // X|X|X|N|K|D|Q|E

        let data = Data {
            endianness_flag: SubmessageFlag(true),
            inline_qos_flag: SubmessageFlag(false),
            data_flag: SubmessageFlag(false),
            key_flag: SubmessageFlag(false),
            non_standard_payload_flag: SubmessageFlag(false),
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            reader_id: ENTITYID_UNKNOWN,
            writer_sn: SequenceNumber::new(1),
            inline_qos: None, 
            serialized_payload: None, 
        };
        let expected = vec![
            0x15, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];
        let mut result = Vec::new();
        data.serialize(&mut result, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(expected, result);
    }


    // const DATA_SUBMESSAGE_BIG_ENDIAN: [u8; 60] = [
    //     0x00, 0x00, 0x00, 0x10, 0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00,
    //     0x00, 0x00, 0x00, 0x04, 0xD1, 0x00, 0x70, 0x00, 0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
    //     0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x00, 0x10, 0x00, 0x08, 0x10,
    //     0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x00, 0x01, 0x00, 0x00, 0x20, 0x30, 0x40, 0x50,
    // ];

    // #[test]
    // fn test_parse_data_submessage_without_inline_qos_or_data() {
    //     let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &0).unwrap();
    //     assert_eq!(data.reader_id, EntityId::new([0x10, 0x12, 0x14], 0x16));
    //     assert_eq!(data.writer_id, EntityId::new([0x26, 0x24, 0x22], 0x20));
    //     assert_eq!(data.writer_sn, 1233);
    //     assert_eq!(data.inline_qos, None);
    //     assert_eq!(data.serialized_payload, Payload::None);
    // }

    // #[test]
    // fn test_parse_data_submessage_with_inline_qos_without_data() {
    //     let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &2).unwrap();
    //     assert_eq!(data.reader_id, EntityId::new([0x10, 0x12, 0x14], 0x16));
    //     assert_eq!(data.writer_id, EntityId::new([0x26, 0x24, 0x22], 0x20));
    //     assert_eq!(data.writer_sn, 1233);
    //     assert_eq!(data.serialized_payload, Payload::None);
    //     let inline_qos = data.inline_qos.unwrap();
    //     assert_eq!(inline_qos.len(), 1);
    //     assert_eq!(
    //         inline_qos[0],
    //         InlineQosParameter::KeyHash([
    //             0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
    //             0x0F, 0x10,
    //         ])
    //     );
    // }

    // #[test]
    // fn test_parse_data_submessage_with_inline_qos_and_data() {
    //     let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &6).unwrap();
    //     assert_eq!(data.reader_id, EntityId::new([0x10, 0x12, 0x14], 0x16));
    //     assert_eq!(data.writer_id, EntityId::new([0x26, 0x24, 0x22], 0x20));
    //     assert_eq!(data.writer_sn, 1233);
    //     let inline_qos = data.inline_qos.unwrap();
    //     assert_eq!(inline_qos.len(), 1);
    //     assert_eq!(
    //         inline_qos[0],
    //         InlineQosParameter::KeyHash([
    //             0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
    //             0x0F, 0x10,
    //         ])
    //     );
    //     if let Payload::Data(serialized_data) = data.serialized_payload {
    //         assert_eq!(serialized_data, vec!(0x20, 0x30, 0x40, 0x50,));
    //     } else {
    //         assert!(false);
    //     }
    // }

    // #[test]
    // fn test_parse_data_submessage_with_inline_qos_and_serialized_key() {
    //     let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &10).unwrap();
    //     assert_eq!(data.reader_id, EntityId::new([0x10, 0x12, 0x14], 0x16));
    //     assert_eq!(data.writer_id, EntityId::new([0x26, 0x24, 0x22], 0x20));
    //     assert_eq!(data.writer_sn, 1233);
    //     let inline_qos = data.inline_qos.unwrap();
    //     assert_eq!(inline_qos.len(), 1);
    //     assert_eq!(
    //         inline_qos[0],
    //         InlineQosParameter::KeyHash([
    //             0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
    //             0x0F, 0x10,
    //         ])
    //     );
    //     if let Payload::Key(serialized_data) = data.serialized_payload {
    //         assert_eq!(serialized_data, vec!(0x20, 0x30, 0x40, 0x50,));
    //     } else {
    //         assert!(false);
    //     }
    // }

    // #[test]
    // fn test_parse_data_submessage_without_inline_qos_with_serialized_data() {
    //     // Parse message considering serialized data and no inline qos
    //     let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &8).unwrap();
    //     assert_eq!(data.reader_id, EntityId::new([0x10, 0x12, 0x14], 0x16));
    //     assert_eq!(data.writer_id, EntityId::new([0x26, 0x24, 0x22], 0x20));
    //     assert_eq!(data.writer_sn, 1233);
    //     assert_eq!(data.inline_qos, None);
    //     if let Payload::Key(serialized_data) = data.serialized_payload {
    //         assert_eq!(
    //             serialized_data,
    //             vec!(
    //                 0x00, 0x70, 0x00, 0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
    //                 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x00, 0x10, 0x00, 0x08, 0x10, 0x11,
    //                 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x00, 0x01, 0x00, 0x00, 0x20, 0x30, 0x40,
    //                 0x50,
    //             )
    //         );
    //     } else {
    //         assert!(false);
    //     }
    // }
    
    // #[test]
    // fn test_serialize_data_message() {
    //     let expected_serialized_data = 
    //     vec![0x15, 0x07, 0x1c, 0x01, //032 submessageId: SubmessageKind => DATA | flags: SubmessageFlag[8] => N=0|K=0|D=1|Q=1|E=1 Endianess=little && InlineQosFlag && serializedPayload contains data | submessageLength (octetsToNextHeader): ushort => 284
    //         0x00, 0x00, 0x10, 0x00, //036  [Data Submessage] Flags: extraFlags | octetsToInlineQos: ushort => 16
    //         0x00, 0x00, 0x00, 0x00, //040  [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
    //         0x00, 0x01, 0x00, 0xc2, //044  [Data Submessage] EntityId writerId => ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER ([0, 0x01, 0x00], ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY)
    //         0x00, 0x00, 0x00, 0x00, //048  [Data Submessage] SequenceNumber writerSN 
    //         0x01, 0x00, 0x00, 0x00, //052  [Data Submessage] SequenceNumber writerSN => 1
    //         0x70, 0x00, 0x10, 0x00, //056  [Data Submessage: inLineQos] parameterId_1: short => PID_KEY_HASH | length: short => 16
    //         0x7f, 0x20, 0xf7, 0xd7, //060  [Data Submessage: inLineQos: KEY_HASH] 
    //         0x00, 0x00, 0x01, 0xbb, //064  [Data Submessage: inLineQos: KEY_HASH] 
    //         0x00, 0x00, 0x00, 0x01, //068  [Data Submessage: inLineQos: KEY_HASH]  
    //         0x00, 0x00, 0x01, 0xc1, //072  [Data Submessage: inLineQos: KEY_HASH]  
    //         0x01, 0x00, 0x00, 0x00, //076  [Data Submessage]  parameterId_1: short => PID_SENTINEL | 0
    //         0x00, 0x03, 0x00, 0x00, //080  [Data Submessage: SerializedPayload]   representation_identifier: octet[2] => PL_CDR_LE | representation_options: octet[2] => none
    //         0x15, 0x00, 0x04, 0x00, //084  [Data Submessage: SerializedPayload]   parameterId_1: short => PID_PROTOCOL_VERSION | length: short => 4
    //         0x02, 0x01, 0x00, 0x00, //088  [Data Submessage: SerializedPayload: PID_PROTOCOL_VERSION]  major: octet => 2 | minor: octet =>1 | padding 
    //         0x16, 0x00, 0x04, 0x00, //092  [Data Submessage: SerializedPayload]  parameterId_1: short => PID_VENDORID  | length: short => 4
    //         0x01, 0x02, 0x00, 0x00, //096  [Data Submessage: SerializedPayload: PID_VENDORID] vendorId: octet[2] => 12
    //         0x31, 0x00, 0x18, 0x00, //100  [Data Submessage: SerializedPayload]  parameterId_1: short =>  PID_DEFAULT_UNICAST_LOCATOR | length: short => 24
    //         0x01, 0x00, 0x00, 0x00, //104  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0xf3, 0x1c, 0x00, 0x00, //108  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x00, 0x00, 0x00, 0x00, //112  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x00, 0x00, 0x00, 0x00, //116  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]  
    //         0x00, 0x00, 0x00, 0x00, //120  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0xc0, 0xa8, 0x02, 0x04, //124  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]
    //         0x32, 0x00, 0x18, 0x00, //128  [Data Submessage: SerializedPayload] parameterId_1: short => PID_METATRAFFIC_UNICAST_LOCATOR | length: short => 24
    //         0x01, 0x00, 0x00, 0x00, //132  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0xf2, 0x1c, 0x00, 0x00, //136  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x00, 0x00, 0x00, 0x00, //140  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x00, 0x00, 0x00, 0x00, //144  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]  
    //         0x00, 0x00, 0x00, 0x00, //148  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0xc0, 0xa8, 0x02, 0x04, //152  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
    //         0x02, 0x00, 0x08, 0x00, //156  [Data Submessage: SerializedPayload] parameterId_1: short => PID_PARTICIPANT_LEASE_DURATION | length: short => 8
    //         0x0b, 0x00, 0x00, 0x00, //160  [Data Submessage: SerializedPayload: PID_PARTICIPANT_LEASE_DURATION] seconds: long => 11 
    //         0x00, 0x00, 0x00, 0x00, //164  [Data Submessage: SerializedPayload: PID_PARTICIPANT_LEASE_DURATION] fraction: ulong => 0    
    //         0x50, 0x00, 0x10, 0x00, //168  [Data Submessage: SerializedPayload] parameterId_1: short => PID_PARTICIPANT_GUID | length: short => 16
    //         0x7f, 0x20, 0xf7, 0xd7, //172  [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID] 
    //         0x00, 0x00, 0x01, 0xbb, //176  [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID]   
    //         0x00, 0x00, 0x00, 0x01, //180  [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID]   
    //         0x00, 0x00, 0x01, 0xc1, //184  [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID]   
    //         0x58, 0x00, 0x04, 0x00, //188  [Data Submessage: SerializedPayload] parameterId_1: short => PID_BUILTIN_ENDPOINT_SET | length: short => 4
    //         0x15, 0x04, 0x00, 0x00, //192  [Data Submessage: SerializedPayload: PID_BUILTIN_ENDPOINT_SET] BuiltinEndpointSet: bitmask => (0100 0001 0101‬) PARTICIPANT_ANNOUNCER && PUBLICATIONS_ANNOUNCER && SUBSCRIPTIONS_ANNOUNCER && PARTICIPANT_MESSAGE_DATA_WRITER
    //         0x00, 0x80, 0x04, 0x00, //196  [Data Submessage: SerializedPayload] parameterId_1: short => Vendor-specific ParameterId (0x8000) | length: short => 4   
    //         0x15, 0x00, 0x00, 0x00, //200  [Data Submessage: SerializedPayload: Vendor-specific 0x0]  
    //         0x07, 0x80, 0x5c, 0x00, //204  [Data Submessage: SerializedPayload] parameterId_1: short => Vendor-specific ParameterId (0x8007) | length: short => 92     
    //         0x00, 0x00, 0x00, 0x00, //208  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x2f, 0x00, 0x00, 0x00, //212  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x05, 0x00, 0x00, 0x00, //216  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x00, 0x00, 0x00, 0x00, //220  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x50, 0x00, 0x00, 0x00, //224  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x42, 0x00, 0x00, 0x00, //228  [Data Submessage: SerializedPayload: Vendor-specific 0x7]  
    //         0x44, 0x45, 0x53, 0x4b, //232  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x54, 0x4f, 0x50, 0x2d, //236  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x4f, 0x52, 0x46, 0x44, //240  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x4f, 0x53, 0x35, 0x2f, //244  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x36, 0x2e, 0x31, 0x30, //248  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x2e, 0x32, 0x2f, 0x63, //252  [Data Submessage: SerializedPayload: Vendor-specific 0x7]  
    //         0x63, 0x36, 0x66, 0x62, //256  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x39, 0x61, 0x62, 0x33, //260  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x36, 0x2f, 0x39, 0x30, //264  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x37, 0x65, 0x66, 0x66, //268  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x30, 0x32, 0x65, 0x33, //272  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x2f, 0x22, 0x78, 0x38, //276  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x36, 0x5f, 0x36, 0x34, //280  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x2e, 0x77, 0x69, 0x6e, //284  [Data Submessage: SerializedPayload: Vendor-specific 0x7]  
    //         0x2d, 0x76, 0x73, 0x32, //288  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x30, 0x31, 0x35, 0x22, //292  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x2f, 0x00, 0x00, 0x00, //296  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
    //         0x25, 0x80, 0x0c, 0x00, //300  [Data Submessage: SerializedPayload] parameterId_1: short => Vendor-specific ParameterId (0x8025) | length: short => 12       
    //         0xd7, 0xf7, 0x20, 0x7f, //304  [Data Submessage: SerializedPayload: Vendor-specific ParameterId 0x25]   
    //         0xbb, 0x01, 0x00, 0x00, //308  [Data Submessage: SerializedPayload: Vendor-specific ParameterId 0x25]   
    //         0x01, 0x00, 0x00, 0x00, //312  [Data Submessage: SerializedPayload: Vendor-specific ParameterId 0x25]  
    //         0x01, 0x00, 0x00, 0x00, //316  [Data Submessage: SerializedPayload] parameterId_1: short => PID_SENTINEL |  length: short => 0
    //     ];

    //     let payload = vec![0x00, 0x03, 0x00, 0x00,
    //     0x15, 0x00, 0x04, 0x00,
    //     0x02, 0x01, 0x00, 0x00,
    //     0x16, 0x00, 0x04, 0x00,
    //     0x01, 0x02, 0x00, 0x00,
    //     0x31, 0x00, 0x18, 0x00,
    //     0x01, 0x00, 0x00, 0x00,
    //     0xf3, 0x1c, 0x00, 0x00,
    //     0x00, 0x00, 0x00, 0x00,
    //     0x00, 0x00, 0x00, 0x00,
    //     0x00, 0x00, 0x00, 0x00,
    //     0xc0, 0xa8, 0x02, 0x04,
    //     0x32, 0x00, 0x18, 0x00,
    //     0x01, 0x00, 0x00, 0x00,
    //     0xf2, 0x1c, 0x00, 0x00,
    //     0x00, 0x00, 0x00, 0x00,
    //     0x00, 0x00, 0x00, 0x00,
    //     0x00, 0x00, 0x00, 0x00,
    //     0xc0, 0xa8, 0x02, 0x04,
    //     0x02, 0x00, 0x08, 0x00,
    //     0x0b, 0x00, 0x00, 0x00,
    //     0x00, 0x00, 0x00, 0x00,
    //     0x50, 0x00, 0x10, 0x00,
    //     0x7f, 0x20, 0xf7, 0xd7,
    //     0x00, 0x00, 0x01, 0xbb,
    //     0x00, 0x00, 0x00, 0x01,
    //     0x00, 0x00, 0x01, 0xc1,
    //     0x58, 0x00, 0x04, 0x00,
    //     0x15, 0x04, 0x00, 0x00,
    //     0x00, 0x80, 0x04, 0x00,
    //     0x15, 0x00, 0x00, 0x00,
    //     0x07, 0x80, 0x5c, 0x00,
    //     0x00, 0x00, 0x00, 0x00,
    //     0x2f, 0x00, 0x00, 0x00,
    //     0x05, 0x00, 0x00, 0x00,
    //     0x00, 0x00, 0x00, 0x00,
    //     0x50, 0x00, 0x00, 0x00,
    //     0x42, 0x00, 0x00, 0x00,
    //     0x44, 0x45, 0x53, 0x4b,
    //     0x54, 0x4f, 0x50, 0x2d,
    //     0x4f, 0x52, 0x46, 0x44,
    //     0x4f, 0x53, 0x35, 0x2f,
    //     0x36, 0x2e, 0x31, 0x30,
    //     0x2e, 0x32, 0x2f, 0x63,
    //     0x63, 0x36, 0x66, 0x62,
    //     0x39, 0x61, 0x62, 0x33,
    //     0x36, 0x2f, 0x39, 0x30,
    //     0x37, 0x65, 0x66, 0x66,
    //     0x30, 0x32, 0x65, 0x33,
    //     0x2f, 0x22, 0x78, 0x38,
    //     0x36, 0x5f, 0x36, 0x34,
    //     0x2e, 0x77, 0x69, 0x6e,
    //     0x2d, 0x76, 0x73, 0x32,
    //     0x30, 0x31, 0x35, 0x22,
    //     0x2f, 0x00, 0x00, 0x00,
    //     0x25, 0x80, 0x0c, 0x00,
    //     0xd7, 0xf7, 0x20, 0x7f,
    //     0xbb, 0x01, 0x00, 0x00,
    //     0x01, 0x00, 0x00, 0x00,
    //     0x01, 0x00, 0x00, 0x00,];

    //     let data = Data::new(
    //         ENTITYID_UNKNOWN, /*reader_id*/
    //         ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, /*writer_id*/
    //         [0;16], /*key_hash*/
    //         1, /*writer_sn*/
    //         Some(InlineQosParameterList::new_from_vec(vec![InlineQosParameter::KeyHash([0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00, 0x01, 0xbb,0x00, 0x00, 0x00, 0x01,0x00, 0x00, 0x01, 0xc1])])), /*inline_qos*/
    //         Payload::Data(payload) /*serialized_payload*/);
        
    //     let serialized_data = cdr::ser::serialize_data::<_,_,LittleEndian>(&data, Infinite).unwrap();
    //     println!("Data: {:x?}", serialized_data);

    //     assert_eq!(serialized_data, expected_serialized_data);
    // }
}
