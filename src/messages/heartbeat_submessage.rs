use crate::types::{Count, EntityId, SequenceNumber};
use crate::serdes::{RtpsSerialize, RtpsCompose, RtpsParse, RtpsDeserialize, EndianessFlag, RtpsSerdesResult, SizeSerializer, PrimitiveSerdes, SizeCheckers, RtpsSerdesError};
use super::{SubmessageKind, SubmessageFlag};

#[derive(PartialEq, Debug)]
pub struct Heartbeat {
    endianess_flag: SubmessageFlag,
    final_flag: SubmessageFlag,
    liveliness_flag: SubmessageFlag,
    // group_info_flag: SubmessageFlag,
    reader_id: EntityId,
    writer_id: EntityId,
    first_sn: SequenceNumber,
    last_sn: SequenceNumber,
    count: Count,
    // current_gsn: SequenceNumber,
    // first_gsn: SequenceNumber,
    // last_gsn: SequenceNumber,
    // writer_set: GroupDigest,
    // secure_writer_set: GroupDigest,
}

impl Heartbeat {
    const FINAL_FLAG_MASK: u8 = 0x02;
    const LIVELINESS_FLAG_MASK: u8 = 0x04;

    pub fn new(reader_id: EntityId,
        writer_id: EntityId,
        first_sn: SequenceNumber,
        last_sn: SequenceNumber,
        count: Count,
        final_flag: SubmessageFlag,
        liveliness_flag: SubmessageFlag,
        endianess_flag: SubmessageFlag) -> Self {
            Heartbeat {
                reader_id,
                writer_id,
                first_sn,
                last_sn,
                count,
                final_flag,
                liveliness_flag,
                endianess_flag,
            }
        }

    pub fn serialize_message_body(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.first_sn.serialize(writer, endianness)?;
        self.last_sn.serialize(writer, endianness)?;
        self.count.serialize(writer, endianness)
    }

    pub fn is_valid(&self) -> bool{
        if self.first_sn < SequenceNumber::new(1) {
            return false;
        };

        if self.last_sn < SequenceNumber::new(0) {
            return false;
        }

        if self.last_sn < self.first_sn - 1 {
            return false;
        }

        true
    }
}

impl RtpsCompose for Heartbeat {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        todo!()
    }
}

// impl RtpsSerialize for Heartbeat {
//     fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
//         SubmessageKind::Heartbeat.serialize(writer, endianness)?;

//         let mut flags = endianness as u8;
//         if self.final_flag {
//             flags |= Heartbeat::FINAL_FLAG_MASK;
//         }

//         if self.liveliness_flag {
//             flags |= Heartbeat::LIVELINESS_FLAG_MASK;
//         }

//         writer.write(&[flags])?;
        
//         let mut size_serializer = SizeSerializer::new();
//         self.serialize_message_body(&mut size_serializer, endianness)?;

//         writer.write(&PrimitiveSerdes::serialize_u16(size_serializer.get_size() as u16, endianness))?;

//         self.serialize_message_body(writer, endianness)
//     }
// }

// impl RtpsParse for Heartbeat {
//     fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
//         const SERIALIZED_HEARTBEAT_SIZE: usize = 32;
//         const SUBMESSAGE_ID_INDEX: usize = 0;
//         const FLAGS_INDEX: usize = 1;
//         const READER_ID_FIRST_INDEX: usize = 4;
//         const READER_ID_LAST_INDEX: usize = 7;
//         const WRITER_ID_FIRST_INDEX: usize = 8;
//         const WRITER_ID_LAST_INDEX: usize = 11;
//         const FIRST_SN_FIRST_INDEX: usize = 12;
//         const FIRST_SN_LAST_INDEX: usize = 19;
//         const LAST_SN_FIRST_INDEX: usize = 20;
//         const LAST_SN_LAST_INDEX: usize = 27;
//         const COUNT_FIRST_INDEX: usize = 28;
//         const COUNT_LAST_INDEX: usize = 31;

//         SizeCheckers::check_size_equal(bytes, SERIALIZED_HEARTBEAT_SIZE)?;

//         let submessage_kind = SubmessageKind::parse(&[bytes[SUBMESSAGE_ID_INDEX]])?;
//         if submessage_kind != SubmessageKind::Heartbeat {
//             return Err(RtpsSerdesError::InvalidSubmessageHeader);
//         }

//         let flags = bytes[FLAGS_INDEX];
//         let endianness : EndianessFlag = flags.into();
//         let final_flag = flags & Heartbeat::FINAL_FLAG_MASK == Heartbeat::FINAL_FLAG_MASK;
//         let liveliness_flag = flags & Heartbeat::LIVELINESS_FLAG_MASK == Heartbeat::LIVELINESS_FLAG_MASK;

