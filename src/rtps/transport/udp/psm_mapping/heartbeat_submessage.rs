use crate::rtps::messages::submessages::Heartbeat;
use crate::rtps::messages::submessages::SubmessageHeader;

use super::{UdpPsmMappingResult, };
use super::submessage_elements::{serialize_entity_id, deserialize_entity_id, serialize_sequence_number, deserialize_sequence_number, serialize_count, deserialize_count};


pub fn serialize_heartbeat(heartbeat: &Heartbeat, writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()> {
    let endianness = heartbeat.endianness_flag().into();
    
    serialize_entity_id(&heartbeat.reader_id(), writer)?;
    serialize_entity_id(&heartbeat.writer_id(), writer)?;
    serialize_sequence_number(&heartbeat.first_sn(), writer, endianness)?;
    serialize_sequence_number(&heartbeat.last_sn(), writer, endianness)?;
    serialize_count(&heartbeat.count(), writer, endianness)?;
    Ok(())
}

pub fn deserialize_heartbeat(bytes: &[u8], header: SubmessageHeader) -> UdpPsmMappingResult<Heartbeat> { 
    
    let flags = header.flags();
    // X|X|X|X|X|L|F|E
    /*E*/ let endianness_flag = flags[0];
    /*F*/ let final_flag = flags[1];
    /*L*/ let liveliness_flag = flags[2];

    let endianness = endianness_flag.into();

    const HEADER_SIZE : usize = 8;
    let reader_id = deserialize_entity_id(&bytes[0..4])?;
    let writer_id = deserialize_entity_id(&bytes[4..8])?;
    let first_sn = deserialize_sequence_number(&bytes[8..16], endianness)?;
    let last_sn = deserialize_sequence_number(&bytes[16..24], endianness)?;
    let count = deserialize_count(&bytes[24..28], endianness)?;
    

    Ok(Heartbeat::from_raw_parts(endianness_flag, final_flag, liveliness_flag, reader_id, writer_id, first_sn, last_sn, count))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps::types::{EntityId, constants};
    use crate::rtps::messages::types::Endianness;
    use crate::rtps::messages::submessages::Submessage;

    #[test]
    fn test_serialize_deserialize_heartbeat() {
        let mut writer = Vec::new();

        let reader_id = EntityId::new([0x10, 0x12, 0x14], constants::ENTITY_KIND_USER_DEFINED_READER_WITH_KEY);
        let writer_id = EntityId::new([0x26, 0x24, 0x22], constants::ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY);
        let first_sn = 1233;
        let last_sn = 1237;
        let count = 8;
        let is_final = true;
        let manual_liveliness = false;

        let heartbeat_big_endian = Heartbeat::new(
            Endianness::BigEndian,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
            is_final,
            manual_liveliness
        );

        serialize_heartbeat(&heartbeat_big_endian, &mut writer).unwrap();
        let submessage_big_endian = [
            // 0x07, 0x02, 0x00, 0x1C, // Submessage Header
            0x10, 0x12, 0x14, 0x04, // Reader ID
            0x26, 0x24, 0x22, 0x02, // Writer ID
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD1, // First Sequence Number
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD5, // Last Sequence Number
            0x00, 0x00, 0x00, 0x08, // Count
        ];

        assert_eq!(writer, submessage_big_endian);
        assert_eq!(deserialize_heartbeat(&writer, heartbeat_big_endian.submessage_header(submessage_big_endian.len() as u16)).unwrap(), heartbeat_big_endian);

        writer.clear();

        let heartbeat_little_endian = Heartbeat::new(
            Endianness::LittleEndian,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
            is_final,
            manual_liveliness
        );

        serialize_heartbeat(&heartbeat_little_endian, &mut writer).unwrap();
        let submessage_little_endian = [
            // 0x07, 0x03, 0x1C, 0x00, // Submessage Header
            0x10, 0x12, 0x14, 0x04, // Reader ID
            0x26, 0x24, 0x22, 0x02, // Writer ID
            0x00, 0x00, 0x00, 0x00, 0xD1, 0x04, 0x00, 0x00, // First Sequence Number
            0x00, 0x00, 0x00, 0x00, 0xD5, 0x04, 0x00, 0x00, // Last Sequence Number
            0x08, 0x00, 0x00, 0x00, // Count
        ];
        assert_eq!(writer, submessage_little_endian);
        assert_eq!(deserialize_heartbeat(&writer, heartbeat_little_endian.submessage_header(submessage_little_endian.len() as u16)).unwrap(), heartbeat_little_endian);
    }
}
