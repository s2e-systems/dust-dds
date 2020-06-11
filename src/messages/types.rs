/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.13 - Types used to define RTPS messages
///  
 
use crate::serdes::{SizeCheckers, RtpsSerialize, RtpsDeserialize, RtpsSerdesResult, RtpsSerdesError, EndianessFlag, };
use num_derive::{FromPrimitive, ToPrimitive, };
use num_traits::{FromPrimitive, };
use std::time::SystemTime;
use std::convert::TryInto;
use crate::primitive_types::{UShort, Long, ULong, };

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

    
    pub const PROTOCOL_RTPS: ProtocolId = ProtocolId([b'R', b'T', b'P', b'S']);
}


pub trait Pid {
    fn pid() -> ParameterIdT;
}


// /////////// ProtocolId_t //////////

#[derive(PartialEq, Debug)]
pub struct ProtocolId(pub [u8; 4]);

impl RtpsSerialize for ProtocolId {
    fn serialize(&self, writer: &mut impl std::io::Write, _endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        writer.write(&self.0)?;
        Ok(())
    }    
}

impl RtpsDeserialize for ProtocolId {
    fn deserialize(bytes: &[u8], _endianness: EndianessFlag) -> RtpsSerdesResult<Self> {
        if bytes == self::constants::PROTOCOL_RTPS.0 {
            Ok(ProtocolId(bytes[0..4].try_into()?))
        } else {
            Err(RtpsSerdesError::InvalidEnumRepresentation)
        }
    }    
}



// /////////// SubmessageFlag ////////

pub type SubmessageFlag = bool;

impl RtpsSerialize for [SubmessageFlag; 8] {
    fn serialize(&self, writer: &mut impl std::io::Write, _endianness: EndianessFlag) -> RtpsSerdesResult<()>{
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
    fn deserialize(bytes: &[u8], _endianness: EndianessFlag) -> RtpsSerdesResult<Self> { 
        // SizeCheckers::check_size_equal(bytes, 1)?;
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
    fn serialize(&self, writer: &mut impl std::io::Write, _endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        let submessage_kind_u8 = *self as u8;
        writer.write(&[submessage_kind_u8])?;
        Ok(())
    }
}

impl RtpsDeserialize for SubmessageKind {
    fn deserialize(bytes: &[u8], _endianness: EndianessFlag) -> RtpsSerdesResult<Self> { 
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
    pub fn new (seconds: u32, fraction: u32) -> Self {
        Time {
            seconds,
            fraction,
        }
    }

    pub fn now() -> Self {
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        Time{seconds: current_time.as_secs() as u32 , fraction: current_time.subsec_nanos() as u32}
    }
}
 
impl RtpsSerialize for Time {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        self.seconds.serialize(writer, endianness)?;
        self.fraction.serialize(writer, endianness)?;
        Ok(())
    }
}

impl RtpsDeserialize for Time {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> {
        SizeCheckers::check_size_equal(bytes, 8)?;

        let seconds = ULong::deserialize(&bytes[0..4], endianness)?;
        let fraction = ULong::deserialize(&bytes[4..8], endianness)?;

        Ok(Time::new(seconds, fraction))
    }
}



// /////////// Count_t ///////////////

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
pub struct Count(pub i32);

impl std::ops::AddAssign<i32> for Count {
    fn add_assign(&mut self, rhs: i32) {
        *self = Count(self.0+rhs)
    }
}

impl RtpsSerialize for Count {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        (self.0 as Long).serialize(writer, endianness)?;
        Ok(())
    }
}

impl RtpsDeserialize for Count {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> {
        let value = Long::deserialize(bytes, endianness)?;
        Ok(Count(value))
    }
}



// /////////// ParameterId_t /////////

#[derive(Debug, PartialEq, FromPrimitive, ToPrimitive, Eq, Clone, Copy)]
#[repr(u16)]
pub enum ParameterIdT {
    TopicName = 0x0005,
    Durability = 0x001d,
    Presentation = 0x0021,
    Deadline = 0x0023,
    LatencyBudget = 0x0027,
    Ownership = 0x001f,
    OwnershipStrength = 0x0006,
    Liveliness = 0x001b,
    Partition = 0x0029,
    Reliability = 0x001a,
    TransportPriority = 0x0049,
    Lifespan = 0x002b,
    DestinationOrder = 0x0025,
    ContentFilterInfo = 0x0055,
    CoherentSet = 0x0056,
    DirectedWrite = 0x0057,
    OriginalWriterInfo = 0x0061,
    GroupCoherentSet = 0x0063,
    GroupSeqNum = 0x0064,
    WriterGroupInfo = 0x0065,
    SecureWriterGroupInfo = 0x0066,
    KeyHash = 0x0070,
    StatusInfo = 0x0071,
    Sentinel = 0x0001,
    VendorTest0 = 0x0000 | 0x8000,
    VendorTest1 = 0x0001 | 0x8000,
    VendorTest3 = 0x0003 | 0x8000,
    VendorTest4 = 0x0004 | 0x8000,
    VendorTest5 = 0x0005 | 0x8000,
    VendorTestShort = 0x0006 | 0x8000,
}

impl RtpsSerialize for ParameterIdT {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        (*self as UShort).serialize(writer, endianness)?;
        Ok(())
    }
}

