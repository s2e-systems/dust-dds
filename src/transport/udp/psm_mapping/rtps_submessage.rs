impl UdpPsmMapping for SubmessageHeader {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = Endianness::from(self.flags[0]);
        serialize_submessage_kind(self.submessage_id, writer)?;
        serialize_submessage_flags(&self.flags, writer)?;
        self.submessage_length.serialize(writer, endianness)?;
        Ok(())
    }

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let submessage_id = deserialize_submessage_kind(&bytes[0..1])?;
        let flags = deserialize_submessage_flags(&bytes[1..2])?;
        let endianness = Endianness::from(flags[0]);
        let submessage_length = submessage_elements::UShort::deserialize(&bytes[2..4], endianness)?;
        Ok(SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        })
    }
}

impl UdpPsmMapping for RtpsSubmessage {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        match self {
            RtpsSubmessage::AckNack(acknack) => acknack.compose(writer),
            RtpsSubmessage::Data(data) => data.compose(writer),
            RtpsSubmessage::Gap(gap) => gap.compose(writer),
            RtpsSubmessage::Heartbeat(heartbeat) => heartbeat.compose(writer),
            RtpsSubmessage::InfoTs(infots) => infots.compose(writer),
        }
    }

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let submessage_id =
            deserialize_submessage_kind(&[bytes[0]])?;
        match submessage_id {
            SubmessageKind::Data => Ok(RtpsSubmessage::Data(Data::parse(bytes)?)),
            SubmessageKind::Pad => todo!(),
            SubmessageKind::AckNack => Ok(RtpsSubmessage::AckNack(AckNack::parse(bytes)?)),
            SubmessageKind::Heartbeat => Ok(RtpsSubmessage::Heartbeat(Heartbeat::parse(bytes)?)),
            SubmessageKind::Gap => Ok(RtpsSubmessage::Gap(Gap::parse(bytes)?)),
            SubmessageKind::InfoTimestamp => Ok(RtpsSubmessage::InfoTs(InfoTs::parse(bytes)?)),
            SubmessageKind::InfoSource => todo!(),
            SubmessageKind::InfoReplyIP4 => todo!(),
            SubmessageKind::InfoDestination => todo!(),
            SubmessageKind::InfoReply => todo!(),
            SubmessageKind::NackFrag => todo!(),
            SubmessageKind::HeartbeatFrag => todo!(),
            SubmessageKind::DataFrag => todo!(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, ENTITYID_UNKNOWN};
    use super::data_submessage::Payload;

    #[test]
    fn test_parse_submessage_header() {
        let bytes = [0x15_u8, 0b00000001, 20, 0x0];
        let f = false;
        let flags: [SubmessageFlag; 8] = [true, f, f, f, f, f, f, f];
        let expected = SubmessageHeader {
            submessage_id: SubmessageKind::Data,
            flags,
            submessage_length: submessage_elements::UShort(20),
        };
        let result = SubmessageHeader::parse(&bytes);

        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_compose_submessage_header() {
        let mut result = Vec::new();

        let f = false;
        let t = true;
        let header = SubmessageHeader {
            submessage_id: SubmessageKind::Data,
            flags: [t, t, f, f, f, f, f, f],
            submessage_length: submessage_elements::UShort(16),
        };
        let expected = vec![0x15, 0b00000011, 16, 0x0];
        header.compose(&mut result).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_compose_submessage() {
        let submessage = RtpsSubmessage::Data(Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            1,
            None,
            Payload::None,
        ));

        let expected = vec![
            0x15_u8, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00, 16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];

        let mut writer = Vec::new();
        submessage.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);
    }

    #[test]
    fn test_parse_submessage() {
        let bytes = vec![
            0x15_u8, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00, 16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];
        let expected = RtpsSubmessage::Data(Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            1,
            None,
            Payload::None,
        ));
        let result = RtpsSubmessage::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }
}
