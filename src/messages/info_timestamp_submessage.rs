use crate::types_primitives::Ushort;
use crate::messages::types::Time;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsCompose, RtpsParse, EndianessFlag, RtpsSerdesResult, };
use super::{SubmessageKind, SubmessageFlag, Submessage, SubmessageHeader, };

#[derive(PartialEq, Debug)]
pub struct InfoTs {
    endianness_flag: SubmessageFlag,
    invalidate_flag: SubmessageFlag,
    timestamp: Option<Time>,
}

impl InfoTs {
    const INVALID_TIME_FLAG_MASK: u8 = 0x02;

    pub fn new(timestamp: Option<Time>, endianness: EndianessFlag) -> InfoTs {
        let endianness_flag = endianness.into();
        let invalidate_flag =
        if timestamp.is_some() {
            SubmessageFlag(false)
        } else {
            SubmessageFlag(true)
        };

        InfoTs {
            endianness_flag,
            invalidate_flag,
            timestamp
        }
    }

    pub fn get_timestamp(&self) -> &Option<Time> {
        &self.timestamp
    }
}

impl Submessage for InfoTs {
    fn submessage_header(&self) -> SubmessageHeader {
        let x = SubmessageFlag(false);
        let e = self.endianness_flag; // Indicates endianness.
        let i = self.invalidate_flag; // Indicates whether subsequent Submessages should be considered as having a timestamp or not.
        // X|X|X|X|X|X|I|E
        let flags = [e, i, x, x, x, x, x, x];

        let octets_to_next_header = if self.invalidate_flag.is_set() {
            0
        } else {
            self.timestamp.octets()
        };
            
        SubmessageHeader { 
            submessage_id: SubmessageKind::InfoTimestamp,
            flags,
            submessage_length: Ushort(octets_to_next_header as u16), // This cast could fail in weird ways by truncation
        }
    }
}


impl RtpsCompose for InfoTs
{
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = EndianessFlag::from(self.endianness_flag);
        self.submessage_header().compose(writer)?;
        self.timestamp.serialize(writer, endianness)?;

        Ok(())
    }
    
}

impl RtpsParse for InfoTs {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let header = SubmessageHeader::parse(bytes)?;
        let flags = header.flags();
        // X|X|X|X|X|X|I|E
        /*E*/ let endianness_flag = flags[0];
        /*I*/ let invalidate_flag = flags[1];

        let endianness = endianness_flag.into();

//         const SERIALIZED_INFOTS_MINIMUM_SIZE: usize = 4;
//         const SERIALIZED_INFOTS_WITH_TIMESTAMP_MINIMUM_SIZE: usize = 12;
//         const SUBMESSAGE_ID_INDEX: usize = 0;
//         const FLAGS_INDEX: usize = 1;
//         SizeCheckers::check_size_bigger_equal_than(bytes, SERIALIZED_INFOTS_MINIMUM_SIZE)?;


        if invalidate_flag.is_set() {
            Ok(InfoTs::new(None, endianness))
        } else {
            // SizeCheckers::check_size_bigger_equal_than(bytes, SERIALIZED_INFOTS_WITH_TIMESTAMP_MINIMUM_SIZE)?;
            
            let time = Time::deserialize(&bytes[4..12], endianness)?;

            Ok(InfoTs::new(Some(time), endianness))
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

        let test_time = Time::new(1565525425, 269558339);

        let infots_big_endian = InfoTs::new(Some(test_time), EndianessFlag::BigEndian);
        // infots.compose(&mut writer_le, EndianessFlag::LittleEndian).unwrap();
        infots_big_endian.compose(&mut writer).unwrap();
        assert_eq!(writer, info_timestamp_message_big_endian);
        assert_eq!(InfoTs::parse(&writer).unwrap(), infots_big_endian);

        writer.clear();

        let infots_little_endian = InfoTs::new(Some(test_time), EndianessFlag::LittleEndian);
        infots_little_endian.compose(&mut writer).unwrap();
        assert_eq!(writer, info_timestamp_message_little_endian);
        assert_eq!(InfoTs::parse(&writer).unwrap(), infots_little_endian);
    }
}
