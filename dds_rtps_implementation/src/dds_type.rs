use std::io::Write;

use byteorder::ByteOrder;
use rust_dds_api::return_type::DDSResult;

type RepresentationType = [u8; 2];
pub trait Representation {
    type Endianness: ByteOrder;
    const REPRESENTATION_IDENTIFIER: RepresentationType;
    const REPRESENTATION_OPTIONS: RepresentationType;
}



pub trait DdsType {
    fn type_name() -> &'static str;

    fn has_key() -> bool;
}

pub trait DdsSerialize {
    fn serialize<W: Write, R: Representation>(&self, writer: W) -> DDSResult<()>;
}
