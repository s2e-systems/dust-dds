use byteorder::{ByteOrder, LittleEndian};
use std::io::Write;

pub type Result = std::result::Result<(), std::io::Error>;

pub trait MappingWrite {
    fn mapping_write<W: Write>(&self, writer: W) -> Result;
}

pub trait MappingWriteByteOrdered {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(&self, writer: W) -> Result;
}

pub trait NumberOfBytes {
    fn number_of_bytes(&self) -> usize;
}

pub fn to_bytes_le<S: MappingWriteByteOrdered>(
    value: &S,
) -> std::result::Result<Vec<u8>, std::io::Error> {
    let mut writer = Vec::<u8>::new();
    value.mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)?;
    Ok(writer)
}

pub fn to_bytes<S: MappingWrite>(value: &S) -> std::result::Result<Vec<u8>, std::io::Error> {
    let mut writer = Vec::<u8>::new();
    value.mapping_write(&mut writer)?;
    Ok(writer)
}
