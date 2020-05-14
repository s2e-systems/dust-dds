use crate::types::{EntityId, SequenceNumber, Ushort, KeyHash};
use crate::inline_qos::{InlineQosParameter, InlineQosParameterList};
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
            }
            mask <<= 1;
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

    let expected: [SubmessageFlag; 8] = [t, t, f, t, f, f, f, f];
    let bytes = [0b00001011_u8];    
    let result = <[SubmessageFlag; 8]>::parse(&bytes).unwrap();
    assert_eq!(expected, result);

    let expected: [SubmessageFlag; 8] = [t, t, t, t, t, t, t, t];
    let bytes = [0b11111111_u8];    
    let result = <[SubmessageFlag; 8]>::parse(&bytes).unwrap();
    assert_eq!(expected, result);

    let expected: [SubmessageFlag; 8] = [f, f, f, f, f, f, f, f];
    let bytes = [0b00000000_u8];    
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
pub struct SerializedPayload(pub Vec<u8>);

impl RtpsCompose for SerializedPayload {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> { 
        writer.write(self.0.as_slice())?;
        Ok(())
    }
    
    fn octets(&self) -> usize { self.0.len() }
}

// impl RtpsSerialize for SerializedPayload {
//     fn serialize(&self, writer: &mut impl std::io::Write, _endianness: EndianessFlag) -> RtpsSerdesResult<()> { 
//         writer.write(self.0.as_slice())?;
//         Ok(())
//     }
    
//     fn octets(&self) -> usize { self.0.len() }
// }

// impl RtpsDeserialize for SerializedPayload {
//     fn deserialize(bytes: &[u8], _endianness: EndianessFlag) -> RtpsSerdesResult<Self> { 
//         Ok(SerializedPayload(Vec::from(bytes)))
//     }
// }

