use num_derive::FromPrimitive;
use std::io;

#[derive(Debug)]
pub enum RtpsSerdesError {
    WrongSize,
    MessageTooSmall,
    InvalidEnumRepresentation,
    InvalidHeader,
    InvalidSubmessageHeader,
    InvalidSubmessage,
    InvalidKeyAndDataFlagCombination,
    IoError(std::io::Error),
    InvalidTypeConversion,
    DeserializationMessageSizeTooSmall,
}

impl From<std::io::Error> for RtpsSerdesError {
    fn from(error: std::io::Error) -> Self {
        RtpsSerdesError::IoError(error)
    }
}

impl From<std::array::TryFromSliceError> for RtpsSerdesError {
    fn from(_error: std::array::TryFromSliceError) -> Self {
        RtpsSerdesError::WrongSize
    }
}

pub type RtpsSerdesResult<T> = std::result::Result<T, RtpsSerdesError>;

#[derive(FromPrimitive, PartialEq, Debug, Clone, Copy)]
pub enum EndianessFlag {
    BigEndian = 0,
    LittleEndian = 1,
}

impl From<u8> for EndianessFlag {
    fn from(value: u8) -> Self {
        const ENDIANNESS_FLAG_MASK: u8 = 1;

        let flag_u8 = value & ENDIANNESS_FLAG_MASK;
        if flag_u8 == 0 {
            EndianessFlag::BigEndian
        } else {
            EndianessFlag::LittleEndian
        }
    }
}

pub struct SizeSerializer {
    size: usize,
}

impl SizeSerializer {
    pub fn new() -> Self {
        SizeSerializer {
            size: 0,
        }
    }

    pub fn get_size(&self) -> usize {
        self.size
    }
}

impl std::io::Write for SizeSerializer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>{
        let size = buf.len();
        self.size += size;
        Ok(size)
    }

    fn flush(&mut self) -> io::Result<()>{
        Ok(())
    }
}

pub trait RtpsSerialize where 
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()>;
}

pub trait RtpsParse {
    type Output;

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self::Output>;
}

pub trait RtpsDeserialize {
    type Output;

    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self::Output>;
}

impl<T> RtpsSerialize for Option<T> 
where 
    T: RtpsSerialize
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianess: EndianessFlag) -> RtpsSerdesResult<()> {
        if let Some(value) = self {
            value.serialize(writer, endianess)
        } else {
            Ok(())
        }
    }
}

pub struct PrimitiveSerdes{}
impl PrimitiveSerdes{

    pub fn serialize_u32(value: u32, endianness: EndianessFlag) -> [u8;4] {
        match endianness {
            EndianessFlag::BigEndian => value.to_be_bytes(),
            EndianessFlag::LittleEndian => value.to_le_bytes(),
        }
    }

    pub fn deserialize_u32(bytes: [u8; 4], endianness: EndianessFlag) -> u32 {
        match endianness {
            EndianessFlag::BigEndian => u32::from_be_bytes(bytes),
            EndianessFlag::LittleEndian => u32::from_le_bytes(bytes),
        }
    }

    pub fn serialize_i32(value: i32, endianness: EndianessFlag) -> [u8;4] {
        match endianness {
            EndianessFlag::BigEndian => value.to_be_bytes(),
            EndianessFlag::LittleEndian => value.to_le_bytes(),
        }
    }

    pub fn deserialize_i32(bytes: [u8; 4], endianness: EndianessFlag) -> i32 {
        match endianness {
            EndianessFlag::BigEndian => i32::from_be_bytes(bytes),
            EndianessFlag::LittleEndian => i32::from_le_bytes(bytes),
        }
    }

    pub fn serialize_u16(value: u16, endianness: EndianessFlag) -> [u8;2] {
        match endianness {
            EndianessFlag::BigEndian => value.to_be_bytes(),
            EndianessFlag::LittleEndian => value.to_le_bytes(),
        }
    }

    pub fn deserialize_u16(bytes: [u8; 2], endianness: EndianessFlag) -> u16 {
        match endianness {
            EndianessFlag::BigEndian => u16::from_be_bytes(bytes),
            EndianessFlag::LittleEndian => u16::from_le_bytes(bytes),
        }
    }

    pub fn serialize_i16(value: i16, endianness: EndianessFlag) -> [u8;2] {
        match endianness {
            EndianessFlag::BigEndian => value.to_be_bytes(),
            EndianessFlag::LittleEndian => value.to_le_bytes(),
        }
    }

    pub fn deserialize_i16(bytes: [u8; 2], endianness: EndianessFlag) -> i16 {
        match endianness {
            EndianessFlag::BigEndian => i16::from_be_bytes(bytes),
            EndianessFlag::LittleEndian => i16::from_le_bytes(bytes),
        }
    }
}

pub struct SizeCheckers{}
impl SizeCheckers {
    pub fn check_size_equal(bytes: &[u8], expected_size: usize) -> RtpsSerdesResult<()> {
        if bytes.len() != expected_size {
            Err(RtpsSerdesError::WrongSize)
        } else {
            Ok(())
        }
    }
    
    pub fn check_size_bigger_equal_than(bytes: &[u8], expected_size: usize) -> RtpsSerdesResult<()> {
        if bytes.len() >= expected_size {
            Ok(())
        } else {
            Err(RtpsSerdesError::MessageTooSmall)
        }
    }
}
