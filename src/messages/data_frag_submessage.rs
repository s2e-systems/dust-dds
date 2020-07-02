use crate::primitive_types::{UShort, ULong, };
use crate::types::{EntityId, SequenceNumber, };
use crate::types::constants::SEQUENCE_NUMBER_UNKNOWN;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsParse, RtpsCompose, Endianness, RtpsSerdesResult, };
use crate::serialized_payload::SerializedPayload;

use super::types::{SubmessageKind, SubmessageFlag, };
use super::{SubmessageHeader, Submessage, };
use super::submessage_elements::{FragmentNumber, ParameterList, };

#[derive(PartialEq, Debug)]
pub struct DataFrag {
    endianness_flag: SubmessageFlag,
    inline_qos_flag: SubmessageFlag,   
    non_standard_payload_flag: SubmessageFlag, 
    key_flag: SubmessageFlag,
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    fragment_starting_num: FragmentNumber,
    fragments_in_submessage: UShort,
    data_size: ULong,
    fragment_size: UShort,
    inline_qos: Option<ParameterList>,
    serialized_payload: Option<SerializedPayload>,
}


impl Submessage for DataFrag {
    fn submessage_header(&self) -> SubmessageHeader {
        const X: SubmessageFlag = false;
        let e = self.endianness_flag; 
        let q = self.inline_qos_flag;
        let k = self.key_flag; 
        let n = self.non_standard_payload_flag;
        let flags = [e, q, k, n, X, X, X, X];

        let mut octets_to_next_header = 4 /*extra_flags and octetsToInlineQos*/ + 
            self.reader_id.octets() + self.writer_id.octets() + self.writer_sn.octets() + 
            self.fragment_starting_num.octets() + self.fragments_in_submessage.octets() + 
            self.data_size.octets() + self.fragment_size.octets() + 
            self.serialized_payload.octets();

        if let Some(inline_qos) = &self.inline_qos {
            octets_to_next_header += inline_qos.octets();
        }

        SubmessageHeader { 
            submessage_id: SubmessageKind::Data,
            flags,
            submessage_length: octets_to_next_header as UShort,
        }
    }

    fn is_valid(&self) -> bool {
        let serialized_data_size = match &self.serialized_payload {
            Some(data) => data.0.len(),
            None => 0,
        };

        if (self.writer_sn < SequenceNumber(1) || self.writer_sn == SEQUENCE_NUMBER_UNKNOWN) ||
           (self.fragment_starting_num < FragmentNumber(1)) ||
           (self.fragment_size as u32 > self.data_size) ||
           (serialized_data_size > self.fragments_in_submessage as usize * self.fragment_size as usize)
        {
            // TODO: Check total number of fragments
            // TODO: Check validity of inline_qos
            false
        } else {
            false
        }
    }
}

impl RtpsCompose for DataFrag {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        // let sample_size = ULong(0); // TODO: what is sample_size? It is in PSM but nut in PIM. Probably: data_size
        let endianness = Endianness::from(self.endianness_flag);
        let extra_flags: UShort = 0;
        let octecs_to_inline_qos = (
            self.reader_id.octets() + 
            self.writer_id.octets() + 
            self.writer_sn.octets() + 
            self.fragment_starting_num.octets() + 
            self.fragments_in_submessage.octets() + 
            self.fragment_size.octets() + 
            self.data_size.octets()) as UShort;
        
        self.submessage_header().compose(writer)?;
        extra_flags.serialize(writer, endianness)?;
        octecs_to_inline_qos.serialize(writer, endianness)?;
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.writer_sn.serialize(writer, endianness)?;
        self.fragment_starting_num.serialize(writer, endianness)?;
        self.fragments_in_submessage.serialize(writer, endianness)?;
        self.fragment_size.serialize(writer, endianness)?;
        self.data_size.serialize(writer, endianness)?;
        if self.inline_qos_flag {
            self.inline_qos.as_ref().unwrap().serialize(writer, endianness)?;
        };
        self.serialized_payload.serialize(writer, endianness)?;
        Ok(())
    }    
}

impl RtpsParse for DataFrag {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> { 
        let header = SubmessageHeader::parse(bytes)?;
        let flags = header.flags();
        // X|X|X|X|N|K|Q|E
        /*E*/ let endianness_flag = flags[0];
        /*Q*/ let inline_qos_flag = flags[1];
        /*K*/ let key_flag = flags[2];
        /*N*/ let non_standard_payload_flag = flags[3];

        let endianness = Endianness::from(endianness_flag);

        const HEADER_SIZE : usize = 8;
        let octets_to_inline_qos = usize::from(UShort::deserialize(&bytes[6..8], endianness)?) + HEADER_SIZE /* header and extra flags*/;
        let reader_id = EntityId::deserialize(&bytes[8..12], endianness)?;        
        let writer_id = EntityId::deserialize(&bytes[12..16], endianness)?;
        let writer_sn = SequenceNumber::deserialize(&bytes[16..24], endianness)?;
        let fragment_starting_num = FragmentNumber::deserialize(&bytes[24..28], endianness)?;
        let fragments_in_submessage = UShort::deserialize(&bytes[28..30], endianness)?;
        let fragment_size = UShort::deserialize(&bytes[30..32], endianness)?;
        let data_size = ULong::deserialize(&bytes[32..36], endianness)?;


