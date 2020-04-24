use crate::types::{EntityId, SequenceNumber, SequenceNumberSet};

use super::helpers::{
    deserialize, endianess, parse_sequence_number_set, SequenceNumberSerialization,
};

use super::{ErrorMessage, Result};

#[derive(PartialEq, Debug)]
pub struct Gap {
    reader_id: EntityId,
    writer_id: EntityId,
    gap_start: SequenceNumber,
    gap_list: SequenceNumberSet,
}

impl Gap {
    pub fn new(
        reader_id: EntityId,
        writer_id: EntityId,
        gap_start: SequenceNumber,
        gap_list: SequenceNumberSet,) -> Self {
            Gap {
                reader_id,
                writer_id,
                gap_start,
                gap_list,
            }
        }
}

pub fn parse_gap_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<Gap> {
    const READER_ID_FIRST_INDEX: usize = 0;
    const READER_ID_LAST_INDEX: usize = 3;
    const WRITER_ID_FIRST_INDEX: usize = 4;
    const WRITER_ID_LAST_INDEX: usize = 7;
    const GAP_START_FIRST_INDEX: usize = 8;
    const GAP_START_LAST_INDEX: usize = 15;
    const GAP_LIST_FIRST_INDEX: usize = 16;

    let submessage_endianess = endianess(submessage_flags)?;

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

    let gap_start: i64 = deserialize::<SequenceNumberSerialization>(
        submessage,
        &GAP_START_FIRST_INDEX,
        &GAP_START_LAST_INDEX,
        &submessage_endianess,
    )?
    .into();
    if gap_start < 1 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let (gap_list, _sequence_number_set_size) =
        parse_sequence_number_set(submessage, &GAP_LIST_FIRST_INDEX, &submessage_endianess)?;

    // TODO: The GAP message in the PSM is not matching the description given in the PIM. Have to check for that.

    Ok(Gap {
        reader_id,
        writer_id,
        gap_start,
        gap_list,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gap_submessage_big_endian() {
        let submessage_big_endian = [
            0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x04, 0xD1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD2, 0x00, 0x00, 0x00, 0x08,
            0x00, 0x00, 0x00, 0x0C,
        ];

        let gap_big_endian = parse_gap_submessage(&submessage_big_endian, &0).unwrap();

        assert_eq!(
            gap_big_endian.reader_id,
            EntityId::new([0x10, 0x12, 0x14], 0x16)
        );
        assert_eq!(
            gap_big_endian.writer_id,
            EntityId::new([0x26, 0x24, 0x22], 0x20)
        );
        assert_eq!(gap_big_endian.gap_start, 1233);
        assert_eq!(gap_big_endian.gap_list.len(), 8);
        assert_eq!(
            gap_big_endian.gap_list,
            [
                (1234, false),
                (1235, false),
                (1236, true),
                (1237, true),
                (1238, false),
                (1239, false),
                (1240, false),
                (1241, false)
            ].iter().cloned().collect()
        );
    }

    #[test]
    fn test_parse_gap_submessage_little_endian() {
        let submessage_little_endian = [
            0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00, 0x00, 0xD1, 0x04,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD2, 0x04, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x0C, 0x00, 0x00, 0x00,
        ];

        let gap_little_endian = parse_gap_submessage(&submessage_little_endian, &1).unwrap();

        assert_eq!(
            gap_little_endian.reader_id,
            EntityId::new([0x10, 0x12, 0x14], 0x16)
        );
        assert_eq!(
            gap_little_endian.writer_id,
            EntityId::new([0x26, 0x24, 0x22], 0x20)
        );
        assert_eq!(gap_little_endian.gap_start, 1233);
        assert_eq!(gap_little_endian.gap_list.len(), 8);
        assert_eq!(
            gap_little_endian.gap_list,
            [
                (1234, false),
                (1235, false),
                (1236, true),
                (1237, true),
                (1238, false),
                (1239, false),
                (1240, false),
                (1241, false)
            ].iter().cloned().collect()
        );
    }

    #[test]
    fn test_parse_gap_submessage_invalid() {
        let submessage_big_endian = [
            0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x04, 0xD1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD2, 0x00, 0x00, 0x00, 0x08,
            0x00, 0x00, 0x00, 0x0C,
        ];

        let gap_big_endian = parse_gap_submessage(&submessage_big_endian, &0);

        if let Err(ErrorMessage::InvalidSubmessage) = gap_big_endian {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
