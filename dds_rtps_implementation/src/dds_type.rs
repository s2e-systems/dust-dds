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

#[derive(Debug, PartialEq)]
struct ParameterSerialize<T> {
    parameter_id: u16,
    value: T,
}

impl<T: serde::Serialize> ParameterSerialize<T> {
    fn new(parameter_id: u16, value: T) -> Self {
        Self {
            parameter_id,
            value,
        }
    }
}
const PID_SENTINEL: u16 = 1;

type ParameterListSerialize = Vec<ParameterSerialize<Box<dyn erased_serde::Serialize>>>;
impl DdsSerialize for ParameterListSerialize {
    fn serialize<W: Write, E: Endianness>(&self, mut writer: W) -> DDSResult<()> {
        E::REPRESENTATION_IDENTIFIER.write(&mut writer)?;
        E::REPRESENTATION_OPTIONS.write(&mut writer)?;
        for parameter_i in self {
            parameter_i.serialize::<_, E>(&mut writer).unwrap();
        }
        PID_SENTINEL.serialize::<_, E>(&mut writer)?;
        [0,0].serialize::<_, E>(&mut writer)?;
        Ok(())
    }
}
