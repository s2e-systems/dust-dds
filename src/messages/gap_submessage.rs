use crate::types_primitives::Ushort;
use crate::types::{EntityId, SequenceNumber, };
use crate::messages::types::{SubmessageKind, SubmessageFlag, };
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsParse, RtpsCompose, EndianessFlag, RtpsSerdesResult, };
use super::{SubmessageHeader, Submessage, };
use super::submessage_elements::SequenceNumberSet;

#[derive(PartialEq, Debug)]
pub struct Gap {
    endianness_flag: SubmessageFlag,
    // group_info_flag: SubmessageFlag,
    reader_id: EntityId,
    writer_id: EntityId,
    gap_start: SequenceNumber,
    gap_list: SequenceNumberSet,    
    // gap_start_gsn: SequenceNumber,
    // gap_end_gsn: SequenceNumber,
}

impl Submessage for Gap {
    fn submessage_header(&self) -> SubmessageHeader {
        let x = SubmessageFlag(false);
        let e = self.endianness_flag; // Indicates endianness.
        // X|X|X|X|X|X|X|E
        let flags = [e, x, x, x, x, x, x, x];

        let octets_to_next_header = 
            self.reader_id.octets() + 
            self.writer_id.octets() +
            self.gap_start.octets() +
            self.gap_list.octets();

        SubmessageHeader { 
            submessage_id: SubmessageKind::Gap,
            flags,
            submessage_length: Ushort(octets_to_next_header as u16), // This cast could fail in weird ways by truncation
        }
    }
}


impl RtpsCompose for Gap {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = self.endianness_flag.into();
        self.submessage_header().compose(writer)?;
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.gap_start.serialize(writer, endianness)?;
        self.gap_list.serialize(writer, endianness)?;
        Ok(())
    }
}

impl RtpsParse for Gap {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> { 
        let header = SubmessageHeader::parse(bytes)?;
        let flags = header.flags();
        // X|X|X|X|X|X|X|E
        /*E*/ let endianness_flag = flags[0];
        let endianness = EndianessFlag::from(endianness_flag);

        let reader_id = EntityId::deserialize(&bytes[4..8], endianness)?;
        let writer_id = EntityId::deserialize(&bytes[8..12], endianness)?;
        let gap_start = SequenceNumber::deserialize(&bytes[12..20], endianness)?;
        let gap_list = SequenceNumberSet::deserialize(&bytes[20..], endianness)?;
         

        Ok(Gap {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types_other::{EntityKey, EntityKind, };
    
    #[test]
    fn serialize_gap_submessage_big_endian() {
        let expected = vec![
            0x08, 0b00000000, 0, 32, // Header 
            0x10, 0x12, 0x14, 0x04, // readerId
            0x26, 0x24, 0x22, 0x02, // writerId
            0x00, 0x00, 0x00, 0x00, // gapStart
            0x00, 0x00, 0x04, 0xB0, // gapStart
            0x00, 0x00, 0x00, 0x00, // gapList base
            0x00, 0x00, 0x04, 0xD2, // gapList base
            0x00, 0x00, 0x00,    2, // gapList numBits
            0b11000000, 0x00, 0x00, 0x00, // gapList bitmap
        ];
        let endianness_flag = EndianessFlag::BigEndian.into();
        let reader_id = EntityId::new(EntityKey([0x10, 0x12, 0x14]), EntityKind::UserDefinedReaderWithKey);
        let writer_id = EntityId::new(EntityKey([0x26, 0x24, 0x22]), EntityKind::UserDefinedWriterWithKey);
        let gap_start = SequenceNumber(1200);
        let gap_list = SequenceNumberSet::new([
            SequenceNumber(1234),
            SequenceNumber(1235),
        ].iter().cloned().collect());

        let message = Gap {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        };
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);        
    }

    #[test]
    fn serialize_gap_submessage_little_endian() {
        let expected = vec![
            0x08, 0b00000001, 32, 0, // Header 
            0x10, 0x12, 0x14, 0x04, // readerId
            0x26, 0x24, 0x22, 0x02, // writerId
            0x00, 0x00, 0x00, 0x00, // gapStart
            0xB0, 0x04, 0x00, 0x00, // gapStart
            0x00, 0x00, 0x00, 0x00, // gapList base
            0xD2, 0x04, 0x00, 0x00, // gapList base
               2, 0x00, 0x00, 0x00, // gapList numBits
            0x00, 0x00, 0x00, 0b11000000, // gapList bitmap
        ];
        let endianness_flag = EndianessFlag::LittleEndian.into();
        let reader_id = EntityId::new(EntityKey([0x10, 0x12, 0x14]), EntityKind::UserDefinedReaderWithKey);
        let writer_id = EntityId::new(EntityKey([0x26, 0x24, 0x22]), EntityKind::UserDefinedWriterWithKey);
        let gap_start = SequenceNumber(1200);
        let gap_list = SequenceNumberSet::new([
            SequenceNumber(1234),
            SequenceNumber(1235),
        ].iter().cloned().collect());

        let message = Gap {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        };
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);        
    }

    #[test]
    fn deserialize_gap_submessage_big_endian() {
        let bytes = vec![
            0x08, 0b00000000, 0, 32, // Header 
            0x10, 0x12, 0x14, 0x04, // readerId
            0x26, 0x24, 0x22, 0x02, // writerId
            0x00, 0x00, 0x00, 0x00, // gapStart
            0x00, 0x00, 0x04, 0xB0, // gapStart
            0x00, 0x00, 0x00, 0x00, // gapList base
            0x00, 0x00, 0x04, 0xD2, // gapList base
            0x00, 0x00, 0x00,    2, // gapList numBits
            0b11000000, 0x00, 0x00, 0x00, // gapList bitmap
        ];

        let endianness_flag = EndianessFlag::BigEndian.into();
        let reader_id = EntityId::new(EntityKey([0x10, 0x12, 0x14]), EntityKind::UserDefinedReaderWithKey);
        let writer_id = EntityId::new(EntityKey([0x26, 0x24, 0x22]), EntityKind::UserDefinedWriterWithKey);
        let gap_start = SequenceNumber(1200);
        let gap_list = SequenceNumberSet::new([
            SequenceNumber(1234),
            SequenceNumber(1235),
        ].iter().cloned().collect());

        let expected = Gap {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        };

        let result = Gap::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_gap_submessage_little_endian() {
        let bytes = vec![
            0x08, 0b00000001, 32, 0, // Header 
            0x10, 0x12, 0x14, 0x04, // readerId
            0x26, 0x24, 0x22, 0x02, // writerId
            0x00, 0x00, 0x00, 0x00, // gapStart
            0xB0, 0x04, 0x00, 0x00, // gapStart
            0x00, 0x00, 0x00, 0x00, // gapList base
            0xD2, 0x04, 0x00, 0x00, // gapList base
               2, 0x00, 0x00, 0x00, // gapList numBits
            0x00, 0x00, 0x00, 0b11000000, // gapList bitmap
        ];

        let endianness_flag = EndianessFlag::LittleEndian.into();
        let reader_id = EntityId::new(EntityKey([0x10, 0x12, 0x14]), EntityKind::UserDefinedReaderWithKey);
        let writer_id = EntityId::new(EntityKey([0x26, 0x24, 0x22]), EntityKind::UserDefinedWriterWithKey);
        let gap_start = SequenceNumber(1200);
        let gap_list = SequenceNumberSet::new([
            SequenceNumber(1234),
            SequenceNumber(1235),
        ].iter().cloned().collect());

        let expected = Gap {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        };

        let result = Gap::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }
}