use std::collections::BTreeSet;

use crate::primitive_types::UShort;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsParse, RtpsCompose, Endianness, RtpsSerdesResult, };

use super::types::{SubmessageKind, SubmessageFlag, };
use super::{SubmessageHeader, Submessage, };
use super::submessage_elements;
// use super::submessage_elements::SequenceNumberSet;

#[derive(PartialEq, Debug)]
pub struct Gap {
    endianness_flag: SubmessageFlag,
    // group_info_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId,
    writer_id: submessage_elements::EntityId,
    gap_start: submessage_elements::SequenceNumber,
    gap_list: submessage_elements::SequenceNumberSet,    
    // gap_start_gsn: submessage_elements::SequenceNumber,
    // gap_end_gsn: submessage_elements::SequenceNumber,
}

impl Gap {
    pub fn new(
        reader_id: submessage_elements::EntityId,
        writer_id: submessage_elements::EntityId,
        gap_start: submessage_elements::SequenceNumber,
        endianness: Endianness,) -> Self {

            let mut gap_list_set = BTreeSet::new();
            gap_list_set.insert(gap_start.0);

            Gap {
                reader_id,
                writer_id,
                gap_start,
                gap_list: submessage_elements::SequenceNumberSet::from_set(gap_list_set),
                endianness_flag: endianness.into(),
            }
    }

    pub fn reader_id(&self) -> &submessage_elements::EntityId {
        &self.reader_id
    }

    pub fn writer_id(&self) -> &submessage_elements::EntityId {
        &self.writer_id
    }

    pub fn gap_start(&self) -> &submessage_elements::SequenceNumber {
        &self.gap_start
    }

    pub fn gap_list(&self) -> &submessage_elements::SequenceNumberSet {
        &self.gap_list
    }
}

impl Submessage for Gap {
    fn submessage_header(&self) -> SubmessageHeader {
        let x = false;
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
            submessage_length: octets_to_next_header as UShort, 
        }
    }

    fn is_valid(&self) -> bool {
        if self.gap_start.0 <= 0 ||
           !self.gap_list.is_valid()
        {
            false
        } else {
            true
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
        let endianness = Endianness::from(endianness_flag);

        let reader_id = submessage_elements::EntityId::deserialize(&bytes[4..8], endianness)?;
        let writer_id = submessage_elements::EntityId::deserialize(&bytes[8..12], endianness)?;
        let gap_start = submessage_elements::SequenceNumber::deserialize(&bytes[12..20], endianness)?;
        let gap_list = submessage_elements::SequenceNumberSet::deserialize(&bytes[20..], endianness)?;
         

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
    use crate::types::{EntityKey, EntityKind, };
    
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
        let endianness_flag = Endianness::BigEndian.into();
        let reader_id = EntityId::new(EntityKey([0x10, 0x12, 0x14]), EntityKind::UserDefinedReaderWithKey);
        let writer_id = EntityId::new(EntityKey([0x26, 0x24, 0x22]), EntityKind::UserDefinedWriterWithKey);
        let gap_start = SequenceNumber(1200);
        let gap_list = SequenceNumberSet::from_set([
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
        let endianness_flag = Endianness::LittleEndian.into();
        let reader_id = EntityId::new(EntityKey([0x10, 0x12, 0x14]), EntityKind::UserDefinedReaderWithKey);
        let writer_id = EntityId::new(EntityKey([0x26, 0x24, 0x22]), EntityKind::UserDefinedWriterWithKey);
        let gap_start = SequenceNumber(1200);
        let gap_list = SequenceNumberSet::from_set([
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

        let endianness_flag = Endianness::BigEndian.into();
        let reader_id = EntityId::new(EntityKey([0x10, 0x12, 0x14]), EntityKind::UserDefinedReaderWithKey);
        let writer_id = EntityId::new(EntityKey([0x26, 0x24, 0x22]), EntityKind::UserDefinedWriterWithKey);
        let gap_start = SequenceNumber(1200);
        let gap_list = SequenceNumberSet::from_set([
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

        let endianness_flag = Endianness::LittleEndian.into();
        let reader_id = EntityId::new(EntityKey([0x10, 0x12, 0x14]), EntityKind::UserDefinedReaderWithKey);
        let writer_id = EntityId::new(EntityKey([0x26, 0x24, 0x22]), EntityKind::UserDefinedWriterWithKey);
        let gap_start = SequenceNumber(1200);
        let gap_list = SequenceNumberSet::from_set([
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