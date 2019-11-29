use crate::types::{EntityId,FragmentNumber,Count, SequenceNumber};

use super::helpers::{deserialize, endianess, SequenceNumberSerialization};

use super::{Result, ErrorMessage};

#[derive(PartialEq, Debug)]
pub struct HeartbeatFrag {
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    last_fragment_num: FragmentNumber,
    count: Count,
}

pub fn parse_heartbeat_frag_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<HeartbeatFrag> {
    const READER_ID_FIRST_INDEX: usize = 0;
    const READER_ID_LAST_INDEX: usize = 3;
    const WRITER_ID_FIRST_INDEX: usize = 4;
    const WRITER_ID_LAST_INDEX: usize = 7;
    const WRITER_SN_FIRST_INDEX: usize = 8;
    const WRITER_SN_LAST_INDEX: usize = 15;
    const FRAGMENT_NUMBER_FIRST_INDEX: usize = 16;
    const FRAGMENT_NUMBER_LAST_INDEX: usize = 19;
    const COUNT_FIRST_INDEX: usize = 20;
    const COUNT_LAST_INDEX: usize = 23;
    
    let submessage_endianess = endianess(submessage_flags)?;

    let reader_id = deserialize::<EntityId>(submessage, &READER_ID_FIRST_INDEX, &READER_ID_LAST_INDEX, &submessage_endianess)?;
    
    let writer_id = deserialize::<EntityId>(submessage, &WRITER_ID_FIRST_INDEX, &WRITER_ID_LAST_INDEX, &submessage_endianess)?;

    let writer_sn : SequenceNumber = deserialize::<SequenceNumberSerialization>(submessage, &WRITER_SN_FIRST_INDEX, &WRITER_SN_LAST_INDEX, &submessage_endianess)?.into();
    if writer_sn < 1 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let last_fragment_num = deserialize::<FragmentNumber>(submessage, &FRAGMENT_NUMBER_FIRST_INDEX, &FRAGMENT_NUMBER_LAST_INDEX, &submessage_endianess)?;

    let count = deserialize::<Count>(submessage, &COUNT_FIRST_INDEX, &COUNT_LAST_INDEX, &submessage_endianess)?;

    Ok(HeartbeatFrag{
        reader_id,
        writer_id,
        writer_sn,
        last_fragment_num,
        count
    })
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_parse_heartbeat_frag_submessage_big_endian() {
        let submessage_big_endian = [
                0x10,0x11,0x12,0x13,
                0x26,0x25,0x24,0x23,
                0x00,0x00,0x10,0x01,
                0x01,0x02,0x03,0x04,
                0x00,0x05,0x70,0x10,
                0x00,0x00,0x00,0x05,
            ];

        let heartbeat_frag = parse_heartbeat_frag_submessage(&submessage_big_endian, &0).unwrap();

        assert_eq!(heartbeat_frag.reader_id, EntityId::new([0x10,0x11,0x12],0x13));
        assert_eq!(heartbeat_frag.writer_id, EntityId::new([0x26,0x25,0x24],0x23));
        assert_eq!(heartbeat_frag.writer_sn, 17_596_497_920_772);
        assert_eq!(heartbeat_frag.last_fragment_num, 356_368);
        assert_eq!(heartbeat_frag.count, 5);
    }

    #[test]
    fn test_parse_heartbeat_frag_submessage_little_endian() {
        let submessage_little_endian = [
            0x10,0x11,0x12,0x13,
            0x26,0x25,0x24,0x23,
            0x01,0x10,0x00,0x00,
            0x04,0x03,0x02,0x01,
            0x10,0x70,0x05,0x00,
            0x05,0x00,0x00,0x00,
        ];

        let heartbeat_frag = parse_heartbeat_frag_submessage(&submessage_little_endian, &1).unwrap();

        assert_eq!(heartbeat_frag.reader_id, EntityId::new([0x10,0x11,0x12],0x13));
        assert_eq!(heartbeat_frag.writer_id, EntityId::new([0x26,0x25,0x24],0x23));
        assert_eq!(heartbeat_frag.writer_sn, 17_596_497_920_772);
        assert_eq!(heartbeat_frag.last_fragment_num, 356_368);
        assert_eq!(heartbeat_frag.count, 5);
    }
}