use crate::messages::submessages::{InfoTs, SubmessageHeader};
use crate::messages::types::constants::TIME_INVALID;
use crate::messages::submessages::submessage_elements::Timestamp;

use super::{UdpPsmMappingResult, TransportEndianness};
use super::submessage_elements::{serialize_timestamp, deserialize_timestamp};

pub fn serialize_info_timestamp(info_timestamp: &InfoTs, writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()> {
    let endianness = info_timestamp.endianness_flag().into();
    match info_timestamp.invalidate_flag() {
        true => (),
        false => serialize_timestamp(info_timestamp.timestamp(), writer, endianness)?,
    }
    
    Ok(())
}

pub fn deserialize_info_timestamp(bytes: &[u8], header: SubmessageHeader) -> UdpPsmMappingResult<InfoTs> {
    let flags = header.flags();

    // X|X|X|X|X|X|I|E
    /*E*/ let endianness_flag = flags[0];
    /*I*/ let invalidate_flag = flags[1];

    let endianness = endianness_flag.into();
    if invalidate_flag {
        Ok(InfoTs::from_raw_parts(endianness_flag, invalidate_flag, Timestamp(TIME_INVALID)))
    } else {            
        let timestamp = deserialize_timestamp(&bytes[4..12], endianness)?;
        Ok(InfoTs::from_raw_parts(endianness_flag, invalidate_flag, timestamp))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::submessages::Submessage;

    #[test]
    fn test_serialize_deserialize_infots() {
        // let mut writer_le = Vec::new();
        let mut writer = Vec::new();
        let info_timestamp_message_little_endian =
            [0xB1, 0x05, 0x50, 0x5D, 0x43, 0x22, 0x11, 0x10]; // 0x09, 0x01, 0x08, 0x00,  
        let info_timestamp_message_big_endian = 
            [0x5D, 0x50, 0x05, 0xB1, 0x10, 0x11, 0x22, 0x43]; //0x09, 0x00, 0x00, 0x08, 

        let test_time = crate::messages::types::Time::new(1565525425, 269558339);
        let flags = [true, false, false, false, false, false, false, false];

        let mut infots = InfoTs::new(Some(test_time));
        infots.set_endianness_flag(TransportEndianness::BigEndian.into());
        serialize_info_timestamp(&infots, &mut writer).unwrap();
        assert_eq!(writer, info_timestamp_message_big_endian);
        assert_eq!(deserialize_info_timestamp(&writer, infots.submessage_header(info_timestamp_message_big_endian.len() as u16)).unwrap(), infots);

        writer.clear();

        infots.set_endianness_flag(TransportEndianness::LittleEndian.into());
        serialize_info_timestamp(&infots, &mut writer).unwrap();
        assert_eq!(writer, info_timestamp_message_little_endian);
        assert_eq!(deserialize_info_timestamp(&writer, infots.submessage_header(info_timestamp_message_little_endian.len() as u16)).unwrap(), infots);
    }
}