impl RtpsDeserialize for ParameterIdT {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> {
        let value = UShort::deserialize(bytes, endianness)?;
        Ok(ParameterIdT::from_u16(value).unwrap())
    }
}



// /////////// FragmentNumber_t //////
// Same as in self::submessage_elements

// /////////// GroupDigest_t /////////
//  todo





#[cfg(test)]
mod tests {
    use super::*;

    // /////////////////////// ProtocolId_t Tests ////////////////////////
        
    #[test]
    fn test_serialize_protocol_id() {
        let mut writer = Vec::new();
        self::constants::PROTOCOL_RTPS.serialize(&mut writer, EndianessFlag::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(writer, vec![0x52, 0x54, 0x50, 0x53]);
    }

    #[test]
    fn test_deserialize_protocol_id() {
        let expected = ProtocolId([b'R', b'T', b'P', b'S']);
        let bytes = [0x52_u8, 0x54, 0x50, 0x53];    
        let result = ProtocolId::deserialize(&bytes, EndianessFlag::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_deserialize_invalid_protocol_id() {
        let bytes = [0x52_u8, 0x54, 0x50, 0x99];    
        assert!(ProtocolId::deserialize(&bytes, EndianessFlag::LittleEndian /*irrelevant*/).is_err());

        let bytes = [0x52_u8];    
        assert!(ProtocolId::deserialize(&bytes, EndianessFlag::LittleEndian /*irrelevant*/).is_err());
    }


    // /////////////////////// SubmessageFlag Tests ////////////////////////
    
    #[test]
    fn test_deserialize_submessage_flags() {
        let f = false;
        let t = true;

        let expected: [SubmessageFlag; 8] = [t, f, f, f, f, f, f, f];
        let bytes = [0b00000001_u8];    
        let result = <[SubmessageFlag; 8]>::deserialize(&bytes, EndianessFlag::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(expected, result);

        let expected: [SubmessageFlag; 8] = [t, t, f, t, f, f, f, f];
        let bytes = [0b00001011_u8];    
        let result = <[SubmessageFlag; 8]>::deserialize(&bytes, EndianessFlag::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(expected, result);

        let expected: [SubmessageFlag; 8] = [t, t, t, t, t, t, t, t];
        let bytes = [0b11111111_u8];    
        let result = <[SubmessageFlag; 8]>::deserialize(&bytes, EndianessFlag::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(expected, result);

        let expected: [SubmessageFlag; 8] = [f, f, f, f, f, f, f, f];
        let bytes = [0b00000000_u8];    
        let result = <[SubmessageFlag; 8]>::deserialize(&bytes, EndianessFlag::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(expected, result);
    }
   
    #[test]
    fn test_serialize_submessage_flags() {
        let f = false;
        let t = true;
        let mut writer = Vec::new();

        writer.clear();
        let flags: [SubmessageFlag; 8] = [t, f, f, f, f, f, f, f];
        flags.serialize(&mut writer, EndianessFlag::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(writer, vec![0b00000001]);
        
        writer.clear();
        let flags: [SubmessageFlag; 8] = [f; 8];
        flags.serialize(&mut writer, EndianessFlag::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(writer, vec![0b00000000]);
        
        writer.clear();
        let flags: [SubmessageFlag; 8] = [t; 8];
        flags.serialize(&mut writer, EndianessFlag::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(writer, vec![0b11111111]);
        
        writer.clear();
        let flags: [SubmessageFlag; 8] = [f, t, f, f, t, t, f, t];
        flags.serialize(&mut writer, EndianessFlag::LittleEndian /*irrelevant*/).unwrap();
        assert_eq!(writer, vec![0b10110010]);
    }



    // /////////////////////// SubmessageKind Tests ////////////////////////



    // /////////////////////// Time_t Tests ////////////////////////
     
    #[test]
    fn test_time_serialization_deserialization_big_endian() {
        let mut vec = Vec::new();
        let test_time = Time::new(1234567, 98765432);

        
        const TEST_TIME_BIG_ENDIAN : [u8;8] = [0x00, 0x12, 0xD6, 0x87, 0x05, 0xE3, 0x0A, 0x78];
        test_time.serialize(&mut vec, EndianessFlag::BigEndian).unwrap();
        assert_eq!(vec, TEST_TIME_BIG_ENDIAN);
        assert_eq!(Time::deserialize(&vec, EndianessFlag::BigEndian).unwrap(), test_time);
    }

    #[test]
    fn test_time_serialization_deserialization_little_endian() {
        let mut vec = Vec::new();
        let test_time = Time::new(1234567, 98765432);
        
        const TEST_TIME_LITTLE_ENDIAN : [u8;8] = [0x87, 0xD6, 0x12, 0x00, 0x78, 0x0A, 0xE3, 0x05];
        test_time.serialize(&mut vec, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(vec, TEST_TIME_LITTLE_ENDIAN);
        assert_eq!(Time::deserialize(&vec, EndianessFlag::LittleEndian).unwrap(), test_time);
    }

    #[test]
    fn test_invalid_time_deserialization() {
        let wrong_vec = vec![1,2,3,4];

        let expected_error = Time::deserialize(&wrong_vec, EndianessFlag::LittleEndian);
        match expected_error {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        };
    }



    // /////////////////////// Count_t Tests ////////////////////////
     

     
    // /////////////////////// ParameterId_t Tests //////////////////
    #[test]
    fn serialize_parameter_id_t() {
        let mut writer = Vec::new();

        writer.clear();
        let parameter = ParameterIdT::KeyHash;
        parameter.serialize(&mut writer, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(writer, vec![0x70, 0x00]);

        writer.clear();
        parameter.serialize(&mut writer, EndianessFlag::BigEndian).unwrap();
        assert_eq!(writer, vec![0x00, 0x70]);
        
        writer.clear();
        let parameter = ParameterIdT::VendorTest1;
        parameter.serialize(&mut writer, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(writer, vec![0x01, 0x80]);

        writer.clear();
        let parameter = ParameterIdT::VendorTest1;
        parameter.serialize(&mut writer, EndianessFlag::BigEndian).unwrap();
        assert_eq!(writer, vec![0x80, 0x01]);
    }

    #[test]
    fn deserialize_parameter_id_t() {
        let bytes = [0x70, 0x00];    
        let result = ParameterIdT::deserialize(&bytes, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(ParameterIdT::KeyHash, result);
        
        let bytes = [0x00, 0x70];    
        let result = ParameterIdT::deserialize(&bytes, EndianessFlag::BigEndian).unwrap();
        assert_eq!(ParameterIdT::KeyHash, result);

        let bytes = [0x01, 0x80];    
        let result = ParameterIdT::deserialize(&bytes, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(ParameterIdT::VendorTest1, result);

        let bytes = [0x80, 0x01];    
        let result = ParameterIdT::deserialize(&bytes, EndianessFlag::BigEndian).unwrap();
        assert_eq!(ParameterIdT::VendorTest1, result);
    }
    

    ////////////////////////// FragmentNumber_t Tests ///////////////////////



    ////////////////////////// GroupDigest_t Tests ///////////////////////
}