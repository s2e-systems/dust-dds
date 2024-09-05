use super::{deserialize::XTypesDeserialize, error::XcdrError};

pub trait DeserializeFinalStruct<'a> {
    fn deserialize_field<T: XTypesDeserialize<'a>>(&mut self, name: &str) -> Result<T, XcdrError>;
    fn deserialize_optional_field<T: XTypesDeserialize<'a>>(
        &mut self,
        name: &str,
    ) -> Result<Option<T>, XcdrError>;
}

pub trait DeserializeAppendableStruct<'a> {
    fn deserialize_field<T: XTypesDeserialize<'a>>(&mut self, name: &str) -> Result<T, XcdrError>;
}

pub trait DeserializeMutableStruct<'a> {
    fn deserialize_field<T: XTypesDeserialize<'a>>(
        &mut self,
        pid: u16,
        name: &str,
    ) -> Result<T, XcdrError>;
    fn deserialize_optional_field<T: XTypesDeserialize<'a>>(
        &mut self,
        pid: u16,
        name: &str,
    ) -> Result<Option<T>, XcdrError>;
}

pub trait DeserializeSequence<'a> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn deserialize_element<T: XTypesDeserialize<'a>>(&mut self) -> Result<T, XcdrError>;
}
pub trait DeserializeArray<'a> {
    fn deserialize_element<T: XTypesDeserialize<'a>>(&mut self) -> Result<T, XcdrError>;
}
/// A trait representing an object with the capability of deserializing a value from a CDR format.
pub trait XTypesDeserializer<'de>: Sized {
    fn deserialize_final_struct(self) -> Result<impl DeserializeFinalStruct<'de>, XcdrError>;
    fn deserialize_appendable_struct(
        self,
    ) -> Result<impl DeserializeAppendableStruct<'de>, XcdrError>;
    fn deserialize_mutable_struct(self) -> Result<impl DeserializeMutableStruct<'de>, XcdrError>;
    fn deserialize_array(self) -> Result<impl DeserializeArray<'de>, XcdrError>;
    fn deserialize_sequence(self) -> Result<impl DeserializeSequence<'de>, XcdrError>;

    /// Deserialize a [`bool`] value.
    fn deserialize_boolean(self) -> Result<bool, XcdrError>;

    /// Deserialize an [`i8`] value.
    fn deserialize_int8(self) -> Result<i8, XcdrError>;

    /// Deserialize an [`i16`] value.
    fn deserialize_int16(self) -> Result<i16, XcdrError>;

    /// Deserialize an [`i32`] value.
    fn deserialize_int32(self) -> Result<i32, XcdrError>;

    /// Deserialize an [`i64`] value.
    fn deserialize_int64(self) -> Result<i64, XcdrError>;

    /// Deserialize a [`u8`] value.
    fn deserialize_uint8(self) -> Result<u8, XcdrError>;

    /// Deserialize a [`u16`] value.
    fn deserialize_uint16(self) -> Result<u16, XcdrError>;

    /// Deserialize a [`u32`] value.
    fn deserialize_uint32(self) -> Result<u32, XcdrError>;

    /// Deserialize a [`u64`] value.
    fn deserialize_uint64(self) -> Result<u64, XcdrError>;

    /// Deserialize an [`f32`] value.
    fn deserialize_float32(self) -> Result<f32, XcdrError>;

    /// Deserialize an [`f64`] value.
    fn deserialize_float64(self) -> Result<f64, XcdrError>;

    /// Deserialize a [`char`] value.
    fn deserialize_char8(self) -> Result<char, XcdrError>;

    /// Deserialize a [`&str`] value.
    fn deserialize_string(self) -> Result<&'de str, XcdrError>;

    /// Deserialize a variable sized sequence of bytes by borrowing.
    fn deserialize_byte_sequence(self) -> Result<&'de [u8], XcdrError>;

    /// Deserialize an array of bytes by borrowing.
    fn deserialize_byte_array<const N: usize>(self) -> Result<&'de [u8; N], XcdrError>;
}
