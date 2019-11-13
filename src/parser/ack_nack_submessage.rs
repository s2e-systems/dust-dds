use crate::types::{EntityId,Count, SequenceNumberSet};

use super::helpers::{deserialize, endianess, parse_sequence_number_set};

use super::{Result};

#[derive(PartialEq, Debug)]
pub struct AckNack {
    final_flag: bool,
    reader_id: EntityId,
    writer_id: EntityId,
    reader_sn_state: SequenceNumberSet,
    count: Count,
}

pub fn parse_ack_nack_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<AckNack> {
    const FINAL_FLAG_MASK: u8 = 0x02;
    const READER_ID_FIRST_INDEX: usize = 0;
    const READER_ID_LAST_INDEX: usize = 3;
    const WRITER_ID_FIRST_INDEX: usize = 4;
    const WRITER_ID_LAST_INDEX: usize = 7;
    const SEQUENCE_NUMBER_SET_FIRST_INDEX: usize = 8;
    const COUNT_SIZE: usize = 4;

    let submessage_endianess = endianess(submessage_flags)?;
    let final_flag = (submessage_flags & FINAL_FLAG_MASK) == FINAL_FLAG_MASK;

    let reader_id = deserialize::<EntityId>(submessage, &READER_ID_FIRST_INDEX, &READER_ID_LAST_INDEX, &submessage_endianess)?;
    
    let writer_id = deserialize::<EntityId>(submessage, &WRITER_ID_FIRST_INDEX, &WRITER_ID_LAST_INDEX, &submessage_endianess)?;
    
    let (reader_sn_state, sequence_number_set_size) = parse_sequence_number_set(submessage, &SEQUENCE_NUMBER_SET_FIRST_INDEX, &submessage_endianess)?;

    let count_first_index = SEQUENCE_NUMBER_SET_FIRST_INDEX + sequence_number_set_size;
    let count_last_index = count_first_index + COUNT_SIZE - 1;

    let count = deserialize::<Count>(submessage, &count_first_index, &count_last_index, &submessage_endianess)?;

    Ok( AckNack {
        final_flag,
        reader_id,
        writer_id,
        reader_sn_state,
        count,
    })
}

#[cfg(test)]
mod tests{
    use super::*;
    
    #[test]
    fn test_parse_ack_nack_submessage() {
        {
            let ack_nack_submessage_big_endian = [
            0x10,0x12,0x14,0x16,
            0x26,0x24,0x22,0x20,
            0x00,0x00,0x00,0x00,
            0x00,0x00,0x04,0xD2,
            0x00,0x00,0x00,0x08,
            0x00,0x00,0x00,0x0C,
            0x00,0x00,0x00,0x0F,
            ];

            let ack_nack_big_endian = parse_ack_nack_submessage(&ack_nack_submessage_big_endian, &0).unwrap();
            assert_eq!(ack_nack_big_endian.final_flag, false);
            assert_eq!(ack_nack_big_endian.reader_id, [0x10,0x12,0x14,0x16,]);
            assert_eq!(ack_nack_big_endian.writer_id, [0x26,0x24,0x22,0x20,]);
            assert_eq!(ack_nack_big_endian.count, 15);
            assert_eq!(ack_nack_big_endian.reader_sn_state,
                vec![(1234, false),(1235, false), (1236, true), (1237, true),
                    (1238, false),(1239, false), (1240, false), (1241, false),] );

            let ack_nack_big_endian_final = parse_ack_nack_submessage(&ack_nack_submessage_big_endian, &2).unwrap();
            assert_eq!(ack_nack_big_endian_final.final_flag, true);
            assert_eq!(ack_nack_big_endian_final.reader_id, [0x10,0x12,0x14,0x16,]);
            assert_eq!(ack_nack_big_endian_final.writer_id, [0x26,0x24,0x22,0x20,]);
            assert_eq!(ack_nack_big_endian_final.count, 15);
            assert_eq!(ack_nack_big_endian_final.reader_sn_state,
                vec![(1234, false),(1235, false), (1236, true), (1237, true),
                    (1238, false),(1239, false), (1240, false), (1241, false),] );
        }

        {
            let ack_nack_submessage_little_endian = [
            0x10,0x12,0x14,0x16,
            0x26,0x24,0x22,0x20,
            0x00,0x00,0x00,0x00,
            0xD2,0x04,0x00,0x00,
            0x08,0x00,0x00,0x00,
            0x0C,0x00,0x00,0x00,
            0x0F,0x00,0x00,0x00,
            ];

            let ack_nack_little_endian = parse_ack_nack_submessage(&ack_nack_submessage_little_endian, &1).unwrap();
            assert_eq!(ack_nack_little_endian.final_flag, false);
            assert_eq!(ack_nack_little_endian.reader_id, [0x10,0x12,0x14,0x16,]);
            assert_eq!(ack_nack_little_endian.writer_id, [0x26,0x24,0x22,0x20,]);
            assert_eq!(ack_nack_little_endian.count, 15);
            assert_eq!(ack_nack_little_endian.reader_sn_state,
                vec![(1234, false),(1235, false), (1236, true), (1237, true),
                    (1238, false),(1239, false), (1240, false), (1241, false),] );

            let ack_nack_little_endian_final = parse_ack_nack_submessage(&ack_nack_submessage_little_endian, &3).unwrap();
            assert_eq!(ack_nack_little_endian_final.final_flag, true);
            assert_eq!(ack_nack_little_endian_final.reader_id, [0x10,0x12,0x14,0x16,]);
            assert_eq!(ack_nack_little_endian_final.writer_id, [0x26,0x24,0x22,0x20,]);
            assert_eq!(ack_nack_little_endian_final.count, 15);
            assert_eq!(ack_nack_little_endian_final.reader_sn_state,
                vec![(1234, false),(1235, false), (1236, true), (1237, true),
                    (1238, false),(1239, false), (1240, false), (1241, false),] );
        }   
    }
}