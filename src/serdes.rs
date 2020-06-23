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
pub enum Endianness {
    BigEndian = 0,
    LittleEndian = 1,
}

impl From<bool> for Endianness {
    fn from(value: bool) -> Self {
        match value {
            true => Endianness::LittleEndian,
            false => Endianness::BigEndian,
        }
    }
}

impl From<Endianness> for bool {
    fn from(value: Endianness) -> Self {
        match value {
            Endianness::LittleEndian => true,
            Endianness::BigEndian => false,
        }
    }
}

impl From<u8> for Endianness {
    fn from(value: u8) -> Self {
        const ENDIANNESS_FLAG_MASK: u8 = 1;

        let flag_u8 = value & ENDIANNESS_FLAG_MASK;
        if flag_u8 == 0 {
            Endianness::BigEndian
        } else {
            Endianness::LittleEndian
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

pub trait RtpsSerialize 
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()>;
    fn octets(&self) -> usize {
        let mut size_serializer = SizeSerializer::new();
        self.serialize(&mut size_serializer, Endianness::LittleEndian /*No impact on size*/ ).unwrap(); // Should panic on failure
        size_serializer.get_size()
    }
}

pub trait RtpsCompose 
{
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()>;
    fn octets(&self) -> usize {
        let mut size_serializer = SizeSerializer::new();
        self.compose(&mut size_serializer).unwrap(); // Should panic on failure
        size_serializer.get_size()
    }
}

pub trait RtpsParse
where
    Self: std::marker::Sized
{
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self>;
}

pub trait RtpsDeserialize
where
    Self: std::marker::Sized
{
    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self>;
}

impl<T> RtpsSerialize for Option<T> 
where 
    T: RtpsSerialize
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianess: Endianness) -> RtpsSerdesResult<()> {
        if let Some(value) = self {
            value.serialize(writer, endianess)
        } else {
            Ok(())
        }
    } 
}



pub struct SizeCheckers{}
impl SizeCheckers {
    #[inline]
    pub fn check_size_equal(bytes: &[u8], expected_size: usize) -> RtpsSerdesResult<()> {
        if bytes.len() != expected_size {
            Err(RtpsSerdesError::WrongSize)
        } else {
            Ok(())
        }
    }
    
    #[inline]
    pub fn check_size_bigger_equal_than(bytes: &[u8], expected_size: usize) -> RtpsSerdesResult<()> {
        if bytes.len() >= expected_size {
            Ok(())
        } else {
            Err(RtpsSerdesError::MessageTooSmall)
        }
    }
}

