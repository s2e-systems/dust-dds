use crate::serdes::{SubmessageElement, RtpsCompose, RtpsParse, Endianness, RtpsSerdesResult, };
use super::{SubmessageKind, SubmessageFlag, Submessage, SubmessageHeader, };
use super::submessage_elements;


#[derive(PartialEq, Debug)]
pub struct NackFrag {
    endianness_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId,
    writer_id: submessage_elements::EntityId,
    writer_sn: submessage_elements::SequenceNumber,
    fragment_number_state: submessage_elements::FragmentNumberSet,
    count: submessage_elements::Count,
}


impl Submessage for NackFrag {
    fn submessage_header(&self) -> SubmessageHeader {
        const X: SubmessageFlag = false;
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
            submessage_length: octets_to_next_header as u16,
        }
    }

    fn is_valid(&self) -> bool {
        if self.writer_sn.0 <= 0 ||
        !self.fragment_number_state.is_valid() {
            false
        } else {
            true
        }
    }

    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = Endianness::from(self.endianness_flag);
       
        self.submessage_header().compose(writer)?;
        
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.writer_sn.serialize(writer, endianness)?;
        self.fragment_number_state.serialize(writer, endianness)?;
        self.count.serialize(writer, endianness)?;
        Ok(())
    }

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> { 
        let header = SubmessageHeader::parse(bytes)?;
        let endianness_flag = header.flags()[0];
        let endianness = Endianness::from(endianness_flag);       
        let end_of_message = usize::from(header.submessage_length()) + header.octets();
        let index_count = end_of_message - 4;
        
        let reader_id = submessage_elements::EntityId::deserialize(&bytes[4..8], endianness)?;        
        let writer_id = submessage_elements::EntityId::deserialize(&bytes[8..12], endianness)?;
        let writer_sn = submessage_elements::SequenceNumber::deserialize(&bytes[12..20], endianness)?;
        let fragment_number_state = submessage_elements::FragmentNumberSet::deserialize(&bytes[20..index_count], endianness)?;
        let count = submessage_elements::Count::deserialize(&bytes[index_count..end_of_message], endianness)?;
  
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
    use super::super::submessage_elements::FragmentNumber;

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
            endianness_flag: Endianness::LittleEndian.into(),
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN), 
            writer_id: submessage_elements::EntityId(ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER), 
            writer_sn: submessage_elements::SequenceNumber(1),
            fragment_number_state: submessage_elements::FragmentNumberSet::new([FragmentNumber(2), FragmentNumber(3)].iter().cloned().collect()),
            count: submessage_elements::Count(2),
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
            endianness_flag: Endianness::LittleEndian.into(),
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN), 
            writer_id: submessage_elements::EntityId(ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER), 
            writer_sn: submessage_elements::SequenceNumber(1),
            fragment_number_state: submessage_elements::FragmentNumberSet::new([FragmentNumber(2), FragmentNumber(3)].iter().cloned().collect()),
            count: submessage_elements::Count(2),
        };
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);
    }
}
