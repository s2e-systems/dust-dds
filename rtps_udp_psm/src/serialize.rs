use byteorder::{ByteOrder, LittleEndian};
use std::{io::{Error, Write}, result::Result};

// pub type Result = Result<(), Error>;

pub trait MappingWrite {
    fn mapping_write<W: Write>(&self, writer: W) -> Result<(), Error>;
}

pub trait MappingWriteByteOrdered {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(&self, writer: W) -> Result<(), Error>;
}

pub trait NumberOfBytes {
    fn number_of_bytes(&self) -> usize;
}

pub fn to_bytes_le<S: MappingWriteByteOrdered>(
    value: &S,
) -> Result<Vec<u8>, Error> {
    let mut writer = Vec::<u8>::new();
    value.mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)?;
    Ok(writer)
}

pub fn to_bytes<S: MappingWrite>(value: &S) -> Result<Vec<u8>, Error> {
    let mut writer = Vec::<u8>::new();
    value.mapping_write(&mut writer)?;
    Ok(writer)
}
