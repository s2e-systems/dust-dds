use std::io::Read;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};

pub type Result<T> = std::result::Result<T, std::io::Error>;

pub trait Deserialize<'de>: Sized {
    fn deserialize<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder;
}

impl<'de> Deserialize<'de> for u8 {
    fn deserialize<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_u8()
    }
}

impl<'de> Deserialize<'de> for i8 {
    fn deserialize<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_i8()
    }
}

impl<'de> Deserialize<'de> for u16 {
    fn deserialize<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_u16::<B>()
    }
}

impl<'de> Deserialize<'de> for i16 {
    fn deserialize<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_i16::<B>()
    }
}

impl<'de> Deserialize<'de> for u32 {
    fn deserialize<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_u32::<B>()
    }
}

impl<'de> Deserialize<'de> for i32 {
    fn deserialize<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        buf.read_i32::<B>()
    }
}

impl<'de, const N: usize> Deserialize<'de> for [u8; N] {
    fn deserialize<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder,
    {
        let mut value = [0; N];
        buf.read_exact(value.as_mut())?;
        Ok(value)
    }
}


impl<'de> Deserialize<'de> for bool {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self> {
        let value: u8 = Deserialize::deserialize::<B>(buf)?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "bool not valid"))
        }
    }
}

impl<'de> Deserialize<'de> for String {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self> {
        let length: u32 = Deserialize::deserialize::<B>(buf)?;
        let mut string_buf = vec![0_u8; length as usize];
        buf.read_exact(&mut string_buf[..])?;
        string_buf.pop();
        String::from_utf8(string_buf).map_err(|_err| std::io::ErrorKind::Other.into())
    }
}

pub fn from_bytes_le<'de, D: Deserialize<'de>>(mut buf: &'de [u8]) -> Result<D> {
    D::deserialize::<LittleEndian>(&mut buf)
}
