impl UdpPsmMapping for Heartbeat {
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

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> { 
        let header = SubmessageHeader::parse(bytes)?;
        let flags = header.flags();
        // X|X|X|X|X|L|F|E
        /*E*/ let endianness_flag = flags[0];
        /*F*/ let final_flag = flags[1];
        /*L*/ let liveliness_flag = flags[2];

        let endianness = Endianness::from(endianness_flag);

        const HEADER_SIZE : usize = 8;
        let reader_id = submessage_elements::EntityId::deserialize(&bytes[4..8], endianness)?;
        let writer_id = submessage_elements::EntityId::deserialize(&bytes[8..12], endianness)?;
        let first_sn = submessage_elements::SequenceNumber::deserialize(&bytes[12..20], endianness)?;
        let last_sn = submessage_elements::SequenceNumber::deserialize(&bytes[20..28], endianness)?;
        let count = submessage_elements::Count::deserialize(&bytes[28..32], endianness)?;
        

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
    use crate::types::constants::ENTITYID_UNKNOWN;
    use crate::types::EntityKind;

    #[test]
    fn test_heartbeat_validity_function() {
        let valid_heartbeat = Heartbeat {
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            first_sn: submessage_elements::SequenceNumber(2),
            last_sn: submessage_elements::SequenceNumber(5), 
            count: submessage_elements::Count(0),
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(valid_heartbeat.is_valid(), true);

        let valid_heartbeat_first_message = Heartbeat {
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            first_sn: submessage_elements::SequenceNumber(1),
            last_sn: submessage_elements::SequenceNumber(0), 
            count: submessage_elements::Count(2),
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(valid_heartbeat_first_message.is_valid(), true);

        let invalid_heartbeat_zero_first_value = Heartbeat {
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            first_sn: submessage_elements::SequenceNumber(0),
            last_sn: submessage_elements::SequenceNumber(1), 
            count: submessage_elements::Count(2),
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_zero_first_value.is_valid(), false);

        let invalid_heartbeat_negative_last_value = Heartbeat {
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            first_sn: submessage_elements::SequenceNumber(5),
            last_sn: submessage_elements::SequenceNumber(-6), 
            count: submessage_elements::Count(2),
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_negative_last_value.is_valid(), false);

        let invalid_heartbeat_wrong_first_last_value = Heartbeat {
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            first_sn: submessage_elements::SequenceNumber(6),
            last_sn: submessage_elements::SequenceNumber(4), 
            count: submessage_elements::Count(2),
            final_flag: true,
            liveliness_flag: true,
            endianness_flag: Endianness::LittleEndian.into(),
        };

        assert_eq!(invalid_heartbeat_wrong_first_last_value.is_valid(), false);
    }

    #[test]
    fn test_serialize_deserialize_heartbeat() {
        let mut writer = Vec::new();

        let reader_id = types::EntityId::new([0x10, 0x12, 0x14], EntityKind::UserDefinedReaderWithKey);
        let writer_id = types::EntityId::new([0x26, 0x24, 0x22], EntityKind::UserDefinedWriterWithKey);
        let first_sn = 1233;
        let last_sn = 1237;
        let count = 8;
        let is_final = true;
        let manual_liveliness = false;

        let heartbeat_big_endian = Heartbeat::new(reader_id, writer_id, first_sn, last_sn, count, is_final, manual_liveliness, Endianness::BigEndian);
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

        let heartbeat_little_endian = Heartbeat::new(reader_id, writer_id, first_sn, last_sn, count, is_final, manual_liveliness, Endianness::LittleEndian);
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
}