//         let reader_id = EntityId::parse(&bytes[READER_ID_FIRST_INDEX..=READER_ID_LAST_INDEX])?;
//         let writer_id = EntityId::parse(&bytes[WRITER_ID_FIRST_INDEX..=WRITER_ID_LAST_INDEX])?;
//         let first_sn = SequenceNumber::deserialize(&bytes[FIRST_SN_FIRST_INDEX..=FIRST_SN_LAST_INDEX], endianness)?;
//         let last_sn = SequenceNumber::deserialize(&bytes[LAST_SN_FIRST_INDEX..=LAST_SN_LAST_INDEX], endianness)?;
//         let count = Count::deserialize(&bytes[COUNT_FIRST_INDEX..=COUNT_LAST_INDEX], endianness)?;

//         Ok(Heartbeat::new(reader_id, writer_id, first_sn, last_sn, count, final_flag, liveliness_flag))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EntityKind, EntityKey};
    use crate::types::constants::ENTITYID_UNKNOWN;

    #[test]
    fn test_heartbeat_validity_function() {
        let valid_heartbeat = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: SequenceNumber::new(2),
            last_sn: SequenceNumber::new(5), 
            count: Count::new(0),
            final_flag: SubmessageFlag(true),
            liveliness_flag: SubmessageFlag(true),
            endianess_flag: EndianessFlag::LittleEndian.into(),
        };

        assert_eq!(valid_heartbeat.is_valid(), true);

        let valid_heartbeat_first_message = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: SequenceNumber::new(1),
            last_sn: SequenceNumber::new(0), 
            count: Count::new(2),
            final_flag: SubmessageFlag(true),
            liveliness_flag: SubmessageFlag(true),
            endianess_flag: EndianessFlag::LittleEndian.into(),
        };

        assert_eq!(valid_heartbeat_first_message.is_valid(), true);

        let invalid_heartbeat_zero_first_value = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: SequenceNumber::new(0),
            last_sn: SequenceNumber::new(1), 
            count: Count::new(2),
            final_flag: SubmessageFlag(true),
            liveliness_flag: SubmessageFlag(true),
            endianess_flag: EndianessFlag::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_zero_first_value.is_valid(), false);

        let invalid_heartbeat_negative_last_value = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: SequenceNumber::new(5),
            last_sn: SequenceNumber::new(-6), 
            count: Count::new(2),
            final_flag: SubmessageFlag(true),
            liveliness_flag: SubmessageFlag(true),
            endianess_flag: EndianessFlag::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_negative_last_value.is_valid(), false);

        let invalid_heartbeat_wrong_first_last_value = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: SequenceNumber::new(6),
            last_sn: SequenceNumber::new(4), 
            count: Count::new(2),
            final_flag: SubmessageFlag(true),
            liveliness_flag: SubmessageFlag(true),
            endianess_flag: EndianessFlag::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_wrong_first_last_value.is_valid(), false);
    }
    // #[test]
    // fn test_serialize_deserialize_heartbeat() {
    //     let mut writer = Vec::new();

    //     let reader_id = EntityId::new(EntityKey::new([0x10, 0x12, 0x14]), EntityKind::UserDefinedReaderWithKey);
    //     let writer_id = EntityId::new(EntityKey::new([0x26, 0x24, 0x22]), EntityKind::UserDefinedWriterWithKey);
    //     let first_sn = SequenceNumber::new(1233);
    //     let last_sn = SequenceNumber::new(1237);
    //     let count = Count::new(8);
    //     let final_flag = true;
    //     let liveliness_flag = false;

    //     let heartbeat = Heartbeat::new(reader_id, writer_id, first_sn, last_sn, count, final_flag, liveliness_flag);
    //     heartbeat.serialize(&mut writer, EndianessFlag::BigEndian).unwrap();
    //     let submessage_big_endian = [
    //         0x07, 0x02, 0x00, 0x1C, // Submessage Header
    //         0x10, 0x12, 0x14, 0x04, // Reader ID
    //         0x26, 0x24, 0x22, 0x02, // Writer ID
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD1, // First Sequence Number
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD5, // Last Sequence Number
    //         0x00, 0x00, 0x00, 0x08, // Count
    //     ];

    //     assert_eq!(writer, submessage_big_endian);
    //     assert_eq!(Heartbeat::parse(&writer).unwrap(), heartbeat);

    //     writer.clear();

    //     heartbeat.serialize(&mut writer, EndianessFlag::LittleEndian).unwrap();
    //     let submessage_little_endian = [
    //         0x07, 0x03, 0x1C, 0x00, // Submessage Header
    //         0x10, 0x12, 0x14, 0x04, // Reader ID
    //         0x26, 0x24, 0x22, 0x02, // Writer ID
    //         0x00, 0x00, 0x00, 0x00, 0xD1, 0x04, 0x00, 0x00, // First Sequence Number
    //         0x00, 0x00, 0x00, 0x00, 0xD5, 0x04, 0x00, 0x00, // Last Sequence Number
    //         0x08, 0x00, 0x00, 0x00, // Count
    //     ];
    //     assert_eq!(writer, submessage_little_endian);
    //     assert_eq!(Heartbeat::parse(&writer).unwrap(), heartbeat);

    // }
