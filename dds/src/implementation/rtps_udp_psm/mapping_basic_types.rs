use super::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes};
use byteorder::{ByteOrder, WriteBytesExt};
use std::{io::Error, io::Write};

impl MappingWriteByteOrdered for u8 {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        writer.write_u8(*self)
    }
}

impl MappingWriteByteOrdered for i8 {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        writer.write_i8(*self)
    }
}

impl MappingWriteByteOrdered for u16 {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        writer.write_u16::<B>(*self)
    }
}

impl MappingWriteByteOrdered for i16 {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        writer.write_i16::<B>(*self)
    }
}

impl MappingWriteByteOrdered for u32 {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        writer.write_u32::<B>(*self)
    }
}

impl MappingWriteByteOrdered for i32 {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        writer.write_i32::<B>(*self)
    }
}

impl<const N: usize> MappingWriteByteOrdered for [u8; N] {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        writer.write_all(self)?;
        Ok(())
    }
}

impl MappingWriteByteOrdered for &str {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        let length = self.as_bytes().len() as u32 + 1;
        length.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        writer.write_all(self.as_bytes())?;
        writer.write_u8(0)
    }
}

impl MappingWriteByteOrdered for bool {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        u8::from(*self).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
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