impl RtpsParse for SerializedPayload {
    type Output = Self;
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self::Output> {
        Ok(SerializedPayload(Vec::from(bytes)))
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
    reader_id: EntityId,
    writer_id: EntityId,
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
            self.serialized_payload.as_ref().unwrap().compose(writer)?;
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

        let octets_to_inline_qos = Ushort::deserialize(&bytes[6..8], endianness).unwrap().as_usize() + 8 /* header and extra flags*/;
        let reader_id = EntityId::parse(&bytes[8..12])?;        
        let writer_id = EntityId::parse(&bytes[12..16])?;
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
        let serialized_payload = if data_flag.is_set() || key_flag.is_set() || non_standard_payload_flag.is_set() {
            let octets_to_serialized_payload = octets_to_inline_qos + inline_qos_octets;
            SerializedPayload::parse(&bytes[octets_to_serialized_payload..]).ok()
        } else {
            None
        };


        Ok(Data {
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
    fn test_compose_data_submessage_without_inline_qos_without_data() {
        let data = Data {
            endianness_flag: SubmessageFlag(true),
            inline_qos_flag: SubmessageFlag(false),
            data_flag: SubmessageFlag(false),
            key_flag: SubmessageFlag(false),
            non_standard_payload_flag: SubmessageFlag(false),
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
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

    #[test]
    fn test_compose_data_submessage_with_inline_qos_without_data() {
        let mut inline_qos = InlineQosParameterList::new();
        inline_qos.push(InlineQosParameter::KeyHash(KeyHash::new([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16])));
        
        let data = Data {
            endianness_flag: SubmessageFlag(true),
            inline_qos_flag: SubmessageFlag(true),
            data_flag: SubmessageFlag(false),
            key_flag: SubmessageFlag(false),
            non_standard_payload_flag: SubmessageFlag(false),
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber::new(1),
            inline_qos: Some(inline_qos), 
            serialized_payload: None, 
        };
        let expected = vec![
            0x15_u8, 0b00000011, 44, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
            0x70, 0x00, 0x10, 0x00, // [Inline QoS] parameterId, length
            1, 2, 3, 4,             // [Inline QoS] Key hash
            5, 6, 7, 8,             // [Inline QoS] Key hash
            9, 10, 11, 12,          // [Inline QoS] Key hash
            13, 14, 15, 16,         // [Inline QoS] Key hash
            0x01, 0x00, 0x00, 0x00  // [Inline QoS] PID_SENTINEL
        ];
        let mut result = Vec::new();
        data.compose(&mut result).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_compose_data_submessage_with_inline_qos_with_data() {
        let mut inline_qos = InlineQosParameterList::new();
        inline_qos.push(InlineQosParameter::KeyHash(KeyHash::new([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16])));
        
        let serialized_payload = SerializedPayload(vec![1_u8, 2, 3, 4]);

        let data = Data {
            endianness_flag: SubmessageFlag(true),
            inline_qos_flag: SubmessageFlag(true),
            data_flag: SubmessageFlag(true),
            key_flag: SubmessageFlag(false),
            non_standard_payload_flag: SubmessageFlag(false),
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber::new(1),
            inline_qos: Some(inline_qos), 
            serialized_payload: Some(serialized_payload), 
        };
        let expected = vec![
            0x15_u8, 0b00000111, 48, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
            0x70, 0x00, 0x10, 0x00, // [Inline QoS] parameterId, length
            1, 2, 3, 4,             // [Inline QoS] Key hash
            5, 6, 7, 8,             // [Inline QoS] Key hash
            9, 10, 11, 12,          // [Inline QoS] Key hash
            13, 14, 15, 16,         // [Inline QoS] Key hash
            0x01, 0x00, 0x00, 0x00, // [Inline QoS] PID_SENTINEL
            1, 2, 3, 4,             // [Serialized Payload]
        ];
        let mut result = Vec::new();
        data.compose(&mut result).unwrap();
        assert_eq!(expected, result);
    }


    #[test]
    fn test_parse_data_submessage_without_inline_qos_without_data() {
        let expected = Data {
            endianness_flag: SubmessageFlag(true),
            inline_qos_flag: SubmessageFlag(false),
            data_flag: SubmessageFlag(false),
            key_flag: SubmessageFlag(false),
            non_standard_payload_flag: SubmessageFlag(false),
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber::new(1),
            inline_qos: None, 
            serialized_payload: None, 
        };
        let bytes = vec![
            0x15_u8, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];
        let result = Data::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_data_submessage_without_inline_qos_with_non_standard_payload() {       
        let serialized_payload = SerializedPayload(vec![1_u8, 2, 3, 4]);

        let expected = Data {
            endianness_flag: SubmessageFlag(true),
            inline_qos_flag: SubmessageFlag(false),
            data_flag: SubmessageFlag(false),
            key_flag: SubmessageFlag(false),
            non_standard_payload_flag: SubmessageFlag(true),
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber::new(1),
            inline_qos: None, 
            serialized_payload: Some(serialized_payload), 
        };
        let bytes = vec![
            0x15_u8, 0b00010001, 24, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
            1, 2, 3, 4,             // [Serialized Payload]
        ];
        let result = Data::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_data_submessage_with_inline_qos_with_data() {
        let mut inline_qos = InlineQosParameterList::new();
        inline_qos.push(InlineQosParameter::KeyHash(KeyHash::new([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16])));
        
        let serialized_payload = SerializedPayload(vec![1_u8, 2, 3, 4]);

        let expected = Data {
            endianness_flag: SubmessageFlag(true),
            inline_qos_flag: SubmessageFlag(true),
            data_flag: SubmessageFlag(false),
            key_flag: SubmessageFlag(true),
            non_standard_payload_flag: SubmessageFlag(false),
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber::new(1),
            inline_qos: Some(inline_qos), 
            serialized_payload: Some(serialized_payload), 
        };
        let bytes = vec![
            0x15_u8, 0b00001011, 48, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
            0x70, 0x00, 0x10, 0x00, // [Inline QoS] parameterId, length
            1, 2, 3, 4,             // [Inline QoS] Key hash
            5, 6, 7, 8,             // [Inline QoS] Key hash
            9, 10, 11, 12,          // [Inline QoS] Key hash
            13, 14, 15, 16,         // [Inline QoS] Key hash
            0x01, 0x00, 0x00, 0x00, // [Inline QoS] PID_SENTINEL
            1, 2, 3, 4,             // [Serialized Payload]
        ];
        let result = Data::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }
}
