use crate::primitive_types::UShort;
use crate::types::{SequenceNumber, EntityId, };
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsParse, RtpsCompose, Endianness, RtpsSerdesResult, };

use super::types::{SubmessageKind, SubmessageFlag, Count, };
use super::{SubmessageHeader, Submessage, };
use super::submessage_elements::FragmentNumber;

#[derive(PartialEq, Debug)]
pub struct HeartbeatFrag {
    endianness_flag: SubmessageFlag,
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    last_fragment_num: FragmentNumber,
    count: Count,
}

impl Submessage for HeartbeatFrag {
    fn submessage_header(&self) -> SubmessageHeader {
        const X: SubmessageFlag = false;
        let e = self.endianness_flag;
        let flags = [e, X, X, X, X, X, X, X];

        let octets_to_next_header = 
            self.reader_id.octets() + 
            self.writer_id.octets() +
            self.writer_sn.octets() +
            self.last_fragment_num.octets() +
            self.count.octets();

        SubmessageHeader { 
            submessage_id: SubmessageKind::HeartbeatFrag,
            flags,
            submessage_length: octets_to_next_header as UShort,
        }
    }

    fn is_valid(&self) -> bool {
        if self.writer_sn <= SequenceNumber(0) ||
        self.last_fragment_num <= FragmentNumber(0) {
            false
        } else {
            true
        }
    }
}

impl RtpsCompose for HeartbeatFrag {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = self.endianness_flag.into();
        self.submessage_header().compose(writer)?;
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.writer_sn.serialize(writer, endianness)?;
        self.last_fragment_num.serialize(writer, endianness)?;
        self.count.serialize(writer, endianness)?;
        Ok(())
    }    
}

impl RtpsParse for HeartbeatFrag {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let header = SubmessageHeader::parse(bytes)?;
        let endianness_flag = header.flags()[0];
        let endianness = Endianness::from(endianness_flag);

        let reader_id = EntityId::deserialize(&bytes[4..8], endianness)?;
        let writer_id = EntityId::deserialize(&bytes[8..12], endianness)?;
        let writer_sn = SequenceNumber::deserialize(&bytes[12..20], endianness)?;
        let last_fragment_num = FragmentNumber::deserialize(&bytes[20..24], endianness)?;
        let count = Count::deserialize(&bytes[24..28], endianness)?;        

        Ok(HeartbeatFrag {
            endianness_flag,
            reader_id,
            writer_id,
            writer_sn,
            last_fragment_num,
            count,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, };

    #[test]
    fn parse_heartbeat_frag_submessage() {
        let expected = HeartbeatFrag {
            endianness_flag: true,    
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber(1),
            last_fragment_num: FragmentNumber(2),
            count: Count(3),
        };
        let bytes = vec![
            0x13, 0b00000001, 24, 0x0, // Submessgae Header
            0x00, 0x00, 0x00, 0x00, // readerId 
            0x00, 0x01, 0x00, 0xc2, // writerId
            0x00, 0x00, 0x00, 0x00, // writerSN
            0x01, 0x00, 0x00, 0x00, // writerSN 
            0x02, 0x00, 0x00, 0x00, // lastFragmentNum
            0x03, 0x00, 0x00, 0x00, // count
        ];
        let result = HeartbeatFrag::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    
    #[test]
    fn compose_heartbeat_frag_submessage() {
        let message = HeartbeatFrag {
            endianness_flag: true,    
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            writer_sn: SequenceNumber(1),
            last_fragment_num: FragmentNumber(2),
            count: Count(3),
        };
        let expected = vec![
            0x13, 0b00000001, 24, 0x0, // Submessgae Header
            0x00, 0x00, 0x00, 0x00, // readerId 
            0x00, 0x01, 0x00, 0xc2, // writerId
            0x00, 0x00, 0x00, 0x00, // writerSN
            0x01, 0x00, 0x00, 0x00, // writerSN 
            0x02, 0x00, 0x00, 0x00, // lastFragmentNum
            0x03, 0x00, 0x00, 0x00, // count
        ];
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);
    }

}
