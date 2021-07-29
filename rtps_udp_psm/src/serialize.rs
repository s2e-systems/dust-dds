use byteorder::{ByteOrder, LittleEndian};
use std::io::Write;

pub type Result = std::result::Result<(), std::io::Error>;

pub trait Serialize {
    fn serialize<W: Write, B: ByteOrder>(&self, writer: W) -> Result;
}

pub fn to_writer_le<S: Serialize, W: Write>(value: &S, mut writer: W) -> Result {
    value.serialize::<_, LittleEndian>(&mut writer)
}

pub fn to_bytes_le<S: Serialize>(value: &S) -> std::result::Result<Vec<u8>, std::io::Error> {
    let mut writer = Vec::<u8>::new();
    value.serialize::<_, LittleEndian>(&mut writer)?;
    Ok(writer)
}
