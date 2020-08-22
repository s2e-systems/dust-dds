use super::types::constants::PROTOCOL_RTPS;
use super::serdes::{RtpsSerdesResult, };
use crate::types::constants::{PROTOCOL_VERSION_2_4, VENDOR_ID};
use crate::types::{GuidPrefix, ProtocolVersion, VendorId};
use std::convert::TryInto;
use super::submessages::RtpsSubmessage;
use super::UdpPsmMapping;
use super::types::ProtocolId;

#[derive(PartialEq, Debug)]
pub struct Header {
    protocol: ProtocolId,
    version: ProtocolVersion,
    vendor_id: VendorId,
    guid_prefix: GuidPrefix,
}

impl Header {
    pub fn new(guid_prefix: GuidPrefix) -> Self {
        Self {
            protocol: PROTOCOL_RTPS,
            version: PROTOCOL_VERSION_2_4,
            vendor_id: VENDOR_ID,
            guid_prefix,
        }
    }

    pub fn version(&self) -> ProtocolVersion {
        self.version
    }

    pub fn vendor_id(&self) -> VendorId {
        self.vendor_id
    }

    pub fn guid_prefix(&self) -> &GuidPrefix {
        &self.guid_prefix
    }

    
}

impl UdpPsmMapping for Header {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        writer.write(&self.protocol)?;
        writer.write(&[self.version.major])?;
        writer.write(&[self.version.minor])?;
        writer.write(&self.vendor_id)?;
        writer.write(&self.guid_prefix)?;
        Ok(())
    }

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let protocol = bytes[0..4].try_into()?;
        let version = ProtocolVersion {
            major: bytes[4],
            minor: bytes[5],
        };
        let vendor_id = bytes[6..8].try_into()?;
        let guid_prefix = bytes[8..20].try_into()?;
        Ok(Header {
            protocol,
            version,
            vendor_id,
            guid_prefix,
        })
    }

}

#[derive(PartialEq, Debug)]
pub struct RtpsMessage {
    header: Header,
    submessages: Vec<RtpsSubmessage>,
}

impl RtpsMessage {
    pub fn new(guid_prefix: GuidPrefix, submessages: Vec<RtpsSubmessage>) -> Self {
        if submessages.is_empty() {
            panic!("At least one submessage is required");
        };

        RtpsMessage {
            header: Header::new(guid_prefix),
            submessages,
        }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn submessages(&self) -> &Vec<RtpsSubmessage> {
        &self.submessages
    }

    pub fn take_submessages(self) -> Vec<RtpsSubmessage> {
        self.submessages
    }
}

impl UdpPsmMapping for RtpsMessage {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        &self.header.compose(writer)?;
        for submessage in &self.submessages {
            submessage.compose(writer)?;
        }
        Ok(())
    }

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let size = bytes.len();
        let header = Header::parse(bytes)?;

        let mut submessage_start_index: usize = header.octets();
        let mut submessages = Vec::<RtpsSubmessage>::new();

