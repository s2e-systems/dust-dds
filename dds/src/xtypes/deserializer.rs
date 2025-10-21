use crate::xtypes::{
    data_representation::endianness::EndiannessRead, dynamic_type::DynamicData, error::XTypesResult,
};

use super::error::XTypesError;

pub trait PlainCdrDeserializer {
    fn read_u32(&mut self) -> XTypesResult<u32>;
}

pub trait PlainCdrDecode: Sized {
    fn decode(reader: &mut impl PlainCdrDeserializer) -> XTypesResult<Self>;
}

impl PlainCdrDecode for u32 {
    fn decode(reader: &mut impl PlainCdrDeserializer) -> XTypesResult<Self> {
        reader.read_u32()
    }
}

/// A trait representing an object with the capability of deserializing a value from a CDR format.
pub trait XTypesDeserializer<'de>: Sized {
    fn deserialize_final_struct(&mut self, v: &mut DynamicData) -> Result<(), XTypesError>;
    fn deserialize_appendable_struct(&mut self) -> Result<(), XTypesError>;
    fn deserialize_mutable_struct(&mut self) -> Result<(), XTypesError>;
    fn deserialize_array(&mut self) -> Result<(), XTypesError>;
    fn deserialize_sequence(&mut self) -> Result<(), XTypesError>;

    /// Deserialize a [`&str`] value.
    fn deserialize_string(&mut self) -> Result<&'de str, XTypesError>;

    /// Deserialize a variable sized sequence of bytes by borrowing.
    fn deserialize_byte_sequence(&mut self) -> Result<&'de [u8], XTypesError>;

    /// Deserialize an array of bytes by borrowing.
    fn deserialize_byte_array<const N: usize>(&mut self) -> Result<&'de [u8; N], XTypesError>;
}
