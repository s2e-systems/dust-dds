use crate::types_primitives::Ushort;
use crate::types::{EntityId, SequenceNumberSet, };
use crate::messages::types::{Count, SubmessageKind, SubmessageFlag, };
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsParse, RtpsCompose, EndianessFlag, RtpsSerdesResult, };
use super::{SubmessageHeader, Submessage, };

#[derive(PartialEq, Debug)]
pub struct AckNack {
    endianness_flag: SubmessageFlag,
    final_flag: SubmessageFlag,
    reader_id: EntityId,
    writer_id: EntityId,
    reader_sn_state: SequenceNumberSet,
    count: Count,
}

impl Submessage for AckNack {
    fn submessage_header(&self) -> SubmessageHeader {
        const X : SubmessageFlag = SubmessageFlag(false);
        let e = self.endianness_flag; 
        let f = self.final_flag; 
        let flags = [e, f, X, X, X, X, X, X];     
        let submessage_length = self.reader_id.octets() + self.writer_id.octets() + self.reader_sn_state.octets() + self.count.octets();
        SubmessageHeader { 
            submessage_id: SubmessageKind::InfoReply,
            flags,
            submessage_length: Ushort::from(submessage_length),
        }
    }    
}

impl RtpsCompose for AckNack {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = EndianessFlag::from(self.endianness_flag);       
        self.submessage_header().compose(writer)?;
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.reader_sn_state.serialize(writer, endianness)?;
        self.count.serialize(writer, endianness)?;        
        Ok(())
    }
}

impl RtpsParse for AckNack {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let header = SubmessageHeader::parse(bytes)?;
        let endianness_flag = header.flags()[0];
        let final_flag = header.flags()[1];
        let endianness = endianness_flag.into();
        let end_of_message = usize::from(header.submessage_length()) + header.octets();
        let index_count = end_of_message - 4;
        let reader_id = EntityId::deserialize(&bytes[4..8], endianness)?;
        let writer_id = EntityId::deserialize(&bytes[8..12], endianness)?;
        let reader_sn_state = SequenceNumberSet::deserialize(&bytes[12..index_count], endianness)?;
        let count = Count::deserialize(&bytes[index_count..end_of_message], endianness)?;
        
        Ok(Self{endianness_flag, final_flag, reader_id, writer_id, reader_sn_state, count})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, };
    use crate::types::SequenceNumber;
    
    #[test]
    fn test_parse_ack_nack_submessage() {
        let bytes = [
            0x0f, 0b00000011, 28, 0, 
            0x00, 0x00, 0x00, 0x00, // readerId 
            0x00, 0x01, 0x00, 0xc2, // writerId
            0, 0, 0, 0, // reader_sn_state: base
            2, 0, 0, 0, // reader_sn_state: base
            2, 0, 0, 0, // reader_sn_state: num bits
            0b_00000000, 0b_00000000, 0b_00000000, 0b_11000000, // reader_sn_state: bitmap
            2, 0, 0, 0, // Count
        ];
        
        let expected = AckNack {
            endianness_flag: EndianessFlag::LittleEndian.into(),
            final_flag: SubmessageFlag(true),
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            reader_sn_state: SequenceNumberSet::new([SequenceNumber(2), SequenceNumber(3)].iter().cloned().collect()),
            count: Count(2),
        };
        let result = AckNack::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn compose_gap_submessage() {
        let expected = vec![
            0x0f, 0b00000011, 28, 0, 
            0x00, 0x00, 0x00, 0x00, // readerId 
            0x00, 0x01, 0x00, 0xc2, // writerId
            0, 0, 0, 0, // reader_sn_state: base
            2, 0, 0, 0, // reader_sn_state: base
            2, 0, 0, 0, // reader_sn_state: num bits
            0b_00000000, 0b_00000000, 0b_00000000, 0b_11000000, // reader_sn_state: bitmap
            2, 0, 0, 0, // Count
        ];

        let message = AckNack {
            endianness_flag: EndianessFlag::LittleEndian.into(),
            final_flag: SubmessageFlag(true),
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            reader_sn_state: SequenceNumberSet::new([SequenceNumber(2), SequenceNumber(3)].iter().cloned().collect()),
            count: Count(2),
        };

        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);        
    }
}
