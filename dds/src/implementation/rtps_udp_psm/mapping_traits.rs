use byteorder::ByteOrder;
use std::io::{Error, Write};

pub trait MappingWriteByteOrderInfoInData {
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, writer: W) -> Result<(), Error>;
}

pub trait MappingWriteByteOrdered {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(&self, writer: W) -> Result<(), Error>;
}

pub trait NumberOfBytes {
    fn number_of_bytes(&self) -> usize;
}

#[allow(dead_code)]
pub fn to_bytes_le<S: MappingWriteByteOrdered>(value: &S) -> Result<Vec<u8>, Error> {
    let mut writer = Vec::<u8>::new();
    value.mapping_write_byte_ordered::<_, byteorder::LittleEndian>(&mut writer)?;
    Ok(writer)
}

pub fn to_bytes<S: MappingWriteByteOrderInfoInData>(value: &S) -> Result<Vec<u8>, Error> {
    let mut writer = Vec::<u8>::new();
    value.mapping_write_byte_order_info_in_data(&mut writer)?;
    Ok(writer)
}
