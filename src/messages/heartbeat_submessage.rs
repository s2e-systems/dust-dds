use crate::types::{Count, EntityId, SequenceNumber, Ushort};
use crate::serdes::{RtpsSerialize, RtpsCompose, RtpsParse, RtpsDeserialize, EndianessFlag, RtpsSerdesResult};
use super::{SubmessageKind, SubmessageFlag, Submessage, SubmessageHeader};

#[derive(PartialEq, Debug)]
pub struct Heartbeat {
    endianness_flag: SubmessageFlag,
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
        final_flag: bool,
        manual_liveliness: bool,
        endianness_flag: EndianessFlag) -> Self {
            Heartbeat {
                reader_id,
                writer_id,
                first_sn,
                last_sn,
                count,
                final_flag: SubmessageFlag(final_flag),
                liveliness_flag: SubmessageFlag(manual_liveliness),
                endianness_flag: endianness_flag.into(),
            }
        }

    pub fn is_valid(&self) -> bool{
        if self.first_sn < SequenceNumber(1) {
            return false;
        };

        if self.last_sn < SequenceNumber(0) {
            return false;
        }

        if self.last_sn < self.first_sn - 1 {
            return false;
        }

        true
    }

    pub fn reader_id(&self) -> &EntityId {
        &self.reader_id
    }

    pub fn writer_id(&self) -> &EntityId {
        &self.writer_id
    }

    pub fn first_sn(&self) -> &SequenceNumber {
        &self.first_sn
    }

    pub fn last_sn(&self) -> &SequenceNumber {
        &self.last_sn
    }

    pub fn count(&self) -> &Count {
        &self.count
    }

    pub fn is_final(&self) -> bool {
        self.final_flag.is_set()
    }
}

impl Submessage for Heartbeat {
    fn submessage_header(&self) -> SubmessageHeader {
        let x = SubmessageFlag(false);
        let e = self.endianness_flag; // Indicates endianness.
        let f = self.final_flag; //Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
        let l = self.liveliness_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
        // X|X|X|X|X|L|F|E
        let flags = [e, f, l, x, x, x, x, x];

        let octets_to_next_header = 
            self.reader_id.octets() + 
            self.writer_id.octets() +
            self.first_sn.octets() +
            self.last_sn.octets() +
            self.count.octets();

        SubmessageHeader { 
            submessage_id: SubmessageKind::Heartbeat,
            flags,
            submessage_length: Ushort(octets_to_next_header as u16), // This cast could fail in weird ways by truncation
        }
    }
}

impl RtpsCompose for Heartbeat {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = self.endianness_flag.into();
        self.submessage_header().compose(writer)?;
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.first_sn.serialize(writer, endianness)?;
        self.last_sn.serialize(writer, endianness)?;
        self.count.serialize(writer, endianness)?;
        Ok(())
    }
}

impl RtpsParse for Heartbeat {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> { 
        let header = SubmessageHeader::parse(bytes)?;
        let flags = header.flags();
        // X|X|X|X|X|L|F|E
        /*E*/ let endianness_flag = flags[0];
        /*F*/ let final_flag = flags[1];
        /*L*/ let liveliness_flag = flags[2];

        let endianness = EndianessFlag::from(endianness_flag);

        const HEADER_SIZE : usize = 8;
        let reader_id = EntityId::deserialize(&bytes[4..8], endianness)?;
        let writer_id = EntityId::deserialize(&bytes[8..12], endianness)?;
        let first_sn = SequenceNumber::deserialize(&bytes[12..20], endianness)?;
        let last_sn = SequenceNumber::deserialize(&bytes[20..28], endianness)?;
        let count = Count::deserialize(&bytes[28..32], endianness)?;
        

