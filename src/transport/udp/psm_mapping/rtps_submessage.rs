use crate::messages::submessages::{Submessage, SubmessageHeader, RtpsSubmessage};
use crate::messages::types::{SubmessageKind, SubmessageFlag};

use super::{UdpPsmMappingResult, UdpPsmMappingError, SizeCheck, SizeSerializer};
use super::submessage_elements::{serialize_ushort, deserialize_ushort};
use super::ack_nack_submessage::{serialize_ack_nack, deserialize_ack_nack};
use super::data_submessage::{serialize_data, deserialize_data};
use super::gap_submessage::{serialize_gap, deserialize_gap};
use super::heartbeat_submessage::{serialize_heartbeat, deserialize_heartbeat};
use super::info_timestamp_submessage::{serialize_info_timestamp, deserialize_info_timestamp};

/// Section 9.4.5.1.1 SubmessageId
fn serialize_submessage_id(submessage_kind: SubmessageKind, writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()> {
    let value = match submessage_kind {
        SubmessageKind::Pad => 0x01,
        SubmessageKind::AckNack => 0x06,
        SubmessageKind::Heartbeat => 0x07,
        SubmessageKind::Gap => 0x08,
        SubmessageKind::InfoTimestamp => 0x09,
        SubmessageKind::InfoSource => 0x0c,
        SubmessageKind::InfoReplyIP4 => 0x0d,
        SubmessageKind::InfoDestination => 0x0e,
        SubmessageKind::InfoReply => 0x0f,
        SubmessageKind::NackFrag => 0x12,
        SubmessageKind::HeartbeatFrag => 0x13,
        SubmessageKind::Data => 0x15,
        SubmessageKind::DataFrag => 0x16,
    };

    writer.write(&[value])?;

    Ok(())
}

fn deserialize_submessage_id(bytes: &[u8]) -> UdpPsmMappingResult<SubmessageKind> {
    bytes.check_size_equal(1)?;

    match bytes[0] {
        0x01 => Ok(SubmessageKind::Pad),
        0x06 => Ok(SubmessageKind::AckNack),
        0x07 => Ok(SubmessageKind::Heartbeat),
        0x08 => Ok(SubmessageKind::Gap),
        0x09 => Ok(SubmessageKind::InfoTimestamp),
        0x0c => Ok(SubmessageKind::InfoSource),
        0x0d => Ok(SubmessageKind::InfoReplyIP4),
        0x0e => Ok(SubmessageKind::InfoDestination),
        0x0f => Ok(SubmessageKind::InfoReply),
        0x12 => Ok(SubmessageKind::NackFrag),
        0x13 => Ok(SubmessageKind::HeartbeatFrag),
        0x15 => Ok(SubmessageKind::Data),
        0x16 => Ok(SubmessageKind::DataFrag),
        _ => Err(UdpPsmMappingError::UnknownSubmessageId),
    }
}

fn serialize_submessage_flags(submessage_flags: &[SubmessageFlag; 8], writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()> {
    let mut flags = 0u8;
    for i in 0..8 {
        if submessage_flags[i] {
            flags |= 0b00000001 << i;
        }
    }
    writer.write(&[flags])?;
    Ok(())
}

fn deserialize_submessage_flags(bytes: &[u8]) -> UdpPsmMappingResult<[SubmessageFlag;8]> {
    bytes.check_size_equal(1)?;
    let flags: u8 = bytes[0];        
    let mut mask = 0b00000001_u8;
    let mut submessage_flags = [false; 8];
    for i in 0..8 {
        if (flags & mask) > 0 {
            submessage_flags[i] = true;
        }
        mask <<= 1;
    };
    Ok(submessage_flags)
}

pub fn serialize_submessage_header(submessage_header: &SubmessageHeader, writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()> {
    serialize_submessage_id(submessage_header.submessage_id(), writer)?;
    serialize_submessage_flags(submessage_header.flags(), writer)?;
    let endianness = submessage_header.flags()[0].into();
    serialize_ushort(submessage_header.submessage_length(), writer, endianness)?;
    Ok(())
}

pub fn deserialize_submessage_header(bytes: &[u8]) -> UdpPsmMappingResult<SubmessageHeader> {
    bytes.check_size_equal(4)?;

    let submessage_id = deserialize_submessage_id(&[bytes[0]])?;
    let flags = deserialize_submessage_flags(&[bytes[1]])?;
    let endianness = flags[0].into();
    let submessage_length = deserialize_ushort(&bytes[2..4], endianness)?.0;
    Ok(SubmessageHeader::new(submessage_id, flags, submessage_length))
}

pub fn serialize_rtps_submessage(rtps_submessage: &RtpsSubmessage, writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()> {
    let mut size_serializer = SizeSerializer::new();

    match rtps_submessage {
        RtpsSubmessage::AckNack(acknack) => {
            serialize_ack_nack(acknack, &mut size_serializer)?;
            serialize_submessage_header(&acknack.submessage_header(size_serializer.get_size() as u16), writer)?;
            serialize_ack_nack(&acknack, writer)?;
        },
        RtpsSubmessage::Data(data) =>  {
            serialize_data(data, &mut size_serializer)?;
            serialize_submessage_header(&data.submessage_header(size_serializer.get_size() as u16), writer)?;
            serialize_data(data, writer)?;
        },
        RtpsSubmessage::Gap(gap) => {
            serialize_gap(gap, &mut size_serializer)?;
            serialize_submessage_header(&gap.submessage_header(size_serializer.get_size() as u16), writer)?;
            serialize_gap(gap, writer)?;
        },
        RtpsSubmessage::Heartbeat(heartbeat) => {
            serialize_heartbeat(heartbeat, &mut size_serializer)?;
            serialize_submessage_header(&heartbeat.submessage_header(size_serializer.get_size() as u16), writer)?;
            serialize_heartbeat(heartbeat, writer)?;
        },
        RtpsSubmessage::InfoTs(infots) => {
            serialize_info_timestamp(infots, &mut size_serializer)?;
            serialize_submessage_header(&infots.submessage_header(size_serializer.get_size() as u16), writer)?;
            serialize_info_timestamp(infots, writer)?;
        },
    };

    Ok(())
}

pub fn deserialize_rtps_submessage(bytes: &[u8]) -> UdpPsmMappingResult<RtpsSubmessage> {
    bytes.check_size_bigger_equal_than(4)?;

    let submessage_header = deserialize_submessage_header(&bytes[0..4])?;

    match submessage_header.submessage_id() {
        SubmessageKind::AckNack => Ok(RtpsSubmessage::AckNack(deserialize_ack_nack(&bytes[4..4 + submessage_header.submessage_length().0 as usize], submessage_header)?)),
        SubmessageKind::Data => Ok(RtpsSubmessage::Data(deserialize_data(&bytes[4..4 + submessage_header.submessage_length().0 as usize], submessage_header)?)),
        SubmessageKind::Gap =>  Ok(RtpsSubmessage::Gap(deserialize_gap(&bytes[4..4 + submessage_header.submessage_length().0 as usize], submessage_header)?)),
        SubmessageKind::Heartbeat => Ok(RtpsSubmessage::Heartbeat(deserialize_heartbeat(&bytes[4..4 + submessage_header.submessage_length().0 as usize], submessage_header)?)),
        SubmessageKind::InfoTimestamp => Ok(RtpsSubmessage::InfoTs(deserialize_info_timestamp(&bytes[4..4 + submessage_header.submessage_length().0 as usize], submessage_header)?)),
        _ => unimplemented!(),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, ENTITYID_UNKNOWN};
    use crate::messages::Endianness;
    use crate::messages::submessages::Data;
    use crate::messages::submessages::data_submessage::Payload;

    #[test]
    fn test_deserialize_submessage_header() {
        let bytes = [0x15_u8, 0b00000001, 20, 0x0];
        let f = false;
        let flags: [SubmessageFlag; 8] = [true, f, f, f, f, f, f, f];
        let expected = SubmessageHeader::new(SubmessageKind::Data, flags,20);
        let result = deserialize_submessage_header(&bytes).unwrap();

        assert_eq!(expected, result);
    }

    #[test]
    fn test_serialize_submessage_header() {
        let mut result = Vec::new();

        let f = false;
        let t = true;
        let header = SubmessageHeader::new(SubmessageKind::Data, [t, t, f, f, f, f, f, f], 16);
        let expected = vec![0x15, 0b00000011, 16, 0x0];
        serialize_submessage_header(&header, &mut result).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_serialize_rtps_submessage() {
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
        serialize_rtps_submessage(&submessage, &mut writer).unwrap();
        assert_eq!(expected, writer);
    }

    #[test]
    fn test_deserialize_rtps_submessage() {
        let bytes = vec![
            0x15_u8, 0b00000001, 20, 0x0, // Submessage Header
            0x00, 0x00, 16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];

        let data = Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
            1,
            None,
            Payload::None,
        );

        let expected = RtpsSubmessage::Data(data);
        
        let result = deserialize_rtps_submessage(&bytes).unwrap();
        assert_eq!(expected, result);
    }
}
