use std::io::{BufRead, Read};

use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use rust_rtps_pim::messages::overall_structure::RtpsSubmessageHeader;

pub type Result<T> = std::result::Result<T, std::io::Error>;

pub trait MappingRead<'de>: Sized {
    fn read(buf: &mut &'de [u8]) -> Result<Self>;
}

pub trait DeserializeSubmessage<'de>: Sized {
    fn deserialize(buf: &mut &'de [u8]) -> Result<Self> {
        let header: RtpsSubmessageHeader = MappingRead::read(buf)?;
        if header.flags[0] {
            Self::deserialize_submessage::<LittleEndian>(buf, header)
        } else {
            Self::deserialize_submessage::<BigEndian>(buf, header)
        }
    }
    fn deserialize_submessage<B: ByteOrder>(
        buf: &mut &'de [u8],
        header: RtpsSubmessageHeader,
    ) -> Result<Self>;
}

impl<'a, 'de: 'a, T> MappingRead<'de> for T
where
    T: DeserializeSubmessage<'de>,
{
    fn read(buf: &mut &'de [u8]) -> Result<Self> {
        DeserializeSubmessage::deserialize(buf)
    }
}

pub trait MappingReadByteOrdered<'de>: Sized {
    fn read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder;
}

impl<'de> MappingReadByteOrdered<'de> for u8 {
    fn read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_u8()
    }
}
impl<'de> MappingRead<'de> for u8 {
    fn read(buf: &mut &'de [u8]) -> Result<Self> {
        buf.read_u8()
    }
}

impl<'de> MappingReadByteOrdered<'de> for i8 {
    fn read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_i8()
    }
}

impl<'de> MappingReadByteOrdered<'de> for u16 {
    fn read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_u16::<B>()
    }
}

impl<'de> MappingReadByteOrdered<'de> for i16 {
    fn read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_i16::<B>()
    }
}

impl<'de> MappingReadByteOrdered<'de> for u32 {
    fn read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_u32::<B>()
    }
}

impl<'de> MappingReadByteOrdered<'de> for i32 {
    fn read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_i32::<B>()
    }
}

impl<'de, const N: usize> MappingReadByteOrdered<'de> for [u8; N] {
    fn read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        let mut value = [0; N];
        buf.read_exact(value.as_mut())?;
        Ok(value)
    }
}
impl<'de, const N: usize> MappingRead<'de> for [u8; N] {
    fn read(buf: &mut &'de [u8]) -> Result<Self> {
        let mut value = [0; N];
        buf.read_exact(value.as_mut())?;
        Ok(value)
    }
}

impl<'de> MappingReadByteOrdered<'de> for bool {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self> {
        let value: u8 = MappingReadByteOrdered::read_byte_ordered::<B>(buf)?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "bool not valid",
            )),
        }
    }
}

impl<'de> MappingReadByteOrdered<'de> for &'de str {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self> {
        let length: u32 = MappingReadByteOrdered::read_byte_ordered::<B>(buf)?;
        let length = length as usize;
        let result = std::str::from_utf8(&buf[..length - 1])
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        buf.consume(length);
        Ok(result)
    }
}

pub fn from_bytes_le<'de, D: MappingReadByteOrdered<'de>>(mut buf: &'de [u8]) -> Result<D> {
    D::read_byte_ordered::<LittleEndian>(&mut buf)
}

pub fn from_bytes<'de, D: MappingRead<'de>>(mut buf: &'de [u8]) -> Result<D> {
    D::read(&mut buf)
}
