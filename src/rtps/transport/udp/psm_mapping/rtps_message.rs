use std::convert::TryInto;

use super::{UdpPsmMappingResult, UdpPsmMappingError};
use super::rtps_submessage::{serialize_rtps_submessage, deserialize_rtps_submessage, deserialize_submessage_header};

use crate::rtps::types::ProtocolVersion;
use crate::rtps::messages::RtpsSubmessage;
use crate::rtps::messages::message::{Header, RtpsMessage};

const RTPS_MESSAGE_HEADER_OCTETS : usize = 20;

fn serialize_header(header: &Header, writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()> {
        writer.write(&header.protocol())?;
        writer.write(&[header.version().major])?;
        writer.write(&[header.version().minor])?;
        writer.write(&header.vendor_id())?;
        writer.write(&header.guid_prefix())?;
        Ok(())
}

fn deserialize_header(bytes: &[u8]) -> UdpPsmMappingResult<Header> {
    let protocol : [u8;4] = bytes[0..4].try_into()?;

    if protocol != [b'R', b'T', b'P', b'S'] {
        return Err(UdpPsmMappingError::InvalidProtocolId);
    }

    let version = ProtocolVersion {
        major: bytes[4],
        minor: bytes[5],
    };
    let vendor_id = bytes[6..8].try_into()?;
    let guid_prefix = bytes[8..20].try_into()?;
    Ok(Header::new(version, vendor_id, guid_prefix))
}

pub fn serialize_rtps_message(rtps_message: &RtpsMessage, writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()> {
    serialize_header(&rtps_message.header(), writer)?;
    for submessage in rtps_message.submessages() {
        serialize_rtps_submessage(submessage, writer)?;
    }
    Ok(())
}

pub fn deserialize_rtps_message(bytes: &[u8]) -> UdpPsmMappingResult<RtpsMessage> {
    let size = bytes.len();
    let header = deserialize_header(bytes)?;

    let mut submessage_start_index: usize = RTPS_MESSAGE_HEADER_OCTETS;
    let mut submessages = Vec::<RtpsSubmessage>::new();

    while submessage_start_index < size {
        let submessage_header = deserialize_submessage_header(&bytes[submessage_start_index..submessage_start_index+4])?;
        
        let submessage_end_index = submessage_start_index+4+submessage_header.submessage_length() as usize;
        let submessage = deserialize_rtps_submessage(&bytes[submessage_start_index..submessage_end_index])?;
        submessage_start_index = submessage_end_index;

        submessages.push(submessage);
    }
    Ok(RtpsMessage::new(header.version(), header.vendor_id(), header.guid_prefix(), submessages))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps::types::constants::{ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, ENTITYID_UNKNOWN};
    use crate::rtps::types::{EntityKind, };
    use crate::rtps::messages::types::Time;
    use crate::rtps::types;
    use crate::rtps::messages::submessages::data_submessage::Payload;
    use crate::rtps::messages::submessages::{ Data, InfoTs, Heartbeat};
    use crate::rtps::messages::types::Endianness;

    #[test]
    fn test_parse_message_header() {
        let expected = Header::new(ProtocolVersion{ major: 2, minor: 1 }, [1, 2],  [127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1]);

        let bytes = [
            0x52, 0x54, 0x50, 0x53, //000 protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
            0x02, 0x01, 0x01,
            0x02, //004 version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
            0x7f, 0x20, 0xf7, 0xd7, //008 guidPrefix: GuidPrefix_t => 127, 32, 247, 215
            0x00, 0x00, 0x01, 0xbb, //012 guidPrefix: GuidPrefix_t => 0, 0, 1, 187
            0x00, 0x00, 0x00, 0x01, //016 guidPrefix: GuidPrefix_t => 0, 0, 0, 1
        ];
        let result = deserialize_header(&bytes).unwrap();
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
        let message = RtpsMessage::new(
            ProtocolVersion { major: 2, minor: 1 },  
            [1, 2], 
            [127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1], 
            vec![submessage]);

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
        serialize_rtps_message(&message, &mut writer).unwrap();
        assert_eq!(expected, writer);
    }

    #[test]
    fn test_compose_message_three_data_submessages() {
        let test_time = Time::new(1565525425, 269558339);
        let submessage1 =
            RtpsSubmessage::InfoTs(InfoTs::new(Endianness::LittleEndian,Some(test_time)));
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
        let message = RtpsMessage::new(
                ProtocolVersion { major: 2, minor: 1 },
                [1, 2],
                [127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1],
                vec![submessage1, submessage2, submessage3]);

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
        serialize_rtps_message(&message, &mut writer).unwrap();
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
        let expected = RtpsMessage::new(
        ProtocolVersion { major: 2, minor: 1 },
                [1, 2],
                [127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1],
            vec![submessage]);

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
        let result = deserialize_rtps_message(&bytes).unwrap();
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
            Endianness::BigEndian,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
            final_flag,
            liveliness_flag,
        ));
        let expected = RtpsMessage::new(
            ProtocolVersion { major: 2, minor: 1 },
                [1, 2],
                [127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1],
            vec![submessage1, submessage2]);

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
        let result = deserialize_rtps_message(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_message_two_data_submessages() {
        let test_time = Time::new(1565525425, 269558339);
        let submessage1 =
            RtpsSubmessage::InfoTs(InfoTs::new(Endianness::LittleEndian,Some(test_time)));
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
        let expected = RtpsMessage::new(
            ProtocolVersion { major: 2, minor: 1 },
                [1, 2],
                [127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1],
           vec![submessage1, submessage2, submessage3]);

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
            0x0, // [Data Submessage] ExtraFlags, octetsToInlineQos (little indian)
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
        let result = deserialize_rtps_message(&bytes).unwrap();
        assert_eq!(expected, result);
    }
}