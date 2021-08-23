use byteorder::{ByteOrder, LittleEndian};
use std::io::Write;

pub type Result = std::result::Result<(), std::io::Error>;

pub trait Serialize {
    fn serialize<W: Write, B: ByteOrder>(&self, writer: W) -> Result;
}

impl<T: Serialize> Serialize for [T] {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> Result {
        for i in self {
            i.serialize::<_,B>(&mut writer)?;
        }
        Ok(())
    }
}
impl<T: Serialize> Serialize for &[T] {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> Result {
        (*self).serialize::<_, B>(&mut writer)
    }
}
impl<T: Serialize> Serialize for Vec<T> {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> Result {
        self.as_slice().serialize::<_, B>(&mut writer)
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