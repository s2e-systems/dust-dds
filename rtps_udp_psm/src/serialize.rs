use byteorder::{BigEndian, ByteOrder, LittleEndian, WriteBytesExt};
use rust_rtps_pim::messages::overall_structure::RtpsSubmessageHeader;
use std::io::Write;

pub type Result = std::result::Result<(), std::io::Error>;

pub trait MappingWrite {
    fn write<W: Write>(&self, writer: W) -> Result;
}

impl<T> MappingWrite for T
where
    T: SerializeSubmessage,
{
    fn write<W: Write>(&self, mut writer: W) -> crate::serialize::Result {
        self.serialize_submessage(&mut writer)
    }
}

pub trait SerializeSubmessage {
    fn serialize_submessage<W: Write>(&self, mut writer: W) -> crate::serialize::Result {
        self.submessage_header().write(&mut writer)?;
        if self.submessage_header().flags[0] {
            self.serialize_submessage_elements::<_, LittleEndian>(&mut writer)
        } else {
            self.serialize_submessage_elements::<_, BigEndian>(&mut writer)
        }
    }
    fn submessage_header(&self) -> RtpsSubmessageHeader;
    fn serialize_submessage_elements<W: Write, B: ByteOrder>(
        &self,
        writer: W,
    ) -> crate::serialize::Result;
}

pub trait MappingWriteByteOrdered {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, writer: W) -> Result;
}

impl MappingWriteByteOrdered for u8 {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> Result {
        writer.write_u8(*self)
    }
}
impl MappingWrite for u8 {
    fn write<W: Write>(&self, mut writer: W) -> Result {
        writer.write_u8(*self)
    }
}

impl MappingWriteByteOrdered for i8 {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> Result {
        writer.write_i8(*self)
    }
}

impl MappingWriteByteOrdered for u16 {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> Result {
        writer.write_u16::<B>(*self)
    }
}

impl MappingWriteByteOrdered for i16 {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> Result {
        writer.write_i16::<B>(*self)
    }
}

impl MappingWriteByteOrdered for u32 {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> Result {
        writer.write_u32::<B>(*self)
    }
}

impl MappingWriteByteOrdered for i32 {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> Result {
        writer.write_i32::<B>(*self)
    }
}

impl<const N: usize> MappingWriteByteOrdered for [u8; N] {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> Result {
        writer.write_all(self)?;
        Ok(())
    }
}
impl<const N: usize> MappingWrite for [u8; N] {
    fn write<W: Write>(&self, mut writer: W) -> Result {
        writer.write_all(self)
    }
}

impl MappingWriteByteOrdered for &str {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> Result {
        let length = self.as_bytes().len() as u32 + 1;
        length.write_byte_ordered::<_, B>(&mut writer)?;
        writer.write_all(self.as_bytes())?;
        writer.write_u8(0)
    }
}

impl MappingWriteByteOrdered for bool {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> Result {
        if *self { 1_u8 } else { 0 }.write_byte_ordered::<_, B>(&mut writer)
    }
}

pub fn to_writer_le<S: MappingWriteByteOrdered, W: Write>(value: &S, mut writer: W) -> Result {
    value.write_byte_ordered::<_, LittleEndian>(&mut writer)
}

pub fn to_bytes_le<S: MappingWriteByteOrdered>(value: &S) -> std::result::Result<Vec<u8>, std::io::Error> {
    let mut writer = Vec::<u8>::new();
    value.write_byte_ordered::<_, LittleEndian>(&mut writer)?;
    Ok(writer)
}

pub fn to_bytes<S: MappingWrite>(value: &S) -> std::result::Result<Vec<u8>, std::io::Error> {
    let mut writer = Vec::<u8>::new();
    value.write(&mut writer)?;
    Ok(writer)
}

pub trait NumberOfBytes {
    fn number_of_bytes(&self) -> usize;
}
impl NumberOfBytes for bool {
    fn number_of_bytes(&self) -> usize {
        1
    }
}
impl NumberOfBytes for u8 {
    fn number_of_bytes(&self) -> usize {
        1
    }
}
impl NumberOfBytes for u32 {
    fn number_of_bytes(&self) -> usize {
        4
    }
}
impl NumberOfBytes for &str {
    fn number_of_bytes(&self) -> usize {
        4 + self.as_bytes().len() + 1
    }
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
impl<T: NumberOfBytes, const N: usize> NumberOfBytes for [T; N] {
    fn number_of_bytes(&self) -> usize {
        self.as_ref().number_of_bytes()
    }
}
