use std::io::Write;

use byteorder::ByteOrder;

use crate::infrastructure::error::{DdsError, DdsResult};

type RepresentationType = [u8; 2];
pub trait Endianness {
    type Endianness: ByteOrder;
    const REPRESENTATION_IDENTIFIER: RepresentationType;
    const REPRESENTATION_OPTIONS: RepresentationType;
}

pub const PL_CDR_BE: RepresentationType = [0x00, 0x02];
pub const PL_CDR_LE: RepresentationType = [0x00, 0x03];

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

    fn has_key() -> bool {
        false
    }

    fn get_serialized_key<E: Endianness>(&self) -> Vec<u8> {
        if Self::has_key() {
            unimplemented!("DdsType with key must provide an implementation for get_serialized_key")
        } else {
            vec![]
        }
    }

    fn set_key_fields_from_serialized_key(&mut self, _key: &[u8]) -> DdsResult<()> {
        if Self::has_key() {
            unimplemented!("DdsType with key must provide an implementation for set_key_fields_from_serialized_key")
        }
        Ok(())
    }
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
