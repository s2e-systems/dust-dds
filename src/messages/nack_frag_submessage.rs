use crate::types::{Count, EntityId, FragmentNumberSet, SequenceNumber};

use super::helpers::{parse_fragment_number_set, SequenceNumberSerialization};

use super::{deserialize, endianess, RtpsMessageResult};

#[derive(PartialEq, Debug)]
pub struct NackFrag {
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    fragment_number_set: FragmentNumberSet,
    count: Count,
}

pub fn parse_nack_frag_submessage(submessage: &[u8], submessage_flags: &u8) -> RtpsMessageResult<NackFrag> {
    const READER_ID_FIRST_INDEX: usize = 0;
    const READER_ID_LAST_INDEX: usize = 3;
    const WRITER_ID_FIRST_INDEX: usize = 4;
    const WRITER_ID_LAST_INDEX: usize = 7;
    const SEQUENCE_NUMBER_FIRST_INDEX: usize = 8;
    const SEQUENCE_NUMBER_LAST_INDEX: usize = 15;
    const FRAGMENT_NUMBER_SET_FIRST_INDEX: usize = 16;
    const COUNT_SIZE: usize = 4;

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

    let writer_sn: SequenceNumber = deserialize::<SequenceNumberSerialization>(
        submessage,
        &SEQUENCE_NUMBER_FIRST_INDEX,
        &SEQUENCE_NUMBER_LAST_INDEX,
        &submessage_endianess,
    )?
    .into();

    let (fragment_number_set, fragment_number_set_size) = parse_fragment_number_set(
        submessage,
        &FRAGMENT_NUMBER_SET_FIRST_INDEX,
        &submessage_endianess,
    )?;

    let count_first_index = FRAGMENT_NUMBER_SET_FIRST_INDEX + fragment_number_set_size;
    let count_last_index = count_first_index + COUNT_SIZE - 1;

    let count = deserialize::<Count>(
        submessage,
        &count_first_index,
        &count_last_index,
        &submessage_endianess,
    )?;

    Ok(NackFrag {
        reader_id,
        writer_id,
        writer_sn,
        fragment_number_set,
        count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nack_frag_submessage_big_endian() {
        let submessage = [
            0x01, 0x02, 0x03, 0x04, 0x10, 0x11, 0x12, 0x13, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x11, 0x22, 0x00, 0x00, 0x10, 0x11, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x03, 0x0F,
            0x00, 0x00, 0x11, 0x22,
        ];
        let nack_frag_submessage = parse_nack_frag_submessage(&submessage, &0).unwrap();
        assert_eq!(
            nack_frag_submessage.reader_id,
            EntityId::new([0x01, 0x02, 0x03], 0x04)
        );
        assert_eq!(
            nack_frag_submessage.writer_id,
            EntityId::new([0x10, 0x11, 0x12], 0x13)
        );
        assert_eq!(nack_frag_submessage.writer_sn, 4294971682);
        assert_eq!(nack_frag_submessage.fragment_number_set.len(), 10);
        assert_eq!(
            nack_frag_submessage.fragment_number_set,
            vec!(
                (4113, true),
                (4114, true),
                (4115, true),
                (4116, true),
                (4117, false),
                (4118, false),
                (4119, false),
                (4120, false),
                (4121, true),
                (4122, true)
            )
        );
        assert_eq!(nack_frag_submessage.count, 4386);
    }

    #[test]
    fn test_parse_nack_frag_submessage_little_endian() {
        let submessage = [
            0x01, 0x02, 0x03, 0x04, 0x10, 0x11, 0x12, 0x13, 0x01, 0x00, 0x00, 0x00, 0x22, 0x11,
            0x00, 0x00, 0x11, 0x10, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x0F, 0x03, 0x00, 0x00,
            0x22, 0x11, 0x00, 0x00,
        ];
        let nack_frag_submessage = parse_nack_frag_submessage(&submessage, &1).unwrap();
        assert_eq!(
            nack_frag_submessage.reader_id,
            EntityId::new([0x01, 0x02, 0x03], 0x04)
        );
        assert_eq!(
            nack_frag_submessage.writer_id,
            EntityId::new([0x10, 0x11, 0x12], 0x13)
        );
        assert_eq!(nack_frag_submessage.writer_sn, 4294971682);
        assert_eq!(nack_frag_submessage.fragment_number_set.len(), 10);
        assert_eq!(
            nack_frag_submessage.fragment_number_set,
            vec!(
                (4113, true),
                (4114, true),
                (4115, true),
                (4116, true),
                (4117, false),
                (4118, false),
                (4119, false),
                (4120, false),
                (4121, true),
                (4122, true)
            )
        );
        assert_eq!(nack_frag_submessage.count, 4386);
    }
}
