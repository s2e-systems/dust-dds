use std::convert::From;

use crate::primitive_types::UShort;
use crate::types::{SequenceNumber ,EntityId, };
use crate::types::constants::SEQUENCE_NUMBER_UNKNOWN;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsParse, RtpsCompose, Endianness, RtpsSerdesResult, };
use crate::serialized_payload::SerializedPayload;

use super::types::{SubmessageKind, SubmessageFlag,  };
use super::{SubmessageHeader, Submessage, };
use super::submessage_elements::{ParameterList, ParameterOps, Parameter};

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
    inline_qos: Option<ParameterList>,
    serialized_payload: Option<SerializedPayload>,
}

#[derive(PartialEq, Debug)]
pub enum Payload {
    None,
    Data(SerializedPayload),
    Key(SerializedPayload),
    NonStandard(SerializedPayload),
}


impl Data {
    /// Inline_qos_flag is inferred from option of inline_qos
    /// data_flag, key_flag and non_standard_payload_flag are inferred from the kind of payload
    pub fn new(
        endianness_flag: Endianness,
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        inline_qos: Option<&[&dyn ParameterOps]>,
        payload: Payload,) -> Self {
            let inline_qos_flag = inline_qos.is_some();
            let mut data_flag = false;
            let mut key_flag = false;
            let mut non_standard_payload_flag = false;
            let serialized_payload = match  payload {
                Payload::Data(serialized_payload) => {data_flag = true; Some(serialized_payload)},
                Payload::Key(serialized_payload) => {key_flag = true; Some(serialized_payload)},
                Payload::NonStandard(serialized_payload) => {non_standard_payload_flag = true; Some(serialized_payload)},
                Payload::None => {None}
            };

        let inline_qos = match inline_qos {
            Some(params) => Some(ParameterList::new(params.iter().map(|&p| Parameter::new(p, endianness_flag)).collect())),
            None => None,
        };

        Data {
            endianness_flag: endianness_flag.into(),
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos, 
            serialized_payload, 
        }
    }

    pub fn reader_id(&self) -> &EntityId {
        &self.reader_id
    }

    pub fn writer_id(&self) -> &EntityId {
        &self.writer_id
    }

    pub fn writer_sn(&self) -> &SequenceNumber {
        &self.writer_sn
    }

    pub fn serialized_payload(&self) -> &Option<SerializedPayload> {
        //TODO: It is a problem for the outer world to know what this payload represents
        &self.serialized_payload
    }
    
    pub fn inline_qos(&self) -> &Option<ParameterList> {
        &self.inline_qos
    }

    pub fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    pub fn data_flag(&self) -> SubmessageFlag {
        self.data_flag
    }

    pub fn key_flag(&self) -> SubmessageFlag {
        self.key_flag
    }

    pub fn non_standard_payload_flag(&self) -> SubmessageFlag {
        self.non_standard_payload_flag
    }
}

impl Submessage for Data {
    fn submessage_header(&self) -> SubmessageHeader {
        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
        let q = self.inline_qos_flag; //Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
        let d = self.data_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
        let k = self.key_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the key of the data-object. 
        let n = self.non_standard_payload_flag; //Indicates to the Reader that the serializedPayload submessage element is not formatted according to Section 10.
        // X|X|X|N|K|D|Q|E
        let flags = [e, q, d, k, n, x, x, x];

        let mut octets_to_next_header = 4 /*extra_flags and octetsToInlineQos*/ + self.reader_id.octets() + self.writer_id.octets() + self.writer_sn.octets();
        if let Some(inline_qos) = &self.inline_qos {
            octets_to_next_header += inline_qos.octets();
        }
        if let Some(serialized_payload) = &self.serialized_payload {
            octets_to_next_header += serialized_payload.octets();
        }

        SubmessageHeader { 
            submessage_id: SubmessageKind::Data,
            flags,
            submessage_length: octets_to_next_header as UShort, 
        }
    }

    fn is_valid(&self) -> bool {
        if self.writer_sn < SequenceNumber(1) || self.writer_sn == SEQUENCE_NUMBER_UNKNOWN {
            //TODO: Check validity of inline_qos
            false
        } else {
            true
        }
    }
}

impl RtpsCompose for Data {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = Endianness::from(self.endianness_flag);
        let extra_flags: UShort = 0;
        let octecs_to_inline_qos_size = self.reader_id.octets() + self.writer_id.octets() + self.writer_sn.octets();
        let octecs_to_inline_qos = octecs_to_inline_qos_size as UShort;
        self.submessage_header().compose(writer)?;
        extra_flags.serialize(writer, endianness)?;
        octecs_to_inline_qos.serialize(writer, endianness)?;
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.writer_sn.serialize(writer, endianness)?;
        // Note: No check for "Some" is needed here since this is enforced by the invariant
        if self.inline_qos_flag {
            self.inline_qos.as_ref().unwrap().serialize(writer, endianness)?;
        }
        if self.data_flag || self.key_flag {
            self.serialized_payload.as_ref().unwrap().serialize(writer, endianness)?;
        }

        Ok(())
    }    
}

