use std::io::Write;

use byteorder::ByteOrder;
use dds_api::return_type::{DdsError, DdsResult};

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
    fn serialize<W: Write, E: Endianness>(&self, writer: W) -> DdsResult<()>;
}

pub trait DdsDeserialize<'de>: Sized {
    fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self>;
}

pub trait DdsSerde {}

impl<Foo> DdsSerialize for Foo
where
    Foo: serde::Serialize + DdsSerde,
{
    fn serialize<W: Write, E: Endianness>(&self, mut writer: W) -> DdsResult<()> {
        if E::REPRESENTATION_IDENTIFIER == PL_CDR_BE {
            writer
                .write(
                    cdr::serialize::<_, _, cdr::CdrBe>(self, cdr::Infinite)
                        .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?
                        .as_slice(),
                )
                .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?;
        } else {
            writer
                .write(
                    cdr::serialize::<_, _, cdr::CdrLe>(self, cdr::Infinite)
                        .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?
                        .as_slice(),
                )
                .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?;
        }
        Ok(())
    }
}

impl<'de, Foo> DdsDeserialize<'de> for Foo
where
    Foo: serde::Deserialize<'de> + DdsSerde,
{
    fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
        cdr::deserialize(buf).map_err(|e| DdsError::PreconditionNotMet(e.to_string()))
    }
}