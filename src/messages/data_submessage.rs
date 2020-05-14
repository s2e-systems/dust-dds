use crate::types::{EntityId, SequenceNumber, Ushort};
use crate::inline_qos::InlineQosParameterList;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsParse, RtpsCompose, EndianessFlag, RtpsSerdesResult};

use super::SubmessageKind;

#[derive(PartialEq, Debug, Clone, Copy)]
struct SubmessageFlag(pub bool);

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
                flags |= 0b00000001 << i;
            }
        }
        writer.write(&[flags])?;
        Ok(())
    }
    fn octets(&self) -> usize { todo!() }
}

impl RtpsParse for [SubmessageFlag; 8] {
    type Output = Self;
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> { 
        // SizeCheckers::check_size_equal(bytes, 1)?;
        let flags: u8 = bytes[0];        
        let mut mask = 0b00000001_u8;
        let mut submessage_flags = [SubmessageFlag(false); 8];
        for i in 0..8 {
            if (flags & mask) > 0 {
                submessage_flags[i] = SubmessageFlag(true);
                mask = mask << 1;
            }
        };
        Ok(submessage_flags)
    }
}



#[test]
fn test_rtps_deserialize_for_submessage_flags() {
    let f = SubmessageFlag(false);
    let t = SubmessageFlag(true);
    let expected: [SubmessageFlag; 8] = [t, f, f, f, f, f, f, f];
    let bytes = [0b00000001_u8];
    
    let result = <[SubmessageFlag; 8]>::parse(&bytes).unwrap();
    assert_eq!(expected, result);
}

#[test]
fn test_rtps_serialize_for_submessage_flags() {
    let f = SubmessageFlag(false);
    let t = SubmessageFlag(true);
    let mut writer = Vec::new();

    writer.clear();
    let flags: [SubmessageFlag; 8] = [t, f, f, f, f, f, f, f];
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
    let flags: [SubmessageFlag; 8] = [f, t, f, f, t, t, f, t];
    flags.serialize(&mut writer, EndianessFlag::LittleEndian).unwrap();
    assert_eq!(writer, vec![0b10110010]);
}


#[derive(PartialEq, Debug)]
struct RepresentationIdentifier([u8; 2]);

#[derive(PartialEq, Debug)]
struct RepresentationOptions([u8; 2]);

#[derive(PartialEq, Debug)]
struct SerializedPayloadHeader {
    representation_identifier: RepresentationIdentifier,
    representation_options: RepresentationOptions,
}

// #[derive(PartialEq, Debug)]
// struct SerializedPayload {
//     header: SerializedPayloadHeader,
//     data: Vec<u8>,
// }

// impl RtpsSerialize for SerializedPayload {
//     fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> { todo!() }
//     fn octets(&self) -> usize { todo!() }
// }

// impl RtpsDeserialize for SerializedPayload {
//     fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> { 
//         todo!() 
//     }
// }

#[derive(PartialEq, Debug)]
pub struct SerializedPayload(Vec<u8>);

impl RtpsSerialize for SerializedPayload {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> { todo!() }
    fn octets(&self) -> usize { todo!() }
}

impl RtpsDeserialize for SerializedPayload {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> { 
        todo!() 
    }
}
impl RtpsParse for SerializedPayload {
    type Output = Self;
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self::Output> {
        todo!()
    }
}

#[derive(PartialEq, Debug)]
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

impl RtpsCompose for SubmessageHeader {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = EndianessFlag::from(self.flags[0].is_set());
        self.submessage_id.serialize(writer, endianness)?;
        self.flags.serialize(writer, endianness)?;
        self.submessage_length.serialize(writer, endianness)?;
        Ok(())
    }
    
    fn octets(&self) -> usize { todo!() }
}

impl RtpsParse for SubmessageHeader {
    type Output = Self;
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {   
        let submessage_id = SubmessageKind::parse(&bytes[0..1])?;
        let flags = <[SubmessageFlag; 8]>::parse(&bytes[1..2])?;
        let endianness = EndianessFlag::from(flags[0].is_set());
        let submessage_length = Ushort::deserialize(&bytes[2..4], endianness)?;
        Ok(SubmessageHeader {
            submessage_id, 
            flags,
            submessage_length,
        })
    }
}

