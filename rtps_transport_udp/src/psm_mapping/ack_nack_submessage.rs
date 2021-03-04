use rust_rtps::messages::submessages::{AckNack, SubmessageHeader};

use super::submessage_elements::{
    deserialize_count, deserialize_entity_id, deserialize_sequence_number_set, serialize_count,
    serialize_entity_id, serialize_sequence_number_set,
};
use super::UdpPsmMappingResult;

pub fn serialize_ack_nack(
    ack_nack: &AckNack,
    writer: &mut impl std::io::Write,
) -> UdpPsmMappingResult<()> {
    let endianness = ack_nack.endianness_flag().into();
    serialize_entity_id(&ack_nack.reader_id(), writer)?;
    serialize_entity_id(&ack_nack.writer_id(), writer)?;
    serialize_sequence_number_set(ack_nack.reader_sn_state(), writer, endianness)?;
    serialize_count(&ack_nack.count(), writer, endianness)?;
    Ok(())
}

pub fn deserialize_ack_nack(
    bytes: &[u8],
    header: SubmessageHeader,
) -> UdpPsmMappingResult<AckNack> {
    let flags = header.flags();

    let endianness_flag = flags[0];
    let final_flag = flags[1];

    let endianness = endianness_flag.into();

    let end_of_message = usize::from(header.submessage_length());
    let index_count = end_of_message - 4;
    let reader_id = deserialize_entity_id(&bytes[0..4])?;
    let writer_id = deserialize_entity_id(&bytes[4..8])?;
    let reader_sn_state = deserialize_sequence_number_set(&bytes[8..index_count], endianness)?;
    let count = deserialize_count(&bytes[index_count..end_of_message], endianness)?;

    Ok(AckNack::from_raw_parts(
        endianness_flag,
        final_flag,
        reader_id,
        writer_id,
        reader_sn_state,
        count,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_rtps::messages::submessages::Submessage;
    use rust_rtps::messages::types::Endianness;
    use rust_rtps::types::constants::{
        ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, ENTITYID_UNKNOWN,
    };

    #[test]
    fn test_deserialize_ack_nack_submessage() {
        let bytes = [
            // 0x0f, 0b00000011, 28, 0,
            0x00,
            0x00,
            0x00,
            0x00, // readerId
            0x00,
            0x01,
            0x00,
            0xc2, // writerId
            0,
            0,
            0,
            0, // reader_sn_state: base
            2,
            0,
            0,
            0, // reader_sn_state: base
            2,
            0,
            0,
            0, // reader_sn_state: num bits
            0b_00000000,
            0b_00000000,
            0b_00000000,
            0b_11000000, // reader_sn_state: bitmap
            2,
            0,
            0,
            0, // Count
        ];

        let expected = AckNack::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            2,
            &[2, 3],
            2,
            true,
        );
        let result =
            deserialize_ack_nack(&bytes, expected.submessage_header(bytes.len() as u16)).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_serialize_ack_nack_submessage() {
        let expected = vec![
            // 0x0f, 0b00000011, 28, 0,
            0x00,
            0x00,
            0x00,
            0x00, // readerId
            0x00,
            0x01,
            0x00,
            0xc2, // writerId
            0,
            0,
            0,
            0, // reader_sn_state: base
            2,
            0,
            0,
            0, // reader_sn_state: base
            2,
            0,
            0,
            0, // reader_sn_state: num bits
            0b_00000000,
            0b_00000000,
            0b_00000000,
            0b_11000000, // reader_sn_state: bitmap
            2,
            0,
            0,
            0, // Count
        ];

        let ack_nack = AckNack::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            2,
            &[2, 3],
            2,
            true,
        );

        let mut writer = Vec::new();
        serialize_ack_nack(&ack_nack, &mut writer).unwrap();
        assert_eq!(expected, writer);
    }
}
