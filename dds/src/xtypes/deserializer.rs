use crate::xtypes::{
    data_representation::DataKind,
    dynamic_type::{DynamicType, MemberDescriptor},
    error::XTypesResult,
};

use super::{deserialize::XTypesDeserialize, error::XTypesError};

pub trait DeserializeFinalStruct<'a> {
    fn deserialize_field(&mut self, descriptor: &MemberDescriptor)
        -> Result<DataKind, XTypesError>;
    fn deserialize_optional_field(
        &mut self,
        descriptor: &MemberDescriptor,
    ) -> Result<Option<DataKind>, XTypesError>;
}

pub trait DeserializeAppendableStruct<'a> {
    fn deserialize_field<T: XTypesDeserialize<'a>>(&mut self, name: &str)
        -> Result<T, XTypesError>;
}

pub trait DeserializeMutableStruct<'a> {
    fn deserialize_field<T: XTypesDeserialize<'a>>(
        &mut self,
        pid: u32,
        name: &str,
    ) -> Result<T, XTypesError>;
    fn deserialize_optional_field<T: XTypesDeserialize<'a>>(
        &mut self,
        pid: u32,
        name: &str,
    ) -> Result<Option<T>, XTypesError>;
}

pub trait DeserializeSequence<'a> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn deserialize_element<T: XTypesDeserialize<'a>>(&mut self) -> Result<T, XTypesError>;
}
pub trait DeserializeArray<'a> {
    fn deserialize_element<T: XTypesDeserialize<'a>>(&mut self) -> Result<T, XTypesError>;
}
/// A trait representing an object with the capability of deserializing a value from a CDR format.
pub trait XTypesDeserializer<'de>: Sized {
    fn deserialize_final_struct(self) -> Result<impl DeserializeFinalStruct<'de>, XTypesError>;
    fn deserialize_appendable_struct(
        self,
    ) -> Result<impl DeserializeAppendableStruct<'de>, XTypesError>;
    fn deserialize_mutable_struct(self) -> Result<impl DeserializeMutableStruct<'de>, XTypesError>;
    fn deserialize_array(self) -> Result<impl DeserializeArray<'de>, XTypesError>;
    fn deserialize_sequence(self) -> Result<impl DeserializeSequence<'de>, XTypesError>;

    /// Deserialize a [`bool`] value.
    fn deserialize_boolean(self) -> Result<bool, XTypesError>;

    /// Deserialize an [`i8`] value.
    fn deserialize_int8(self) -> Result<i8, XTypesError>;

    /// Deserialize an [`i16`] value.
    fn deserialize_int16(self) -> Result<i16, XTypesError>;

    /// Deserialize an [`i32`] value.
    fn deserialize_int32(self) -> Result<i32, XTypesError>;

    /// Deserialize an [`i64`] value.
    fn deserialize_int64(self) -> Result<i64, XTypesError>;

    /// Deserialize a [`u8`] value.
    fn deserialize_uint8(self) -> Result<u8, XTypesError>;

    /// Deserialize a [`u16`] value.
    fn deserialize_uint16(self) -> Result<u16, XTypesError>;

    /// Deserialize a [`u32`] value.
    fn deserialize_uint32(self) -> Result<u32, XTypesError>;

    /// Deserialize a [`u64`] value.
    fn deserialize_uint64(self) -> Result<u64, XTypesError>;

    /// Deserialize an [`f32`] value.
    fn deserialize_float32(self) -> Result<f32, XTypesError>;

    /// Deserialize an [`f64`] value.
    fn deserialize_float64(self) -> Result<f64, XTypesError>;

    /// Deserialize a [`char`] value.
    fn deserialize_char8(self) -> Result<char, XTypesError>;

    /// Deserialize a [`&str`] value.
    fn deserialize_string(self) -> Result<&'de str, XTypesError>;

    /// Deserialize a variable sized sequence of bytes by borrowing.
    fn deserialize_byte_sequence(self) -> Result<&'de [u8], XTypesError>;

    /// Deserialize an array of bytes by borrowing.
    fn deserialize_byte_array<const N: usize>(self) -> Result<&'de [u8; N], XTypesError>;

    fn deserialize_data_kind(self, dynamic_type: &DynamicType) -> XTypesResult<DataKind>;
}
