use crate::types::Time;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsParse, EndianessFlag, RtpsSerdesResult, RtpsSerdesError, SizeCheckers, SizeSerializer};

use super::{SubmessageKind, OctetsToNextHeader};

#[derive(PartialEq, Debug)]
pub struct InfoTs {
    timestamp: Option<Time>,
}

impl InfoTs {
    const INVALID_TIME_FLAG_MASK: u8 = 0x02;

    pub fn new(timestamp: Option<Time>) -> InfoTs {
        InfoTs { timestamp }
    }

    pub fn get_timestamp(&self) -> &Option<Time> {
        &self.timestamp
    }
}


impl RtpsSerialize for InfoTs
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        SubmessageKind::InfoTimestamp.serialize(writer, endianness)?;

        let mut size_serializer = SizeSerializer::new();

        let mut flags = endianness as u8;
        if self.timestamp.is_none() {
            flags |= InfoTs::INVALID_TIME_FLAG_MASK;
        }
        writer.write(&[flags])?;

        self.timestamp.serialize(&mut size_serializer, endianness)?;
        OctetsToNextHeader(size_serializer.get_size() as u16).serialize(writer, endianness)?;
       
        self.timestamp.serialize(writer, endianness)?;

        Ok(())
    }
}

impl RtpsParse for InfoTs {
    type Output = InfoTs;

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self::Output> {
        const SERIALIZED_INFOTS_MINIMUM_SIZE: usize = 4;
        const SERIALIZED_INFOTS_WITH_TIMESTAMP_MINIMUM_SIZE: usize = 12;
        const SUBMESSAGE_ID_INDEX: usize = 0;
        const FLAGS_INDEX: usize = 1;
        SizeCheckers::check_size_bigger_equal_than(bytes, SERIALIZED_INFOTS_MINIMUM_SIZE)?;

        let submessage_kind = SubmessageKind::parse(&[bytes[0]])?;
        if submessage_kind != SubmessageKind::InfoTimestamp {
            return Err(RtpsSerdesError::InvalidSubmessageHeader);
        }

        let flags = bytes[FLAGS_INDEX];
        let invalid_time_flag = flags & InfoTs::INVALID_TIME_FLAG_MASK == InfoTs::INVALID_TIME_FLAG_MASK;

        if invalid_time_flag {
            Ok(InfoTs::new(None))
        } else {
            SizeCheckers::check_size_bigger_equal_than(bytes, SERIALIZED_INFOTS_WITH_TIMESTAMP_MINIMUM_SIZE)?;
            
            let time = Time::deserialize(&bytes[4..12], flags.into())?;

            Ok(InfoTs::new(Some(time)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_infots() {
        let mut writer_le = Vec::new();
        let mut writer_be = Vec::new();
        let info_timestamp_message_little_endian =
            [0x09, 0x01, 0x08, 0x00, 0xB1, 0x05, 0x50, 0x5D, 0x43, 0x22, 0x11, 0x10];
        let info_timestamp_message_big_endian = 
            [0x09, 0x00, 0x00, 0x08, 0x5D, 0x50, 0x05, 0xB1, 0x10, 0x11, 0x22, 0x43];

        const TEST_TIME: Time = Time {
            seconds: 1565525425,
            fraction: 269558339,
        };

        let infots = InfoTs::new(Some(TEST_TIME));
        infots.serialize(&mut writer_le, EndianessFlag::LittleEndian).unwrap();
        infots.serialize(&mut writer_be, EndianessFlag::BigEndian).unwrap();

        assert_eq!(writer_le, info_timestamp_message_little_endian);
        assert_eq!(writer_be, info_timestamp_message_big_endian);
        assert_eq!(InfoTs::parse(&writer_le).unwrap(), infots);
        assert_eq!(InfoTs::parse(&writer_be).unwrap(), infots);
    }
}
