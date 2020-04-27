use crate::types::Time;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsDeserializeWithEndianess, EndianessFlag, RtpsSerdesResult, RtpsSerdesError, SizeCheckers};
use super::helpers::{deserialize, endianess};

use super::{SubmessageKind, RtpsMessageError, RtpsMessageResult, OctetsToNextHeader};

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

    // pub fn serialize(&self, endi: EndianessFlag) -> Vec<u8> {
    //     let mut serialized_infots = Vec::new();
    //     serialized_infots.push(SubmessageKind::InfoTimestamp as u8);

    //     let mut flags = endi as u8;

    //     let octets_to_next_header = if self.timestamp.is_some() {
    //         OctetsToNextHeader(8)
    //     } else {
    //         flags |= InfoTs::INVALID_TIME_FLAG_MASK;
    //         OctetsToNextHeader(0)
    //     };

    //     serialized_infots.push(flags);
    //     serialized_infots.extend_from_slice(&octets_to_next_header.to_bytes(endi));

    //     if let Some(time) = &self.timestamp {
    //         serialized_infots.extend_from_slice(&time.to_bytes());
    //     }

    //     serialized_infots
    // }

    // pub fn deserialize(submessage: &[u8]) -> RtpsMessageResult<InfoTs> {
    //     const MESSAGE_PAYLOAD_FIRST_INDEX: usize = 4;
    //     const MESSAGE_PAYLOAD_LAST_INDEX: usize = 11;

    //     if MESSAGE_PAYLOAD_LAST_INDEX >= submessage.len() {
    //         return Err(RtpsMessageError::InvalidSubmessage);
    //     }

    //     let submessage_flags = submessage[1];

    //     let submessage_endianess = endianess(&submessage_flags)?;

    //     let timestamp = if submessage_flags & 0x02 == 0x02 {
    //         None
    //     } else {
    //         Some(deserialize::<Time>(
    //             submessage,
    //             &MESSAGE_PAYLOAD_FIRST_INDEX,
    //             &MESSAGE_PAYLOAD_LAST_INDEX,
    //             &submessage_endianess,
    //         )?)
    //     };

    //     Ok(InfoTs { timestamp })


    // }
}

impl<W> RtpsSerialize<W> for InfoTs
where 
    W: std::io::Write
{
    fn serialize(&self, writer: &mut W, endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        SubmessageKind::InfoTimestamp.serialize(writer, endianness)?;

        let mut flags = endianness as u8;
        if self.timestamp.is_none() {
            flags |= InfoTs::INVALID_TIME_FLAG_MASK;
        }
        writer.write(&[flags])?;

        if self.timestamp.is_some() {
            OctetsToNextHeader(8)
        } else {
            OctetsToNextHeader(0)
        }.serialize(writer, endianness)?;

        if let Some(time) = &self.timestamp {
            time.serialize(writer, endianness)?;
        }

        Ok(())
    }
}

impl RtpsDeserialize for InfoTs {
    type Output = InfoTs;

    fn deserialize(bytes: &[u8]) -> RtpsSerdesResult<Self::Output> {
        const SERIALIZED_INFOTS_MINIMUM_SIZE: usize = 4;
        const SUBMESSAGE_ID_INDEX: usize = 0;
        const FLAGS_INDEX: usize = 1;
        SizeCheckers::check_size_bigger_equal_than(bytes, SERIALIZED_INFOTS_MINIMUM_SIZE)?;

        let submessage_kind = SubmessageKind::deserialize(&[bytes[0]])?;
        if submessage_kind != SubmessageKind::InfoTimestamp {
            return Err(RtpsSerdesError::InvalidSubmessageHeader);
        }

        let flags = bytes[FLAGS_INDEX];
        let invalid_time_flag = flags & InfoTs::INVALID_TIME_FLAG_MASK == InfoTs::INVALID_TIME_FLAG_MASK;

        if invalid_time_flag {
            Ok(InfoTs::new(None))
        } else {
            SizeCheckers::check_size_bigger_equal_than(bytes, 12)?;
            
            let time = Time::deserialize_with_endianness(&bytes[4..12], flags.into())?;

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
        assert_eq!(InfoTs::deserialize(&writer_le).unwrap(), infots);
        assert_eq!(InfoTs::deserialize(&writer_be).unwrap(), infots);
    }
}
