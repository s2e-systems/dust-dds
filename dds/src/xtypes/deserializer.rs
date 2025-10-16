use crate::xtypes::{
    data_representation::DataKind, dynamic_type::DynamicType, error::XTypesResult,
};

use super::error::XTypesError;

/// A trait representing an object with the capability of deserializing a value from a CDR format.
pub trait XTypesDeserializer<'de>: Sized {
    fn deserialize_final_struct(&mut self) -> Result<(), XTypesError>;
    fn deserialize_appendable_struct(&mut self) -> Result<(), XTypesError>;
    fn deserialize_mutable_struct(&mut self) -> Result<(), XTypesError>;
    fn deserialize_array(&mut self) -> Result<(), XTypesError>;
    fn deserialize_sequence(&mut self) -> Result<(), XTypesError>;

    /// Deserialize a [`bool`] value.
    fn deserialize_boolean(&mut self) -> Result<bool, XTypesError>;

    /// Deserialize an [`i8`] value.
    fn deserialize_int8(&mut self) -> Result<i8, XTypesError>;

    /// Deserialize an [`i16`] value.
    fn deserialize_int16(&mut self) -> Result<i16, XTypesError>;

    /// Deserialize an [`i32`] value.
    fn deserialize_int32(&mut self) -> Result<i32, XTypesError>;

    /// Deserialize an [`i64`] value.
    fn deserialize_int64(&mut self) -> Result<i64, XTypesError>;

    /// Deserialize a [`u8`] value.
    fn deserialize_uint8(&mut self) -> Result<u8, XTypesError>;

    /// Deserialize a [`u16`] value.
    fn deserialize_uint16(&mut self) -> Result<u16, XTypesError>;

    /// Deserialize a [`u32`] value.
    fn deserialize_uint32(&mut self) -> Result<u32, XTypesError>;

    /// Deserialize a [`u64`] value.
    fn deserialize_uint64(&mut self) -> Result<u64, XTypesError>;

    /// Deserialize an [`f32`] value.
    fn deserialize_float32(&mut self) -> Result<f32, XTypesError>;

    /// Deserialize an [`f64`] value.
    fn deserialize_float64(&mut self) -> Result<f64, XTypesError>;

    /// Deserialize a [`char`] value.
    fn deserialize_char8(&mut self) -> Result<char, XTypesError>;

    /// Deserialize a [`&str`] value.
    fn deserialize_string(&mut self) -> Result<&'de str, XTypesError>;

    /// Deserialize a variable sized sequence of bytes by borrowing.
    fn deserialize_byte_sequence(&mut self) -> Result<&'de [u8], XTypesError>;

    /// Deserialize an array of bytes by borrowing.
    fn deserialize_byte_array<const N: usize>(&mut self) -> Result<&'de [u8; N], XTypesError>;

    fn deserialize_data_kind(&mut self, dynamic_type: &DynamicType) -> XTypesResult<DataKind>;
}
