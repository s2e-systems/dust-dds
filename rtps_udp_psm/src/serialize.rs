use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use std::io::Write;

pub type Result = std::result::Result<(), std::io::Error>;

pub trait Serialize {
    fn serialize<W: Write, B: ByteOrder>(&self, writer: W) -> Result;
}

impl Serialize for u8 {
    fn serialize<W: Write, B: ByteOrder>(&self, writer: W) -> Result {
        writer.write_u8(*self)
    }
}

impl Serialize for i8 {
    fn serialize<W: Write, B: ByteOrder>(&self, writer: W) -> Result {
        writer.write_i8(*self)
    }
}

impl Serialize for u16 {
    fn serialize<W: Write, B: ByteOrder>(&self, writer: W) -> Result {
        writer.write_u16::<B>(*self)
    }
}

impl Serialize for i16 {
    fn serialize<W: Write, B: ByteOrder>(&self, writer: W) -> Result {
        writer.write_i16::<B>(*self)
    }
}

impl Serialize for u32 {
    fn serialize<W: Write, B: ByteOrder>(&self, writer: W) -> Result {
        writer.write_u32::<B>(*self)
    }
}

impl Serialize for i32 {
    fn serialize<W: Write, B: ByteOrder>(&self, writer: W) -> Result {
        writer.write_i32::<B>(*self)
    }
}

impl<const N: usize> Serialize for [u8; N] {
    fn serialize<W: Write, B: ByteOrder>(&self, writer: W) -> Result {
        writer.write_all(self)?;
        Ok(())
    }
}

pub fn to_writer_le<S: Serialize, W: Write>(value: &S, mut writer: W) -> Result {
    value.serialize::<_, LittleEndian>(&mut writer)
}

pub fn to_bytes_le<S: Serialize>(value: &S) -> std::result::Result<Vec<u8>, std::io::Error> {
    let mut writer = Vec::<u8>::new();
    value.serialize::<_, LittleEndian>(&mut writer)?;
    Ok(writer)
}

pub trait NumberOfBytes {
    fn number_of_bytes(&self) -> usize;
}
impl<T: NumberOfBytes> NumberOfBytes for [T] {
    fn number_of_bytes(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.len() * self[0].number_of_bytes()
        }
    }
}
impl<T: NumberOfBytes> NumberOfBytes for Vec<T> {
    fn number_of_bytes(&self) -> usize {
        self.as_slice().number_of_bytes()
    }
}
impl<T: NumberOfBytes> NumberOfBytes for &[T] {
    fn number_of_bytes(&self) -> usize {
        (*self).number_of_bytes()
    }
}