//     fn test_parse_heartbeat_submessage_big_endian() {
//         let submessage_big_endian = [
//             0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//             0x04, 0xD1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD5, 0x00, 0x00, 0x00, 0x08,
//         ];

//         let heartbeat_big_endian = parse_heartbeat_submessage(&submessage_big_endian, &0).unwrap();
//         assert_eq!(
//             heartbeat_big_endian.reader_id,
//             EntityId::new([0x10, 0x12, 0x14], 0x16)
//         );
//         assert_eq!(
//             heartbeat_big_endian.writer_id,
//             EntityId::new([0x26, 0x24, 0x22], 0x20)
//         );
//         assert_eq!(heartbeat_big_endian.first_sn, 1233);
//         assert_eq!(heartbeat_big_endian.last_sn, 1237);
//         assert_eq!(heartbeat_big_endian.count, 8);
//         assert_eq!(heartbeat_big_endian.final_flag, false);
//         assert_eq!(heartbeat_big_endian.liveliness_flag, false);
//     }

//     #[test]
//     fn test_parse_heartbeat_submessage_little_endian() {
//         let submessage_little_endian = [
//             0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0xD1, 0x04,
//             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD5, 0x04, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
//         ];

//         let heartbeat_little_endian =
//             parse_heartbeat_submessage(&submessage_little_endian, &7).unwrap();
//         assert_eq!(
//             heartbeat_little_endian.reader_id,
//             EntityId::new([0x10, 0x12, 0x14], 0x16)
//         );
//         assert_eq!(
//             heartbeat_little_endian.writer_id,
//             EntityId::new([0x26, 0x24, 0x22], 0x20)
//         );
//         assert_eq!(heartbeat_little_endian.first_sn, 1233);
//         assert_eq!(heartbeat_little_endian.last_sn, 1237);
//         assert_eq!(heartbeat_little_endian.count, 8);
//         assert_eq!(heartbeat_little_endian.final_flag, true);
//         assert_eq!(heartbeat_little_endian.liveliness_flag, true);
//     }

//     #[test]
//     fn test_parse_heartbeat_submessage_first_sn_equal_one() {
//         // Test first
//         let submessage_big_endian = [
//             0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//             0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08,
//         ];

//         let heartbeat_big_endian = parse_heartbeat_submessage(&submessage_big_endian, &2).unwrap();
//         assert_eq!(
//             heartbeat_big_endian.reader_id,
//             EntityId::new([0x10, 0x12, 0x14], 0x16)
//         );
//         assert_eq!(
//             heartbeat_big_endian.writer_id,
//             EntityId::new([0x26, 0x24, 0x22], 0x20)
//         );
//         assert_eq!(heartbeat_big_endian.first_sn, 1);
//         assert_eq!(heartbeat_big_endian.last_sn, 0);
//         assert_eq!(heartbeat_big_endian.count, 8);
//         assert_eq!(heartbeat_big_endian.final_flag, true);
//         assert_eq!(heartbeat_big_endian.liveliness_flag, false);
//     }

//     #[test]
//     fn test_parse_heartbeat_submessage_invalid_submessage_last_sn_below_first_sn() {
//         // Test last_sn < first_sn - 1
//         let submessage = [
//             0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//             0x04, 0xD1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xCF, 0x00, 0x00, 0x00, 0x08,
//         ];

//         let heartbeat = parse_heartbeat_submessage(&submessage, &0);
//         if let Err(RtpsMessageError::InvalidSubmessage) = heartbeat {
//             assert!(true);
//         } else {
//             assert!(false);
//         }
//     }

//     #[test]
//     fn test_parse_heartbeat_submessage_invalid_submessage_first_sn_zero() {
//         // Test first_sn = 0
//         let submessage = [
//             0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD5, 0x00, 0x00, 0x00, 0x08,
//         ];

//         let heartbeat = parse_heartbeat_submessage(&submessage, &0);
//         if let Err(RtpsMessageError::InvalidSubmessage) = heartbeat {
//             assert!(true);
//         } else {
//             assert!(false);
//         }
//     }

//     #[test]
//     fn test_parse_heartbeat_submessage_invalid_submessage_last_sn_below_zero() {
//         // Test last_sn < 0
//         let submessage = [
//             0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//             0x04, 0xD1, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD5, 0x00, 0x00, 0x00, 0x08,
//         ];

//         let heartbeat = parse_heartbeat_submessage(&submessage, &0);
//         if let Err(RtpsMessageError::InvalidSubmessage) = heartbeat {
//             assert!(true);
//         } else {
//             assert!(false);
//         }
//     }
}
