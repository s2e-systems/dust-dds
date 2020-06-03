use crate::types_primitives::Ushort;
use crate::types::{EntityId, SequenceNumber, };
use crate::types_other::FragmentNumberSet;
use crate::messages::types::Count;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsCompose, RtpsParse, EndianessFlag, RtpsSerdesResult, };
use super::{SubmessageKind, SubmessageFlag, Submessage, SubmessageHeader, };


#[derive(PartialEq, Debug)]
pub struct NackFrag {
    endianness_flag: SubmessageFlag,
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    fragment_number_state: FragmentNumberSet,
    count: Count,
}


impl Submessage for NackFrag {
    fn submessage_header(&self) -> SubmessageHeader {
        const X: SubmessageFlag = SubmessageFlag(false);
        let e = self.endianness_flag; 
        let flags = [e, X, X, X, X, X, X, X];

        let octets_to_next_header =  
            self.reader_id.octets() + 
            self.writer_id.octets() + 
            self.writer_sn.octets() + 
            self.fragment_number_state.octets() + 
            self.count.octets();

        SubmessageHeader { 
            submessage_id: SubmessageKind::NackFrag,
            flags,
            submessage_length: Ushort::from(octets_to_next_header),
        }
    }
}

impl RtpsCompose for NackFrag {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = EndianessFlag::from(self.endianness_flag);
       
        self.submessage_header().compose(writer)?;
        
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.writer_sn.serialize(writer, endianness)?;
        self.fragment_number_state.serialize(writer, endianness)?;
        self.count.serialize(writer, endianness)?;
        Ok(())
    }    
}

impl RtpsParse for NackFrag {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> { 
        let header = SubmessageHeader::parse(bytes)?;
        let endianness_flag = header.flags()[0];
        let endianness = EndianessFlag::from(endianness_flag);       
        let end_of_message = usize::from(header.submessage_length()) + header.octets();
        let index_count = end_of_message - 4;
        
        let reader_id = EntityId::deserialize(&bytes[4..8], endianness)?;        
        let writer_id = EntityId::deserialize(&bytes[8..12], endianness)?;
        let writer_sn = SequenceNumber::deserialize(&bytes[12..20], endianness)?;
        let fragment_number_state = FragmentNumberSet::deserialize(&bytes[20..index_count], endianness)?;
        let count = Count::deserialize(&bytes[index_count..end_of_message], endianness)?;
  
        Ok(Self {
            endianness_flag,
            reader_id,
            writer_id,
            writer_sn,
            fragment_number_state,
            count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER};
    use crate::messages::types::{FragmentNumber, };
    use crate::types_primitives::{ULong, };

    #[test]
    fn parse_nack_frag_submessage() {
        let bytes = [
            0x12, 0b00000001, 32, 0, 
            0x00, 0x00, 0x00, 0x00, // readerId 
            0x00, 0x01, 0x00, 0xc2, // writerId
            0x00, 0x00, 0x00, 0x00, // writerSN
            0x01, 0x00, 0x00, 0x00, // writerSN
            2, 0, 0, 0, // fragmentNumberState: base
            2, 0, 0, 0, // fragmentNumberState: num bits
            0b_00000000, 0b_00000000, 0b_00000000, 0b_11000000, // fragmentNumberState: bitmap
            2, 0, 0, 0, // Count
        ];
        
        let expected = NackFrag {
            endianness_flag: EndianessFlag::LittleEndian.into(),
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber(1), 
            fragment_number_state: FragmentNumberSet::new([FragmentNumber(ULong(2)), FragmentNumber(ULong(3))].iter().cloned().collect()),
            count: Count(2),
        };
        let result = NackFrag::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }
    #[test]
    fn compose_nack_frag_submessage() {
        let expected = vec![
            0x12, 0b00000001, 32, 0, 
            0x00, 0x00, 0x00, 0x00, // readerId 
            0x00, 0x01, 0x00, 0xc2, // writerId
            0x00, 0x00, 0x00, 0x00, // writerSN
            0x01, 0x00, 0x00, 0x00, // writerSN
            2, 0, 0, 0, // fragmentNumberState: base
            2, 0, 0, 0, // fragmentNumberState: num bits
            0b_00000000, 0b_00000000, 0b_00000000, 0b_11000000, // fragmentNumberState: bitmap
            2, 0, 0, 0, // Count
        ];
        
        let message = NackFrag {
            endianness_flag: EndianessFlag::LittleEndian.into(),
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber(1), 
            fragment_number_state: FragmentNumberSet::new([FragmentNumber(ULong(2)), FragmentNumber(ULong(3))].iter().cloned().collect()),
            count: Count(2),
        };
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);
    }
}