impl RtpsParse for Data {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> { 
        let header = SubmessageHeader::parse(bytes)?;
        let flags = header.flags();
        // X|X|X|N|K|D|Q|E
        /*E*/ let endianness_flag = flags[0];
        /*Q*/ let inline_qos_flag = flags[1];
        /*D*/ let data_flag = flags[2];
        /*K*/ let key_flag = flags[3];
        /*N*/ let non_standard_payload_flag = flags[4];

        let endianness = Endianness::from(endianness_flag);

        const HEADER_SIZE : usize = 8;
        let octets_to_inline_qos = usize::from(UShort::deserialize(&bytes[6..8], endianness)?) + HEADER_SIZE /* header and extra flags*/;
        let reader_id = EntityId::deserialize(&bytes[8..12], endianness)?;        
        let writer_id = EntityId::deserialize(&bytes[12..16], endianness)?;
        let writer_sn = SequenceNumber::deserialize(&bytes[16..24], endianness)?;
        let inline_qos = if inline_qos_flag {
            Some(ParameterList::deserialize(&bytes[octets_to_inline_qos..], endianness)?)
        } else { 
            None
        };
        let inline_qos_octets = if let Some(inline_qos) = &inline_qos {
            inline_qos.octets()
        } else {
            0
        };
        let end_of_submessage = usize::from(header.submessage_length()) + header.octets();
        let serialized_payload = if data_flag || key_flag || non_standard_payload_flag {
            let octets_to_serialized_payload = octets_to_inline_qos + inline_qos_octets;
            SerializedPayload::deserialize(&bytes[octets_to_serialized_payload..end_of_submessage], endianness).ok()
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
    use crate::inline_qos_types::KeyHash;
    use crate::types::constants::{ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, };
    use crate::messages::submessage_elements::Parameter;

        // E: EndiannessFlag - Indicates endianness.
        // Q: InlineQosFlag - Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
        // D: DataFlag - Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
        // K: KeyFlag - Indicates to the Reader that the dataPayload submessage element contains the serialized value of the key of the data-object. 
        // N: NonStandardPayloadFlag  -Indicates to the Reader that the serializedPayload submessage element is not formatted according to Section 10.
        // X|X|X|N|K|D|Q|E
    #[test]
    fn test_data_contructor() {
        let data = Data::new(
            Endianness::LittleEndian, 
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            SequenceNumber(1), 
            Some(&[]),
            Payload::Data(SerializedPayload(vec![]))
        );
        assert_eq!(data.endianness_flag, true);
        assert_eq!(data.inline_qos_flag, true);
        assert_eq!(data.data_flag, true);
        assert_eq!(data.key_flag, false);
        assert_eq!(data.non_standard_payload_flag, false);
    }
    #[test]
    fn test_compose_data_submessage_without_inline_qos_without_data() {
        let data = Data {
            endianness_flag: true,
            inline_qos_flag: false,
            data_flag: false,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber(1),
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
        let endianness = Endianness::LittleEndian;
        let key_hash = KeyHash([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]);
        let inline_qos = ParameterList::new(vec![Parameter::new(&key_hash, endianness)]);
        
        let data = Data {
            endianness_flag: true,
            inline_qos_flag: true,
            data_flag: false,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber(1),
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
        let endianness = Endianness::LittleEndian;
        let key_hash = KeyHash([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]);
        let inline_qos = ParameterList::new(vec![Parameter::new(&key_hash, endianness)]);
        
        let serialized_payload = SerializedPayload(vec![1_u8, 2, 3]);

        let data = Data {
            endianness_flag: endianness.into(),
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber(1),
            inline_qos: Some(inline_qos), 
            serialized_payload: Some(serialized_payload), 
        };
        let expected = vec![
            0x15_u8, 0b00000111, 47, 0x0, // Submessgae Header
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
            1, 2, 3,             // [Serialized Payload]
        ];
        let mut result = Vec::new();
        data.compose(&mut result).unwrap();
        assert_eq!(expected, result);
    }


    #[test]
    fn test_parse_data_submessage_without_inline_qos_without_data() {
        let expected = Data {
            endianness_flag: true,
            inline_qos_flag: false,
            data_flag: false,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber(1),
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
            endianness_flag: true,
            inline_qos_flag: false,
            data_flag: false,
            key_flag: false,
            non_standard_payload_flag: true,
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber(1),
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
        let endianness = Endianness::LittleEndian;
        let key_hash = KeyHash([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]);
        let inline_qos = ParameterList::new(vec![Parameter::new(&key_hash, endianness)]);
        
        let serialized_payload = SerializedPayload(vec![1_u8, 2, 3]);

        let expected = Data {
            endianness_flag: endianness.into(),
            inline_qos_flag: true,
            data_flag: false,
            key_flag: true,
            non_standard_payload_flag: false,
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber(1),
            inline_qos: Some(inline_qos), 
            serialized_payload: Some(serialized_payload), 
        };
        let bytes = vec![
            0x15_u8, 0b00001011, 47, 0x0, // Submessgae Header
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
            1, 2, 3,              // [Serialized Payload]            
            99, 99, 99, 99          // Rubbish Data
        ];
        let result = Data::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }
}