        while submessage_start_index < size {
            let submessage = RtpsSubmessage::parse(&bytes[submessage_start_index..])?;

            submessage_start_index += submessage.octets();
            submessages.push(submessage);
        }
        Ok(RtpsMessage {
            header,
            submessages,
        })
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, ENTITYID_UNKNOWN};
    use crate::types::{EntityKind};
    use crate::messages::types::Time;
    use crate::types;
    use crate::messages::submessages::data_submessage::Payload;
    use super::super::{Endianness,};
    use crate::messages::submessages::{ Data, InfoTs, Heartbeat};

    #[test]
    fn test_parse_message_header() {
        let expected = Header {
            protocol: [b'R', b'T', b'P', b'S'],
            version: ProtocolVersion { major: 2, minor: 1 },
            vendor_id: [1, 2],
            guid_prefix: [127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1],
        };

        let bytes = [
            0x52, 0x54, 0x50, 0x53, //000 protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
            0x02, 0x01, 0x01,
            0x02, //004 version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
            0x7f, 0x20, 0xf7, 0xd7, //008 guidPrefix: GuidPrefix_t => 127, 32, 247, 215
            0x00, 0x00, 0x01, 0xbb, //012 guidPrefix: GuidPrefix_t => 0, 0, 1, 187
            0x00, 0x00, 0x00, 0x01, //016 guidPrefix: GuidPrefix_t => 0, 0, 0, 1
        ];
        let result = Header::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    
    #[test]
    fn test_compose_message() {
        let submessage = RtpsSubmessage::Data(Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            1,
            None,
            Payload::None,
        ));
        let message = RtpsMessage {
            header: Header {
                protocol: [b'R', b'T', b'P', b'S'],
                version: ProtocolVersion { major: 2, minor: 1 },
                vendor_id: [1, 2],
                guid_prefix: [127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1],
            },
            submessages: vec![submessage],
        };

        let expected = vec![
            0x52, 0x54, 0x50, 0x53, //000 protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
            0x02, 0x01, 0x01,
            0x02, //004 version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
            0x7f, 0x20, 0xf7, 0xd7, //008 guidPrefix: GuidPrefix_t => 127, 32, 247, 215
            0x00, 0x00, 0x01, 0xbb, //012 guidPrefix: GuidPrefix_t => 0, 0, 1, 187
            0x00, 0x00, 0x00, 0x01, //016 guidPrefix: GuidPrefix_t => 0, 0, 0, 1
            0x15_u8, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00, 16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);
    }

    #[test]
    fn test_compose_message_three_data_submessages() {
        let test_time = Time::new(1565525425, 269558339);
        let submessage1 =
            RtpsSubmessage::InfoTs(InfoTs::new(Some(test_time), Endianness::LittleEndian));
        let submessage2 = RtpsSubmessage::Data(Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            1,
            None,
            Payload::None,
        ));
        let submessage3 = RtpsSubmessage::Data(Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            2,
            None,
            Payload::None,
        ));
        let message = RtpsMessage {
            header: Header {
                protocol: [b'R', b'T', b'P', b'S'],
                version: ProtocolVersion { major: 2, minor: 1 },
                vendor_id: [1, 2],
                guid_prefix: [127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1],
            },
            submessages: vec![submessage1, submessage2, submessage3],
        };

        let expected = vec![
            0x52, 0x54, 0x50, 0x53, //000 protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
            0x02, 0x01, 0x01,
            0x02, //004 version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
            0x7f, 0x20, 0xf7, 0xd7, //008 guidPrefix: GuidPrefix_t => 127, 32, 247, 215
            0x00, 0x00, 0x01, 0xbb, //012 guidPrefix: GuidPrefix_t => 0, 0, 1, 187
            0x00, 0x00, 0x00, 0x01, //016 guidPrefix: GuidPrefix_t => 0, 0, 0, 1
            0x09, 0x01, 0x08, 0x00, // [Info Timestamp] Submessgae Header
            0xB1, 0x05, 0x50, 0x5D, // [Info Timestamp]
            0x43, 0x22, 0x11, 0x10, // [Info Timestamp]
            0x15_u8, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00, 16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
            0x15_u8, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00, 16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x02, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 2
        ];
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);
    }


    #[test]
    fn test_parse_message() {
        let submessage = RtpsSubmessage::Data(Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            1,
            None,
            Payload::None,
        ));
        let expected = RtpsMessage {
            header: Header {
                protocol: [b'R', b'T', b'P', b'S'],
                version: ProtocolVersion { major: 2, minor: 1 },
                vendor_id: [1, 2],
                guid_prefix: [127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1],
            },
            submessages: vec![submessage],
        };

        let bytes = [
            0x52, 0x54, 0x50, 0x53, //000 protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
            0x02, 0x01, 0x01,
            0x02, //004 version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
            0x7f, 0x20, 0xf7, 0xd7, //008 guidPrefix: GuidPrefix_t => 127, 32, 247, 215
            0x00, 0x00, 0x01, 0xbb, //012 guidPrefix: GuidPrefix_t => 0, 0, 1, 187
            0x00, 0x00, 0x00, 0x01, //016 guidPrefix: GuidPrefix_t => 0, 0, 0, 1
            0x15, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00, 16,
            0x0, // [Data Submessage] ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];
        let result = RtpsMessage::parse(&bytes).unwrap();
        assert_eq!(expected, result);

        let submessage1 = RtpsSubmessage::Data(Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            1,
            None,
            Payload::None,
        ));
        let reader_id = types::EntityId::new(
            [0x10, 0x12, 0x14],
            EntityKind::UserDefinedReaderWithKey,
        );
        let writer_id = types::EntityId::new(
            [0x26, 0x24, 0x22],
            EntityKind::UserDefinedWriterWithKey,
        );
        let first_sn = 1233;
        let last_sn = 1237;
        let count = 8;
        let final_flag = true;
        let liveliness_flag = false;

        let submessage2 = RtpsSubmessage::Heartbeat(Heartbeat::new(
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
            final_flag,
            liveliness_flag,
            Endianness::BigEndian,
        ));
        let expected = RtpsMessage {
            header: Header {
                protocol: [b'R', b'T', b'P', b'S'],
                version: ProtocolVersion { major: 2, minor: 1 },
                vendor_id: [1, 2],
                guid_prefix: [127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1],
            },
            submessages: vec![submessage1, submessage2],
        };
        let bytes = [
            0x52, 0x54, 0x50, 0x53, // protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
            0x02, 0x01, 0x01,
            0x02, // version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
            0x7f, 0x20, 0xf7, 0xd7, // guidPrefix: GuidPrefix_t => 127, 32, 247, 215
            0x00, 0x00, 0x01, 0xbb, // guidPrefix: GuidPrefix_t => 0, 0, 1, 187
            0x00, 0x00, 0x00, 0x01, // guidPrefix: GuidPrefix_t => 0, 0, 0, 1
            0x15, 0b00000001, 20, 0x0, // Submessgae Header => Data
            0x00, 0x00, 16,
            0x0, // [Data Submessage] ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
            0x07, 0b00000010, 0x00, 28, // Submessage Header => Heartbeat
            0x10, 0x12, 0x14, 0x04, // [Heartbeat Submessage] Reader ID
            0x26, 0x24, 0x22, 0x02, // [Heartbeat Submessage] Writer ID
            0x00, 0x00, 0x00, 0x00, // [Heartbeat Submessage] First Sequence Number
            0x00, 0x00, 0x04, 0xD1, // [Heartbeat Submessage] First Sequence Number
            0x00, 0x00, 0x00, 0x00, // [Heartbeat Submessage] Last Sequence Number
            0x00, 0x00, 0x04, 0xD5, // [Heartbeat Submessage] Last Sequence Number
            0x00, 0x00, 0x00, 0x08, // [Heartbeat Submessage] Count
        ];
        let result = RtpsMessage::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_message_two_data_submessages() {
        let test_time = Time::new(1565525425, 269558339);
        let submessage1 =
            RtpsSubmessage::InfoTs(InfoTs::new(Some(test_time), Endianness::LittleEndian));
        let submessage2 = RtpsSubmessage::Data(Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            1,
            None,
            Payload::None,
        ));
        let submessage3 = RtpsSubmessage::Data(Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            2,
            None,
            Payload::None,
        ));
        let expected = RtpsMessage {
            header: Header {
                protocol: [b'R', b'T', b'P', b'S'],
                version: ProtocolVersion { major: 2, minor: 1 },
                vendor_id: [1, 2],
                guid_prefix: [127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1],
            },
            submessages: vec![submessage1, submessage2, submessage3],
        };

        let bytes = [
            0x52, 0x54, 0x50, 0x53, // protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
            0x02, 0x01, 0x01,
            0x02, // version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
            0x7f, 0x20, 0xf7, 0xd7, // guidPrefix: GuidPrefix_t => 127, 32, 247, 215
            0x00, 0x00, 0x01, 0xbb, // guidPrefix: GuidPrefix_t => 0, 0, 1, 187
            0x00, 0x00, 0x00, 0x01, // guidPrefix: GuidPrefix_t => 0, 0, 0, 1
            0x09, 0x01, 0x08, 0x00, // [Info Timestamp] Submessgae Header
            0xB1, 0x05, 0x50, 0x5D, // [Info Timestamp]
            0x43, 0x22, 0x11, 0x10, // [Info Timestamp]
            0x15, 0b00000001, 20, 0x0, // [Data Submessage] Submessgae Header
            0x00, 0x00, 16,
            0x0, // [Data Submessage] ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
            0x15, 0b00000001, 20, 0x0, // [Data Submessage] Submessgae Header
            0x00, 0x00, 16,
            0x0, // [Data Submessage] ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x02, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];
        let result = RtpsMessage::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }


}