#[test]
fn test_deserialize_submessage_header_simple() {
    let bytes = [0x15_u8, 0b00000001, 20, 0x0];
    let f = SubmessageFlag(false);
    let flags: [SubmessageFlag; 8] = [SubmessageFlag(true), f, f, f, f, f, f, f];
    let expected = SubmessageHeader {
        submessage_id : SubmessageKind::Data, 
        flags,
        submessage_length: Ushort(20),
    };
    let result = SubmessageHeader::parse(&bytes);
  
    assert_eq!(expected, result.unwrap());
}

#[test]
fn test_rtps_serialize_for_submessage_header() {
    let mut result = Vec::new();

    let f = SubmessageFlag(false);
    let t = SubmessageFlag(true);
    let header = SubmessageHeader {
        submessage_id: SubmessageKind::Data,
        flags: [t, t, f, f, f, f, f, f],
        submessage_length: Ushort(16),
    };
    let expected = vec![0x15, 0b00000011, 16, 0x0];
    header.compose(&mut result).unwrap();
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
    serialized_payload: Option<SerializedPayload>,
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
        let flags = [e, q, d, k, n, x, x, x];

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

impl RtpsCompose for Data {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = EndianessFlag::from(self.endianness_flag.is_set());
        let extra_flags = Ushort(0);
        let octecs_to_inline_qos_size = self.reader_id.octets() + self.writer_id.octets() + self.writer_sn.octets();
        let octecs_to_inline_qos = Ushort(octecs_to_inline_qos_size as u16);
        self.submessage_header().compose(writer)?;
        extra_flags.serialize(writer, endianness)?;
        octecs_to_inline_qos.serialize(writer, endianness)?;
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.writer_sn.serialize(writer, endianness)?;
        // Note: No check is "Some" needed here since this is enforced by the invariant
        if self.inline_qos_flag.is_set() {
            self.inline_qos.as_ref().unwrap().serialize(writer, endianness)?;
        }
        if self.data_flag.is_set() || self.key_flag.is_set() {
            self.serialized_payload.as_ref().unwrap().serialize(writer, endianness)?;
        }

        Ok(())
    }
    
    fn octets(&self) -> usize { todo!() }
}

impl RtpsParse for Data {
    type Output = Self;
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> { 
        let header = SubmessageHeader::parse(bytes)?;
        let flags = header.flags();
        // X|X|X|N|K|D|Q|E
        /*E*/ let endianness_flag = flags[0];
        /*Q*/ let inline_qos_flag = flags[1];
        /*D*/ let data_flag = flags[2];
        /*K*/ let key_flag = flags[3];
        /*N*/ let non_standard_payload_flag = flags[4];

        let endianness = EndianessFlag::from(endianness_flag.is_set());

        let octets_to_inline_qos = Ushort::deserialize(&bytes[6..8], endianness).unwrap().as_usize();        
        let writer_id = EntityId::deserialize(&bytes[8..12], endianness)?;
        let reader_id = EntityId::deserialize(&bytes[12..16], endianness)?;
        let writer_sn = SequenceNumber::deserialize(&bytes[16..24], endianness)?;
        let inline_qos = if inline_qos_flag.is_set() {
            Some(InlineQosParameterList::deserialize(&bytes[octets_to_inline_qos..], endianness)?)
        } else { 
            None
        };
        let inline_qos_octets = if let Some(inline_qos) = &inline_qos {
            inline_qos.octets()
        } else {
            0
        };
        let serialized_payload = if data_flag.is_set() || key_flag.is_set() {
            let octets_to_serialized_payload = octets_to_inline_qos + inline_qos_octets;
            SerializedPayload::deserialize(&bytes[octets_to_serialized_payload..], endianness).ok()
        } else {
            None
        };


        Ok(Data {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            writer_id,
            reader_id,
            writer_sn,
            inline_qos, 
            serialized_payload, 
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER};

        // E: EndiannessFlag - Indicates endianness.
        // Q: InlineQosFlag - Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
        // D: DataFlag - Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
        // K: KeyFlag - Indicates to the Reader that the dataPayload submessage element contains the serialized value of the key of the data-object. 
        // N: NonStandardPayloadFlag  -Indicates to the Reader that the serializedPayload submessage element is not formatted according to Section 10.
        // X|X|X|N|K|D|Q|E

    #[test]
    fn test_serialize_whole_simple_data_message() {
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
            0x15_u8, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];
        let mut result = Vec::new();
        data.compose(&mut result).unwrap();
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
