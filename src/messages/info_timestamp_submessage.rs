use crate::types::Time;
use crate::serdes::{RtpsSerialize, EndianessFlag, RtpsSerdesResult};
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

    pub fn deserialize(submessage: &[u8]) -> RtpsMessageResult<InfoTs> {
        const MESSAGE_PAYLOAD_FIRST_INDEX: usize = 4;
        const MESSAGE_PAYLOAD_LAST_INDEX: usize = 11;

        if MESSAGE_PAYLOAD_LAST_INDEX >= submessage.len() {
            return Err(RtpsMessageError::InvalidSubmessage);
        }

        let submessage_flags = submessage[1];

        let submessage_endianess = endianess(&submessage_flags)?;

        let timestamp = if submessage_flags & 0x02 == 0x02 {
            None
        } else {
            Some(deserialize::<Time>(
                submessage,
                &MESSAGE_PAYLOAD_FIRST_INDEX,
                &MESSAGE_PAYLOAD_LAST_INDEX,
                &submessage_endianess,
            )?)
        };

        Ok(InfoTs { timestamp })


    }
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

pub fn parse_info_timestamp_submessage(submessage: &[u8], submessage_flags: &u8) -> RtpsMessageResult<InfoTs> {
    const MESSAGE_PAYLOAD_FIRST_INDEX: usize = 0;
    const MESSAGE_PAYLOAD_LAST_INDEX: usize = 7;

    if MESSAGE_PAYLOAD_LAST_INDEX >= submessage.len() {
        return Err(RtpsMessageError::InvalidSubmessage);
    }

    let submessage_endianess = endianess(submessage_flags)?;

    let timestamp = if *submessage_flags & 0x02 == 0x02 {
        None
    } else {
        Some(deserialize::<Time>(
            submessage,
            &MESSAGE_PAYLOAD_FIRST_INDEX,
            &MESSAGE_PAYLOAD_LAST_INDEX,
            &submessage_endianess,
        )?)
    };

    Ok(InfoTs { timestamp })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_info_timestamp_submessage() {
        const BIG_ENDIAN_FLAG: u8 = 0x00;
        const LITTLE_ENDIAN_FLAG: u8 = 0x01;
        const INVALID_FLAG: u8 = 0x02;

        // Unix time: 1565525425=>0x5D5005B1
        // Is equivalent to: 08/11/2019 @ 12:10pm (UTC)
        // Seconds fraction: 0x10112243 => 269558339 => 0.0628
        const TEST_TIME: Time = Time {
            seconds: 1565525425,
            fraction: 269558339,
        };

        let timestamp_message_payload_big_endian = [0x5D, 0x50, 0x05, 0xB1, 0x10, 0x11, 0x22, 0x43];
        let info_ts_big_endian = parse_info_timestamp_submessage(
            &timestamp_message_payload_big_endian,
            &BIG_ENDIAN_FLAG,
        )
        .unwrap();
        assert_eq!(Some(TEST_TIME), info_ts_big_endian.timestamp);

        let timestamp_message_payload_little_endian =
            [0xB1, 0x05, 0x50, 0x5D, 0x43, 0x22, 0x11, 0x10];
        let info_ts_little_endian = parse_info_timestamp_submessage(
            &timestamp_message_payload_little_endian,
            &LITTLE_ENDIAN_FLAG,
        )
        .unwrap();
        assert_eq!(Some(TEST_TIME), info_ts_little_endian.timestamp);

        let info_ts_none_big_endian = parse_info_timestamp_submessage(
            &timestamp_message_payload_big_endian,
            &(BIG_ENDIAN_FLAG + INVALID_FLAG),
        )
        .unwrap();
        assert_eq!(None, info_ts_none_big_endian.timestamp);

        let info_ts_none_little_endian = parse_info_timestamp_submessage(
            &timestamp_message_payload_little_endian,
            &(LITTLE_ENDIAN_FLAG + INVALID_FLAG),
        )
        .unwrap();
        assert_eq!(None, info_ts_none_little_endian.timestamp);
    }

    #[test]
    fn test_serialize_infots() {
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
    }
}
