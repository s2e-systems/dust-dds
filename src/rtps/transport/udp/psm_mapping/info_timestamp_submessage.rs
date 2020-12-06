use crate::rtps::messages::submessages::{InfoTs, SubmessageHeader};
use crate::rtps::messages::types::constants::TIME_INVALID;

use super::{UdpPsmMappingResult};
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
        Ok(InfoTs::from_raw_parts(endianness_flag, invalidate_flag, TIME_INVALID))
    } else {            
        let timestamp = deserialize_timestamp(&bytes[0..8], endianness)?;
        Ok(InfoTs::from_raw_parts(endianness_flag, invalidate_flag, timestamp))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps::messages::types::Endianness;
    use crate::rtps::messages::submessages::Submessage;

    #[test]
    fn test_serialize_deserialize_infots() {
        // let mut writer_le = Vec::new();
        let mut writer = Vec::new();
        let info_timestamp_message_little_endian =
            [0xB1, 0x05, 0x50, 0x5D, 0x43, 0x22, 0x11, 0x10]; // 0x09, 0x01, 0x08, 0x00,  
        let info_timestamp_message_big_endian = 
            [0x5D, 0x50, 0x05, 0xB1, 0x10, 0x11, 0x22, 0x43]; //0x09, 0x00, 0x00, 0x08, 

        let test_time = crate::rtps::messages::types::Time::new(1565525425, 269558339);

        let infots_big_endian = InfoTs::new(Endianness::BigEndian, Some(test_time));
        serialize_info_timestamp(&infots_big_endian, &mut writer).unwrap();
        assert_eq!(writer, info_timestamp_message_big_endian);
        assert_eq!(deserialize_info_timestamp(&writer, infots_big_endian.submessage_header(info_timestamp_message_big_endian.len() as u16)).unwrap(), infots_big_endian);

        writer.clear();

        let infots_little_endian = InfoTs::new(Endianness::LittleEndian, Some(test_time));
        serialize_info_timestamp(&infots_little_endian, &mut writer).unwrap();
        assert_eq!(writer, info_timestamp_message_little_endian);
        assert_eq!(deserialize_info_timestamp(&writer, infots_little_endian.submessage_header(info_timestamp_message_little_endian.len() as u16)).unwrap(), infots_little_endian);
    }
}
