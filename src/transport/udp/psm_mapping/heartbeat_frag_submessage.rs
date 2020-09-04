impl UdpPsmMapping for HeartbeatFrag {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = self.endianness_flag.into();
        self.submessage_header().compose(writer)?;
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.writer_sn.serialize(writer, endianness)?;
        self.last_fragment_num.serialize(writer, endianness)?;
        self.count.serialize(writer, endianness)?;
        Ok(())
    }

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let header = SubmessageHeader::parse(bytes)?;
        let endianness_flag = header.flags()[0];
        let endianness = Endianness::from(endianness_flag);

        let reader_id = submessage_elements::EntityId::deserialize(&bytes[4..8], endianness)?;
        let writer_id = submessage_elements::EntityId::deserialize(&bytes[8..12], endianness)?;
        let writer_sn = submessage_elements::SequenceNumber::deserialize(&bytes[12..20], endianness)?;
        let last_fragment_num = submessage_elements::FragmentNumber::deserialize(&bytes[20..24], endianness)?;
        let count = submessage_elements::Count::deserialize(&bytes[24..28], endianness)?;        

        Ok(HeartbeatFrag {
            endianness_flag,
            reader_id,
            writer_id,
            writer_sn,
            last_fragment_num,
            count,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, };

    #[test]
    fn parse_heartbeat_frag_submessage() {
        let expected = HeartbeatFrag {
            endianness_flag: true,    
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER),
            writer_sn: submessage_elements::SequenceNumber(1),
            last_fragment_num: submessage_elements::FragmentNumber(2),
            count: submessage_elements::Count(3),
        };
        let bytes = vec![
            0x13, 0b00000001, 24, 0x0, // Submessgae Header
            0x00, 0x00, 0x00, 0x00, // readerId 
            0x00, 0x01, 0x00, 0xc2, // writerId
            0x00, 0x00, 0x00, 0x00, // writerSN
            0x01, 0x00, 0x00, 0x00, // writerSN 
            0x02, 0x00, 0x00, 0x00, // lastFragmentNum
            0x03, 0x00, 0x00, 0x00, // count
        ];
        let result = HeartbeatFrag::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    
    #[test]
    fn compose_heartbeat_frag_submessage() {
        let message = HeartbeatFrag {
            endianness_flag: true,    
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER),
            writer_sn: submessage_elements::SequenceNumber(1),
            last_fragment_num: submessage_elements::FragmentNumber(2),
            count: submessage_elements::Count(3),
        };
        let expected = vec![
            0x13, 0b00000001, 24, 0x0, // Submessgae Header
            0x00, 0x00, 0x00, 0x00, // readerId 
            0x00, 0x01, 0x00, 0xc2, // writerId
            0x00, 0x00, 0x00, 0x00, // writerSN
            0x01, 0x00, 0x00, 0x00, // writerSN 
            0x02, 0x00, 0x00, 0x00, // lastFragmentNum
            0x03, 0x00, 0x00, 0x00, // count
        ];
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);
    }

}
