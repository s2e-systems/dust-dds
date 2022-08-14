use std::{
    io::Error,
    io::{BufRead, Read, Write},
};

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};

use super::mapping_traits::{
    MappingRead, MappingReadByteOrdered, MappingWrite, MappingWriteByteOrdered, NumberOfBytes,
};

impl MappingWrite for u8 {
    fn mapping_write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_u8(*self)
    }
}

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
        if *self { 1_u8 } else { 0 }.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingRead<'de> for u8 {
    fn mapping_read(buf: &mut &'de [u8]) -> Result<Self, Error> {
        buf.read_u8()
    }
}

impl<'de> MappingReadByteOrdered<'de> for u8 {
    fn mapping_read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self, Error>
    where
        B: ByteOrder,
    {
        buf.read_u8()
    }
}

impl<'de> MappingReadByteOrdered<'de> for i8 {
    fn mapping_read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self, Error>
    where
        B: ByteOrder,
    {
        buf.read_i8()
    }
}

impl<'de> MappingReadByteOrdered<'de> for u16 {
    fn mapping_read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self, Error>
    where
        B: ByteOrder,
    {
        buf.read_u16::<B>()
    }
}

impl<'de> MappingReadByteOrdered<'de> for i16 {
    fn mapping_read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self, Error>
    where
        B: ByteOrder,
    {
        buf.read_i16::<B>()
    }
}

impl<'de> MappingReadByteOrdered<'de> for u32 {
    fn mapping_read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self, Error>
    where
        B: ByteOrder,
    {
        buf.read_u32::<B>()
    }
}

impl<'de> MappingReadByteOrdered<'de> for i32 {
    fn mapping_read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self, Error>
    where
        B: ByteOrder,
    {
        buf.read_i32::<B>()
    }
}

impl<'de, const N: usize> MappingReadByteOrdered<'de> for [u8; N] {
    fn mapping_read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self, Error>
    where
        B: ByteOrder,
    {
        let mut value = [0; N];
        buf.read_exact(value.as_mut())?;
        Ok(value)
    }
}

impl<'de> MappingReadByteOrdered<'de> for bool {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let value: u8 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
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
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let length: u32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let length = length as usize;
        let result = std::str::from_utf8(&buf[..length - 1])
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))?;
        buf.consume(length);
        Ok(result)
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
