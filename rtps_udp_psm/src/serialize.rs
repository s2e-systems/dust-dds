use byteorder::{BigEndian, ByteOrder, LittleEndian};
use rust_rtps_pim::messages::overall_structure::RtpsSubmessageHeader;
use std::io::Write;

pub type Result = std::result::Result<(), std::io::Error>;

pub trait MappingWrite {
    fn mapping_write<W: Write>(&self, writer: W) -> Result;
}

impl<T> MappingWrite for T
where
    T: MappingWriteSubmessage,
{
    fn mapping_write<W: Write>(&self, mut writer: W) -> crate::serialize::Result {
        self.submessage_header().mapping_write(&mut writer)?;
        if self.submessage_header().flags[0] {
            self.mapping_write_submessage_elements::<_, LittleEndian>(&mut writer)
        } else {
            self.mapping_write_submessage_elements::<_, BigEndian>(&mut writer)
        }
    }
}

pub trait MappingWriteSubmessage {
    fn submessage_header(&self) -> RtpsSubmessageHeader;
    fn mapping_write_submessage_elements<W: Write, B: ByteOrder>(
        &self,
        writer: W,
    ) -> crate::serialize::Result;
}

pub trait MappingWriteByteOrdered {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(&self, writer: W) -> Result;
}

pub trait NumberOfBytes {
    fn number_of_bytes(&self) -> usize;
}

pub fn to_writer_le<S: MappingWriteByteOrdered, W: Write>(value: &S, mut writer: W) -> Result {
    value.mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)
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
