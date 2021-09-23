use std::io::Write;

use byteorder::ByteOrder;
use rust_dds_api::return_type::DDSResult;

type RepresentationType = [u8; 2];
pub trait Endianness {
    type Endianness: ByteOrder;
    const REPRESENTATION_IDENTIFIER: RepresentationType;
    const REPRESENTATION_OPTIONS: RepresentationType;
}

const PL_CDR_BE: RepresentationType = [0x00, 0x02];
const PL_CDR_LE: RepresentationType = [0x00, 0x03];

pub enum LittleEndian {}
impl Endianness for LittleEndian {
    type Endianness = byteorder::LittleEndian;
    const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;
    const REPRESENTATION_OPTIONS: RepresentationType = [0, 0];
}

pub enum BigEndian {}
impl Endianness for BigEndian {
    type Endianness = byteorder::BigEndian;
    const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_BE;
    const REPRESENTATION_OPTIONS: RepresentationType = [0, 0];
}

pub trait DdsType {
    fn type_name() -> &'static str;

    fn has_key() -> bool;
}

pub trait DdsSerialize {
    fn serialize<W: Write, E: Endianness>(&self, writer: W) -> DDSResult<()>;
}

pub trait DdsDeserialize<'de>: Sized {
    fn deserialize(buf: &mut &'de [u8]) -> DDSResult<Self>;
}


impl<T> DdsSerialize for &'_ T where T: DdsSerialize {
    fn serialize<W: Write, E: Endianness>(&self, writer: W) -> DDSResult<()> {
        (*self).serialize::<W, E>(writer)
    }
}