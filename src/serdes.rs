use num_derive::FromPrimitive;
use std::io;

#[derive(Debug)]
pub enum RtpsSerdesError {
    WrongSize,
    MessageTooSmall,
    InvalidEnumRepresentation,
    IoError(std::io::Error),
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

pub trait SubmessageElement 
    where Self: std::marker::Sized 
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()>;
    fn octets(&self) -> usize {
        let mut size_serializer = SizeSerializer::new();
        self.serialize(&mut size_serializer, Endianness::LittleEndian /*No impact on size*/ ).unwrap(); // Should panic on failure
        size_serializer.get_size()
    }

    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self>;   
}


pub trait SizeCheck {
    fn check_size_equal(&self, expected_size: usize) -> RtpsSerdesResult<()>;
    fn check_size_bigger_equal_than(&self, expected_size: usize) -> RtpsSerdesResult<()>;
}

impl SizeCheck for &[u8] {
    #[inline]
    fn check_size_equal(&self, expected_size: usize) -> RtpsSerdesResult<()> {
        if self.len() != expected_size {
            Err(RtpsSerdesError::WrongSize)
        } else {
            Ok(())
        }
    }
    
    #[inline]
    fn check_size_bigger_equal_than(&self, expected_size: usize) -> RtpsSerdesResult<()> {
        if self.len() >= expected_size {
            Ok(())
        } else {
            Err(RtpsSerdesError::MessageTooSmall)
        }
    }
}

