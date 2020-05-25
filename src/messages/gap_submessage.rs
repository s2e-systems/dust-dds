use crate::types::{EntityId, SequenceNumber, SequenceNumberSet, Ushort};
use crate::serdes::{RtpsSerialize, RtpsCompose, RtpsParse, RtpsDeserialize, EndianessFlag, RtpsSerdesResult};
use super::{Submessage, SubmessageFlag, SubmessageHeader, SubmessageKind};

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
    use crate::types::{EntityKey, EntityKind};
    
    #[test]
    fn test_serialize_gap_submessage_big_endian() {
        let expected = vec![
            0x08, 0b00000000, 0, 32, // Header 
            0x10, 0x12, 0x14, 0x04, // readerId
            0x26, 0x24, 0x22, 0x02, // writerId
            0x00, 0x00, 0x00, 0x00, // gapStart
            0x00, 0x00, 0x04, 0xD1, // gapStart
            0x00, 0x00, 0x00, 0x00, // gapList 1
            0x00, 0x00, 0x04, 0xD2, // gapList 1
            0x00, 0x00, 0x00, 0x08, // gapList 2
            0x00, 0x00, 0x00, 0x0C, // gapList 2
        ];
        let endianness_flag = EndianessFlag::BigEndian.into();
        let reader_id = EntityId::new(EntityKey([0x10, 0x12, 0x14]), EntityKind::UserDefinedReaderWithKey);
        let writer_id = EntityId::new(EntityKey([0x26, 0x24, 0x22]), EntityKind::UserDefinedWriterWithKey);
        let gap_start = SequenceNumber(1233);
        let gap_list = SequenceNumberSet([
            (SequenceNumber(1234), false),
            (SequenceNumber(1235), false),
            (SequenceNumber(1236), true),
            (SequenceNumber(1237), true),
            (SequenceNumber(1238), false),
            (SequenceNumber(1239), false),
            (SequenceNumber(1240), false),
            (SequenceNumber(1241), false)
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

//     #[test]
//     fn test_parse_gap_submessage_little_endian() {
//         let submessage_little_endian = [
//             0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0xD1, 0x04,
//             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD2, 0x04, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
//             0x0C, 0x00, 0x00, 0x00,
//         ];

//         let gap_little_endian = parse_gap_submessage(&submessage_little_endian, &1).unwrap();

//         assert_eq!(
//             gap_little_endian.reader_id,
//             EntityId::new([0x10, 0x12, 0x14], 0x16)
//         );
//         assert_eq!(
//             gap_little_endian.writer_id,
//             EntityId::new([0x26, 0x24, 0x22], 0x20)
//         );
//         assert_eq!(gap_little_endian.gap_start, 1233);
//         assert_eq!(gap_little_endian.gap_list.len(), 8);
//         assert_eq!(
//             gap_little_endian.gap_list,
//             [
//                 (1234, false),
//                 (1235, false),
//                 (1236, true),
//                 (1237, true),
//                 (1238, false),
//                 (1239, false),
//                 (1240, false),
//                 (1241, false)
//             ].iter().cloned().collect()
//         );
//     }

//     #[test]
//     fn test_parse_gap_submessage_invalid() {
//         let submessage_big_endian = [
//             0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
//             0x04, 0xD1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD2, 0x00, 0x00, 0x00, 0x08,
//             0x00, 0x00, 0x00, 0x0C,
//         ];

//         let gap_big_endian = parse_gap_submessage(&submessage_big_endian, &0);

//         if let Err(RtpsMessageError::InvalidSubmessage) = gap_big_endian {
//             assert!(true);
//         } else {
//             assert!(false);
//         }
//     }
}
