impl UdpPsmMapping for InfoTs {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = Endianness::from(self.endianness_flag);
        self.submessage_header().compose(writer)?;
        match &self.timestamp {
            Some(timestamp) => timestamp.serialize(writer, endianness)?,
            None => (),
        };

        Ok(())
    }

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let header = SubmessageHeader::parse(bytes)?;
        let flags = header.flags();
        // X|X|X|X|X|X|I|E
        /*E*/ let endianness_flag = flags[0];
        /*I*/ let invalidate_flag = flags[1];

        let endianness = endianness_flag.into();
        if invalidate_flag {
            Ok(InfoTs{ invalidate_flag, endianness_flag, timestamp: None})
        } else {            
            let timestamp = Some(submessage_elements::Timestamp::deserialize(&bytes[4..12], endianness)?);
            Ok(InfoTs{invalidate_flag, endianness_flag, timestamp})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_infots() {
        // let mut writer_le = Vec::new();
        let mut writer = Vec::new();
        let info_timestamp_message_little_endian =
            [0x09, 0x01, 0x08, 0x00, 0xB1, 0x05, 0x50, 0x5D, 0x43, 0x22, 0x11, 0x10];
        let info_timestamp_message_big_endian = 
            [0x09, 0x00, 0x00, 0x08, 0x5D, 0x50, 0x05, 0xB1, 0x10, 0x11, 0x22, 0x43];

        let test_time = crate::messages::types::Time::new(1565525425, 269558339);

        let infots_big_endian = InfoTs::new(Some(test_time), Endianness::BigEndian);
        // infots.compose(&mut writer_le, Endianness::LittleEndian).unwrap();
        infots_big_endian.compose(&mut writer).unwrap();
        assert_eq!(writer, info_timestamp_message_big_endian);
        assert_eq!(InfoTs::parse(&writer).unwrap(), infots_big_endian);

        writer.clear();

        let infots_little_endian = InfoTs::new(Some(test_time), Endianness::LittleEndian);
        infots_little_endian.compose(&mut writer).unwrap();
        assert_eq!(writer, info_timestamp_message_little_endian);
        assert_eq!(InfoTs::parse(&writer).unwrap(), infots_little_endian);
    }
}
