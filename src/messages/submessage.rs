use std::convert::TryInto;
use super::types::{SubmessageKind, SubmessageFlag};
use super::serdes::{RtpsSerdesResult, RtpsSerdesError, SizeCheck};
use super::{UdpPsmMapping, Endianness};
use super::{AckNack, Data, Gap, Heartbeat, InfoTs};

fn serialize_submessage_kind(kind: SubmessageKind, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()>{
    let submessage_kind_u8 = kind as u8;
    writer.write(&[submessage_kind_u8])?;
    Ok(())
}

fn deserialize_submessage_kind(bytes: &[u8]) -> RtpsSerdesResult<SubmessageKind> { 
    bytes.check_size_equal(1)?;
    Ok(num::FromPrimitive::from_u8(bytes[0]).ok_or(RtpsSerdesError::InvalidEnumRepresentation)?)
}

fn serialize_submessage_flags(submessage_flags: &[SubmessageFlag; 8], writer: &mut impl std::io::Write) -> RtpsSerdesResult<()>{
    let mut flags = 0u8;
    for i in 0..8 {
        if submessage_flags[i] {
            flags |= 0b00000001 << i;
        }
    }
    writer.write(&[flags])?;
    Ok(())
}

fn deserialize_submessage_flags(bytes: &[u8]) -> RtpsSerdesResult<[SubmessageFlag; 8]> {
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


#[derive(PartialEq, Debug)]
pub struct SubmessageHeader {
    pub submessage_id: SubmessageKind,
    pub flags: [SubmessageFlag; 8],
    pub submessage_length: u16,
}

impl SubmessageHeader {
    pub fn submessage_id(&self) -> SubmessageKind {
        self.submessage_id
    }
    pub fn flags(&self) -> &[SubmessageFlag; 8] {
        &self.flags
    }
    pub fn submessage_length(&self) -> u16 {
        self.submessage_length
    }
}

impl UdpPsmMapping for SubmessageHeader {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = Endianness::from(self.flags[0]);
        serialize_submessage_kind(self.submessage_id, writer)?;
        serialize_submessage_flags(&self.flags, writer)?;
        match endianness {
            Endianness::LittleEndian => writer.write(&self.submessage_length.to_le_bytes())?,
            Endianness::BigEndian => writer.write(&self.submessage_length.to_be_bytes())?,
        };
        Ok(())
    }

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let submessage_id = deserialize_submessage_kind(&bytes[0..1])?;
        let flags = deserialize_submessage_flags(&bytes[1..2])?;
        let endianness = Endianness::from(flags[0]);
        let submessage_length = match endianness {
            Endianness::LittleEndian => u16::from_le_bytes(bytes[2..4].try_into()?),
            Endianness::BigEndian => u16::from_be_bytes(bytes[2..4].try_into()?),
        };
        Ok(SubmessageHeader {
            submessage_id,
            flags,
            submessage_length,
        })
    }
}



#[derive(Debug, PartialEq)]
pub enum RtpsSubmessage {
    AckNack(AckNack),
    Data(Data),
    // DataFrag(DataFrag),
    Gap(Gap),
    Heartbeat(Heartbeat),
    // HeartbeatFrag(HeartbeatFrag),
    // InfoDst(InfoDst),
    // InfoReply(InfoReply),
    // InfoSrc(InfoSrc),
    InfoTs(InfoTs),
    // NackFrag(NackFrag),
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


pub trait Submessage 
{
    fn submessage_header(&self) -> SubmessageHeader;

    fn is_valid(&self) -> bool;
}




#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, ENTITYID_UNKNOWN};
    use super::super::data_submessage::Payload;

    #[test]
    fn test_parse_submessage_header() {
        let bytes = [0x15_u8, 0b00000001, 20, 0x0];
        let f = false;
        let flags: [SubmessageFlag; 8] = [true, f, f, f, f, f, f, f];
        let expected = SubmessageHeader {
            submessage_id: SubmessageKind::Data,
            flags,
            submessage_length: 20,
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
            submessage_length: 16,
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
