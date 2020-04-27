use crate::types::{
    EntityId, FragmentNumber, InlineQosParameterList, SequenceNumber,
};

use super::helpers::{
    deserialize, endianess, parse_inline_qos_parameter_list, SequenceNumberSerialization,
};

use super::{RtpsMessageError, Payload, RtpsMessageResult};

#[derive(PartialEq, Debug)]
pub struct DataFrag {
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    fragment_starting_num: FragmentNumber,
    fragments_in_submessage: u16,
    fragment_size: u16,
    sample_size: u32,
    inline_qos: Option<InlineQosParameterList>,
    serialized_payload: Payload,
}

pub fn parse_data_frag_submessage(submessage: &[u8], submessage_flags: &u8) -> RtpsMessageResult<DataFrag> {
    const INLINE_QOS_FLAG_MASK: u8 = 0x02;
    const KEY_FLAG_MASK: u8 = 0x04;
    const NON_STANDARD_PAYLOAD_FLAG_MASK: u8 = 0x08;

    const EXTRA_FLAGS_FIRST_INDEX: usize = 0;
    const EXTRA_FLAGS_LAST_INDEX: usize = 1;
    const OCTETS_TO_INLINE_QOS_FIRST_INDEX: usize = 2;
    const OCTETS_TO_INLINE_QOS_LAST_INDEX: usize = 3;
    const READER_ID_FIRST_INDEX: usize = 4;
    const READER_ID_LAST_INDEX: usize = 7;
    const WRITER_ID_FIRST_INDEX: usize = 8;
    const WRITER_ID_LAST_INDEX: usize = 11;
    const WRITER_SN_FIRST_INDEX: usize = 12;
    const WRITER_SN_LAST_INDEX: usize = 19;
    const FRAGMENT_STARTING_NUMBER_FIRST_INDEX: usize = 20;
    const FRAGMENT_STARTING_NUMBER_LAST_INDEX: usize = 23;
    const FRAGMENTS_IN_SUBMESSAGE_FIRST_INDEX: usize = 24;
    const FRAGMENTS_IN_SUBMESSAGE_LAST_INDEX: usize = 25;
    const FRAGMENT_SIZE_FIRST_INDEX: usize = 26;
    const FRAGMENT_SIZE_LAST_INDEX: usize = 27;
    const SAMPLE_SIZE_FIRST_INDEX: usize = 28;
    const SAMPLE_SIZE_LAST_INDEX: usize = 31;

    let submessage_endianess = endianess(submessage_flags)?;
    let inline_qos_flag = submessage_flags & INLINE_QOS_FLAG_MASK == INLINE_QOS_FLAG_MASK;
    let key_flag = submessage_flags & KEY_FLAG_MASK == KEY_FLAG_MASK;

    // TODO: Implement non-standard payload
    let _non_standard_payload_flag =
        submessage_flags & NON_STANDARD_PAYLOAD_FLAG_MASK == NON_STANDARD_PAYLOAD_FLAG_MASK;

    let extra_flags = deserialize::<u16>(
        submessage,
        &EXTRA_FLAGS_FIRST_INDEX,
        &EXTRA_FLAGS_LAST_INDEX,
        &submessage_endianess,
    )?;
    if extra_flags != 0 {
        return Err(RtpsMessageError::InvalidSubmessage);
    }

    let octecs_to_inline_qos = deserialize::<u16>(
        submessage,
        &OCTETS_TO_INLINE_QOS_FIRST_INDEX,
        &OCTETS_TO_INLINE_QOS_LAST_INDEX,
        &submessage_endianess,
    )? as usize;

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
        &WRITER_SN_FIRST_INDEX,
        &WRITER_SN_LAST_INDEX,
        &submessage_endianess,
    )?
    .into();

    let fragment_starting_num: FragmentNumber = deserialize::<FragmentNumber>(
        submessage,
        &FRAGMENT_STARTING_NUMBER_FIRST_INDEX,
        &FRAGMENT_STARTING_NUMBER_LAST_INDEX,
        &submessage_endianess,
    )?;

    let fragments_in_submessage: u16 = deserialize::<u16>(
        submessage,
        &FRAGMENTS_IN_SUBMESSAGE_FIRST_INDEX,
        &FRAGMENTS_IN_SUBMESSAGE_LAST_INDEX,
        &submessage_endianess,
    )?;

    let fragment_size: u16 = deserialize::<u16>(
        submessage,
        &FRAGMENT_SIZE_FIRST_INDEX,
        &FRAGMENT_SIZE_LAST_INDEX,
        &submessage_endianess,
    )?;

    let sample_size: u32 = deserialize::<u32>(
        submessage,
        &SAMPLE_SIZE_FIRST_INDEX,
        &SAMPLE_SIZE_LAST_INDEX,
        &submessage_endianess,
    )?;

    // Octets to data is considered as having the same meaning as octets to inline qos,
    // i.e. counting from the byte after the octets to inline qos field

    let (inline_qos, octets_to_data) = if inline_qos_flag {
        let inline_qos_first_index = OCTETS_TO_INLINE_QOS_LAST_INDEX + octecs_to_inline_qos + 1;
        let (parameter_list, parameter_list_size) = parse_inline_qos_parameter_list(
            submessage,
            &inline_qos_first_index,
            &submessage_endianess,
        )?;
        let octets_to_data = octecs_to_inline_qos + parameter_list_size;
        (Some(parameter_list), octets_to_data)
    } else {
        (None, octecs_to_inline_qos)
    };

    let payload_first_index = OCTETS_TO_INLINE_QOS_LAST_INDEX + octets_to_data + 1;

    let serialized_payload = if !key_flag {
        Payload::Data(submessage[payload_first_index..].to_vec())
    } else {
        Payload::Key(submessage[payload_first_index..].to_vec())
    };

    Ok(DataFrag {
        reader_id,
        writer_id,
        writer_sn,
        fragment_starting_num,
        fragments_in_submessage,
        fragment_size,
        sample_size,
        inline_qos,
        serialized_payload,
    })
}

// #[cfg(test)]
// mod tests{
//     use super::*;

//     #[test]
//     fn test_parse_data_frag_submessage() {
//         parse_data_frag_submessage(&[0,0], &0).unwrap();
//     }
// }
