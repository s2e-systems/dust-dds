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

pub fn from_bytes_le<'de, D: Deserialize<'de>>(mut buf: &'de [u8]) -> Result<D> {
    D::deserialize::<LittleEndian>(&mut buf)
}
