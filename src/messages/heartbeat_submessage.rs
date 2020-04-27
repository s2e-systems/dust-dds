use crate::types::{Count, EntityId, SequenceNumber};

use super::helpers::{deserialize, endianess, SequenceNumberSerialization};

use super::{RtpsMessageError, RtpsMessageResult};

#[derive(PartialEq, Debug)]
pub struct Heartbeat {
    reader_id: EntityId,
    writer_id: EntityId,
    first_sn: SequenceNumber,
    last_sn: SequenceNumber,
    count: Count,
    final_flag: bool,
    liveliness_flag: bool,
}

pub fn parse_heartbeat_submessage(submessage: &[u8], submessage_flags: &u8) -> RtpsMessageResult<Heartbeat> {
    const READER_ID_FIRST_INDEX: usize = 0;
    const READER_ID_LAST_INDEX: usize = 3;
    const WRITER_ID_FIRST_INDEX: usize = 4;
    const WRITER_ID_LAST_INDEX: usize = 7;
    const FIRST_SN_FIRST_INDEX: usize = 8;
    const FIRST_SN_LAST_INDEX: usize = 15;
    const LAST_SN_FIRST_INDEX: usize = 16;
    const LAST_SN_LAST_INDEX: usize = 23;
    const COUNT_FIRST_INDEX: usize = 24;
    const COUNT_LAST_INDEX: usize = 27;

    const FINAL_FLAG_MASK: u8 = 0x02;
    const LIVELINESS_FLAG_MASK: u8 = 0x04;

    let submessage_endianess = endianess(submessage_flags)?;
    let final_flag = (submessage_flags & FINAL_FLAG_MASK) == FINAL_FLAG_MASK;
    let liveliness_flag = (submessage_flags & LIVELINESS_FLAG_MASK) == LIVELINESS_FLAG_MASK;

    let reader_id = deserialize::<EntityId>(
        submessage,
        &READER_ID_FIRST_INDEX,
        &READER_ID_LAST_INDEX,
        &submessage_endianess,
    )?;

    let writer_id = deserialize::<EntityId>(
        submessage,
        &WRITER_ID_FIRST_INDEX,
        &WRITER_ID_LAST_INDEX,
        &submessage_endianess,
    )?;

    let first_sn: SequenceNumber = deserialize::<SequenceNumberSerialization>(
        submessage,
        &FIRST_SN_FIRST_INDEX,
        &FIRST_SN_LAST_INDEX,
        &submessage_endianess,
    )?
    .into();
    if first_sn < 1 {
        return Err(RtpsMessageError::InvalidSubmessage);
    }

    let last_sn: SequenceNumber = deserialize::<SequenceNumberSerialization>(
        submessage,
        &LAST_SN_FIRST_INDEX,
        &LAST_SN_LAST_INDEX,
        &submessage_endianess,
    )?
    .into();
    if last_sn < 0 {
        return Err(RtpsMessageError::InvalidSubmessage);
    }

    if last_sn < first_sn - 1 {
        return Err(RtpsMessageError::InvalidSubmessage);
    }

    let count = deserialize::<Count>(
        submessage,
        &COUNT_FIRST_INDEX,
        &COUNT_LAST_INDEX,
        &submessage_endianess,
    )?;

    Ok(Heartbeat {
        reader_id,
        writer_id,
        first_sn,
        last_sn,
        count,
        final_flag,
        liveliness_flag,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heartbeat_submessage_big_endian() {
        let submessage_big_endian = [
            0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x04, 0xD1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD5, 0x00, 0x00, 0x00, 0x08,
        ];

        let heartbeat_big_endian = parse_heartbeat_submessage(&submessage_big_endian, &0).unwrap();
        assert_eq!(
            heartbeat_big_endian.reader_id,
            EntityId::new([0x10, 0x12, 0x14], 0x16)
        );
        assert_eq!(
            heartbeat_big_endian.writer_id,
            EntityId::new([0x26, 0x24, 0x22], 0x20)
        );
        assert_eq!(heartbeat_big_endian.first_sn, 1233);
        assert_eq!(heartbeat_big_endian.last_sn, 1237);
        assert_eq!(heartbeat_big_endian.count, 8);
        assert_eq!(heartbeat_big_endian.final_flag, false);
        assert_eq!(heartbeat_big_endian.liveliness_flag, false);
    }

    #[test]
    fn test_parse_heartbeat_submessage_little_endian() {
        let submessage_little_endian = [
            0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0xD1, 0x04,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD5, 0x04, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
        ];

        let heartbeat_little_endian =
            parse_heartbeat_submessage(&submessage_little_endian, &7).unwrap();
        assert_eq!(
            heartbeat_little_endian.reader_id,
            EntityId::new([0x10, 0x12, 0x14], 0x16)
        );
        assert_eq!(
            heartbeat_little_endian.writer_id,
            EntityId::new([0x26, 0x24, 0x22], 0x20)
        );
        assert_eq!(heartbeat_little_endian.first_sn, 1233);
        assert_eq!(heartbeat_little_endian.last_sn, 1237);
        assert_eq!(heartbeat_little_endian.count, 8);
        assert_eq!(heartbeat_little_endian.final_flag, true);
        assert_eq!(heartbeat_little_endian.liveliness_flag, true);
    }

    #[test]
    fn test_parse_heartbeat_submessage_first_sn_equal_one() {
        // Test first
        let submessage_big_endian = [
            0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08,
        ];

        let heartbeat_big_endian = parse_heartbeat_submessage(&submessage_big_endian, &2).unwrap();
        assert_eq!(
            heartbeat_big_endian.reader_id,
            EntityId::new([0x10, 0x12, 0x14], 0x16)
        );
        assert_eq!(
            heartbeat_big_endian.writer_id,
            EntityId::new([0x26, 0x24, 0x22], 0x20)
        );
        assert_eq!(heartbeat_big_endian.first_sn, 1);
        assert_eq!(heartbeat_big_endian.last_sn, 0);
        assert_eq!(heartbeat_big_endian.count, 8);
        assert_eq!(heartbeat_big_endian.final_flag, true);
        assert_eq!(heartbeat_big_endian.liveliness_flag, false);
    }

    #[test]
    fn test_parse_heartbeat_submessage_invalid_submessage_last_sn_below_first_sn() {
        // Test last_sn < first_sn - 1
        let submessage = [
            0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x04, 0xD1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xCF, 0x00, 0x00, 0x00, 0x08,
        ];

        let heartbeat = parse_heartbeat_submessage(&submessage, &0);
        if let Err(RtpsMessageError::InvalidSubmessage) = heartbeat {
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parse_heartbeat_submessage_invalid_submessage_first_sn_zero() {
        // Test first_sn = 0
        let submessage = [
            0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD5, 0x00, 0x00, 0x00, 0x08,
        ];

        let heartbeat = parse_heartbeat_submessage(&submessage, &0);
        if let Err(RtpsMessageError::InvalidSubmessage) = heartbeat {
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parse_heartbeat_submessage_invalid_submessage_last_sn_below_zero() {
        // Test last_sn < 0
        let submessage = [
            0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x04, 0xD1, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD5, 0x00, 0x00, 0x00, 0x08,
        ];

        let heartbeat = parse_heartbeat_submessage(&submessage, &0);
        if let Err(RtpsMessageError::InvalidSubmessage) = heartbeat {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
