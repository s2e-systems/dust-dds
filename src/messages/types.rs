/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.13 - Types used to define RTPS messages
///  
use std::time::SystemTime;
use num_derive::{FromPrimitive, };

use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsSerdesResult, RtpsSerdesError, Endianness, SizeCheck };
use crate::primitive_types::{Short, ULong, };

pub mod constants {
    use super::Time;
    use super::ProtocolId;

    const TIME_ZERO: Time = Time {
        seconds: 0,
        fraction: 0,
    };

    const TIME_INFINITE: Time = Time {
        seconds: std::u32::MAX,
        fraction: std::u32::MAX - 1,
    };

    const TIME_INVALID: Time = Time {
        seconds: std::u32::MAX,
        fraction: std::u32::MAX,
    };

    
    pub const PROTOCOL_RTPS: ProtocolId = [b'R', b'T', b'P', b'S'];
}


// /////////// ProtocolId_t //////////
pub type ProtocolId = [u8; 4];

// /////////// SubmessageFlag ////////
pub type SubmessageFlag = bool;

impl RtpsSerialize for [SubmessageFlag; 8] {
    fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()>{
        let mut flags = 0u8;
        for i in 0..8 {
            if self[i] {
                flags |= 0b00000001 << i;
            }
        }
        writer.write(&[flags])?;
        Ok(())
    }
}

impl RtpsDeserialize for [SubmessageFlag; 8] {
    fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
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
}



// /////////// SubmessageKind ////////

#[derive(FromPrimitive, PartialEq, Copy, Clone, Debug)]
pub enum SubmessageKind {
    Pad = 0x01,
    AckNack = 0x06,
    Heartbeat = 0x07,
    Gap = 0x08,
    InfoTimestamp = 0x09,
    InfoSource = 0x0c,
    InfoReplyIP4 = 0x0d,
    InfoDestination = 0x0e,
    InfoReply = 0x0f,
    NackFrag = 0x12,
    HeartbeatFrag = 0x13,
    Data = 0x15,
    DataFrag = 0x16,
}

impl RtpsSerialize for SubmessageKind {
    fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()>{
        let submessage_kind_u8 = *self as u8;
        writer.write(&[submessage_kind_u8])?;
        Ok(())
    }
}

impl RtpsDeserialize for SubmessageKind {
    fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> { 
        bytes.check_size_equal(1)?;
        Ok(num::FromPrimitive::from_u8(bytes[0]).ok_or(RtpsSerdesError::InvalidEnumRepresentation)?)
    }
}



// /////////// Time_t ////////////////

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct Time {
    seconds: u32,
    fraction: u32,
}

impl Time {
    pub fn new (seconds: u32, fraction: u32) -> Self { Self {seconds, fraction, } }
    pub fn seconds(&self) -> u32 {self.seconds}
    pub fn fraction(&self) -> u32 {self.fraction}
    pub fn now() -> Self {
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        Time{seconds: current_time.as_secs() as u32 , fraction: current_time.subsec_nanos() as u32}
    }
}

pub type Count = i32;
pub type ParameterId = Short;
pub type FragmentNumber = ULong;

// /////////// GroupDigest_t /////////
//  todo


#[cfg(test)]
mod tests {
    use super::*;

    // /////////////////////// ProtocolId_t Tests ////////////////////////
        
    // #[test]
    // fn test_serialize_protocol_id() {
    //     let mut writer = Vec::new();
    //     self::constants::PROTOCOL_RTPS.serialize(&mut writer, Endianness::LittleEndian /*irrelevant*/).unwrap();
    //     assert_eq!(writer, vec![0x52, 0x54, 0x50, 0x53]);
    // }

    // #[test]
    // fn test_deserialize_protocol_id() {
    //     let expected = ProtocolId([b'R', b'T', b'P', b'S']);
    //     let bytes = [0x52_u8, 0x54, 0x50, 0x53];    
    //     let result = ProtocolId::deserialize(&bytes, Endianness::LittleEndian /*irrelevant*/).unwrap();
    //     assert_eq!(expected, result);
    // }

    // #[test]
    // fn test_deserialize_invalid_protocol_id() {
    //     let bytes = [0x52_u8, 0x54, 0x50, 0x99];    
    //     assert!(ProtocolId::deserialize(&bytes, Endianness::LittleEndian /*irrelevant*/).is_err());