        Ok(Heartbeat {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        })
    }
}

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
            first_sn: SequenceNumber(2),
            last_sn: SequenceNumber(5), 
            count: Count(0),
            final_flag: SubmessageFlag(true),
            liveliness_flag: SubmessageFlag(true),
            endianness_flag: EndianessFlag::LittleEndian.into(),
        };

        assert_eq!(valid_heartbeat.is_valid(), true);

        let valid_heartbeat_first_message = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: SequenceNumber(1),
            last_sn: SequenceNumber(0), 
            count: Count(2),
            final_flag: SubmessageFlag(true),
            liveliness_flag: SubmessageFlag(true),
            endianness_flag: EndianessFlag::LittleEndian.into(),
        };

        assert_eq!(valid_heartbeat_first_message.is_valid(), true);

        let invalid_heartbeat_zero_first_value = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: SequenceNumber(0),
            last_sn: SequenceNumber(1), 
            count: Count(2),
            final_flag: SubmessageFlag(true),
            liveliness_flag: SubmessageFlag(true),
            endianness_flag: EndianessFlag::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_zero_first_value.is_valid(), false);

        let invalid_heartbeat_negative_last_value = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: SequenceNumber(5),
            last_sn: SequenceNumber(-6), 
            count: Count(2),
            final_flag: SubmessageFlag(true),
            liveliness_flag: SubmessageFlag(true),
            endianness_flag: EndianessFlag::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_negative_last_value.is_valid(), false);

        let invalid_heartbeat_wrong_first_last_value = Heartbeat {
            reader_id: ENTITYID_UNKNOWN,
            writer_id: ENTITYID_UNKNOWN,
            first_sn: SequenceNumber(6),
            last_sn: SequenceNumber(4), 
            count: Count(2),
            final_flag: SubmessageFlag(true),
            liveliness_flag: SubmessageFlag(true),
            endianness_flag: EndianessFlag::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_wrong_first_last_value.is_valid(), false);
    }

    #[test]
    fn test_serialize_deserialize_heartbeat() {
        let mut writer = Vec::new();

        let reader_id = EntityId::new(EntityKey([0x10, 0x12, 0x14]), EntityKind::UserDefinedReaderWithKey);
        let writer_id = EntityId::new(EntityKey([0x26, 0x24, 0x22]), EntityKind::UserDefinedWriterWithKey);
        let first_sn = SequenceNumber(1233);
        let last_sn = SequenceNumber(1237);
        let count = Count(8);
        let is_final = true;
        let manual_liveliness = false;

        let heartbeat_big_endian = Heartbeat::new(reader_id, writer_id, first_sn, last_sn, count, is_final, manual_liveliness, EndianessFlag::BigEndian);
        heartbeat_big_endian.compose(&mut writer).unwrap();
        let submessage_big_endian = [
            0x07, 0x02, 0x00, 0x1C, // Submessage Header
            0x10, 0x12, 0x14, 0x04, // Reader ID
            0x26, 0x24, 0x22, 0x02, // Writer ID
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD1, // First Sequence Number
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD5, // Last Sequence Number
            0x00, 0x00, 0x00, 0x08, // Count
        ];

        assert_eq!(writer, submessage_big_endian);
        assert_eq!(Heartbeat::parse(&writer).unwrap(), heartbeat_big_endian);

        writer.clear();

        let heartbeat_little_endian = Heartbeat::new(reader_id, writer_id, first_sn, last_sn, count, is_final, manual_liveliness, EndianessFlag::LittleEndian);
        heartbeat_little_endian.compose(&mut writer).unwrap();
        let submessage_little_endian = [
            0x07, 0x03, 0x1C, 0x00, // Submessage Header
            0x10, 0x12, 0x14, 0x04, // Reader ID
            0x26, 0x24, 0x22, 0x02, // Writer ID
            0x00, 0x00, 0x00, 0x00, 0xD1, 0x04, 0x00, 0x00, // First Sequence Number
            0x00, 0x00, 0x00, 0x00, 0xD5, 0x04, 0x00, 0x00, // Last Sequence Number
            0x08, 0x00, 0x00, 0x00, // Count
        ];
        assert_eq!(writer, submessage_little_endian);
        assert_eq!(Heartbeat::parse(&writer).unwrap(), heartbeat_little_endian);
    }

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