        let inline_qos = if inline_qos_flag {
            Some(ParameterList::deserialize(&bytes[octets_to_inline_qos..], endianness)?)
        } else { 
            None
        };
        let end_of_submessage = usize::from(header.submessage_length()) + header.octets();
        let octets_to_serialized_payload = octets_to_inline_qos + inline_qos.octets();
        let serialized_payload = SerializedPayload::deserialize(&bytes[octets_to_serialized_payload..end_of_submessage], endianness).ok();
  
        Ok(DataFrag {
            endianness_flag,
            inline_qos_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            fragment_starting_num,
            fragments_in_submessage,
            fragment_size,
            data_size,
            inline_qos, 
            serialized_payload, 
        })
    }
}


#[cfg(test)]
mod tests{
    use super::*;
    use crate::inline_qos_types::KeyHash;
    use crate::types::constants::{ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, };
    use crate::messages::submessage_elements::{Parameter, ParameterList, };

    #[test]
    fn parse_data_frag_submessage() {
        let endianness = Endianness::LittleEndian;
        let key_hash = KeyHash([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]);
        let inline_qos = ParameterList::new(vec![Parameter::new(&key_hash, endianness)]);
        
        let expected = DataFrag {
            endianness_flag: endianness.into(),
            inline_qos_flag: true,
            key_flag: true,
            non_standard_payload_flag: false,
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber(1),
            fragment_starting_num: FragmentNumber(1), 
            fragments_in_submessage: 2,
            fragment_size: 3,
            data_size: 4,
            inline_qos: Some(inline_qos), 
            serialized_payload: Some(SerializedPayload(vec![1, 2, 3])), 
        };

        let bytes = vec![
            0x15_u8, 0b00000111, 59, 0x0, // Submessgae Header
            0x00, 0x00,  28, 0x0, // ExtraFlags, octetsToInlineQos 
            0x00, 0x00, 0x00, 0x00, // readerId 
            0x00, 0x01, 0x00, 0xc2, // writerId
            0x00, 0x00, 0x00, 0x00, // writerSN
            0x01, 0x00, 0x00, 0x00, // writerSN
            1, 0, 0, 0,             // fragmentStartingNum
            2, 0, 3, 0,             // fragmentsInSubmessage | fragmentSize
            4, 0, 0, 0,             // sampleSize
            0x70, 0x00, 0x10, 0x00, // [Inline QoS] parameterId, length
            1, 2, 3, 4,             // [Inline QoS] Key hash
            5, 6, 7, 8,             // [Inline QoS] Key hash
            9, 10, 11, 12,          // [Inline QoS] Key hash
            13, 14, 15, 16,         // [Inline QoS] Key hash
            0x01, 0x00, 0x00, 0x00, // [Inline QoS] PID_SENTINEL
            1, 2, 3,             // [Serialized Payload]
        ];
        let result = DataFrag::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn compose_data_frag_submessage() {
        let endianness = Endianness::LittleEndian;
        let key_hash = KeyHash([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]);
        let inline_qos = ParameterList::new(vec![Parameter::new(&key_hash, endianness)]);

        let message = DataFrag {
            endianness_flag: Endianness::LittleEndian.into(),
            inline_qos_flag: true,
            key_flag: true,
            non_standard_payload_flag: false,
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber(1),
            fragment_starting_num: FragmentNumber(1), 
            fragments_in_submessage: 2,
            fragment_size: 3,
            data_size: 4,
            inline_qos: Some(inline_qos), 
            serialized_payload: Some(SerializedPayload(vec![1, 2, 3])), 
        };

        let expected = vec![
            0x15_u8, 0b00000111, 59, 0x0, // Submessgae Header
            0x00, 0x00,  28, 0x0, // ExtraFlags, octetsToInlineQos 
            0x00, 0x00, 0x00, 0x00, // readerId 
            0x00, 0x01, 0x00, 0xc2, // writerId
            0x00, 0x00, 0x00, 0x00, // writerSN
            0x01, 0x00, 0x00, 0x00, // writerSN
            1, 0, 0, 0,             // fragmentStartingNum
            2, 0, 3, 0,             // fragmentsInSubmessage | fragmentSize
            4, 0, 0, 0,             // sampleSize
            0x70, 0x00, 0x10, 0x00, // [Inline QoS] parameterId, length
            1, 2, 3, 4,             // [Inline QoS] Key hash
            5, 6, 7, 8,             // [Inline QoS] Key hash
            9, 10, 11, 12,          // [Inline QoS] Key hash
            13, 14, 15, 16,         // [Inline QoS] Key hash
            0x01, 0x00, 0x00, 0x00, // [Inline QoS] PID_SENTINEL
            1, 2, 3,             // [Serialized Payload]
        ];
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);
    }
}