    //     let bytes = [0x52_u8];    
    //     assert!(ProtocolId::deserialize(&bytes, Endianness::LittleEndian /*irrelevant*/).is_err());
    // }


    // /////////////////////// SubmessageFlag Tests ////////////////////////
    
    #[test]
    fn test_deserialize_submessage_flags() {
        let f = false;
        let t = true;

        let expected: [SubmessageFlag; 8] = [t, f, f, f, f, f, f, f];
        let bytes = [0b00000001_u8];    
        let result = <[SubmessageFlag; 8]>::deserialize(&bytes, Endianness::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(expected, result);

        let expected: [SubmessageFlag; 8] = [t, t, f, t, f, f, f, f];
        let bytes = [0b00001011_u8];    
        let result = <[SubmessageFlag; 8]>::deserialize(&bytes, Endianness::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(expected, result);

        let expected: [SubmessageFlag; 8] = [t, t, t, t, t, t, t, t];
        let bytes = [0b11111111_u8];    
        let result = <[SubmessageFlag; 8]>::deserialize(&bytes, Endianness::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(expected, result);

        let expected: [SubmessageFlag; 8] = [f, f, f, f, f, f, f, f];
        let bytes = [0b00000000_u8];    
        let result = <[SubmessageFlag; 8]>::deserialize(&bytes, Endianness::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(expected, result);
    }
   
    #[test]
    fn test_serialize_submessage_flags() {
        let f = false;
        let t = true;
        let mut writer = Vec::new();

        writer.clear();
        let flags: [SubmessageFlag; 8] = [t, f, f, f, f, f, f, f];
        flags.serialize(&mut writer, Endianness::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(writer, vec![0b00000001]);
        
        writer.clear();
        let flags: [SubmessageFlag; 8] = [f; 8];
        flags.serialize(&mut writer, Endianness::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(writer, vec![0b00000000]);
        
        writer.clear();
        let flags: [SubmessageFlag; 8] = [t; 8];
        flags.serialize(&mut writer, Endianness::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(writer, vec![0b11111111]);
        
        writer.clear();
        let flags: [SubmessageFlag; 8] = [f, t, f, f, t, t, f, t];
        flags.serialize(&mut writer, Endianness::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(writer, vec![0b10110010]);
    }



    // /////////////////////// SubmessageKind Tests ////////////////////////



    // /////////////////////// Time_t Tests ////////////////////////
     
    // #[test]
    // fn test_time_serialization_deserialization_big_endian() {
    //     let mut vec = Vec::new();
    //     let test_time = Time::new(1234567, 98765432);

        
    //     const TEST_TIME_BIG_ENDIAN : [u8;8] = [0x00, 0x12, 0xD6, 0x87, 0x05, 0xE3, 0x0A, 0x78];
    //     test_time.serialize(&mut vec, Endianness::BigEndian).unwrap();
    //     assert_eq!(vec, TEST_TIME_BIG_ENDIAN);
    //     assert_eq!(Time::deserialize(&vec, Endianness::BigEndian).unwrap(), test_time);
    // }

    // #[test]
    // fn test_time_serialization_deserialization_little_endian() {
    //     let mut vec = Vec::new();
    //     let test_time = Time::new(1234567, 98765432);
        
    //     const TEST_TIME_LITTLE_ENDIAN : [u8;8] = [0x87, 0xD6, 0x12, 0x00, 0x78, 0x0A, 0xE3, 0x05];
    //     test_time.serialize(&mut vec, Endianness::LittleEndian).unwrap();
    //     assert_eq!(vec, TEST_TIME_LITTLE_ENDIAN);
    //     assert_eq!(Time::deserialize(&vec, Endianness::LittleEndian).unwrap(), test_time);
    // }

    // #[test]
    // fn test_invalid_time_deserialization() {
    //     let wrong_vec = vec![1,2,3,4];

    //     let expected_error = Time::deserialize(&wrong_vec, Endianness::LittleEndian);
    //     match expected_error {
    //         Err(RtpsSerdesError::WrongSize) => assert!(true),
    //         _ => assert!(false),
    //     };
    // }



    // /////////////////////// Count_t Tests ////////////////////////
    

    ////////////////////////// FragmentNumber_t Tests ///////////////////////



    ////////////////////////// GroupDigest_t Tests ///////////////////////
}