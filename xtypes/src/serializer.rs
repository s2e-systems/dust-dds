use super::serialize::XTypesSerialize;
use crate::error::XcdrError;

pub trait SerializeFinalStruct {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        name: &str,
    ) -> Result<(), XcdrError>;
    fn serialize_optional_field<T: XTypesSerialize>(
        &mut self,
        value: &Option<T>,
        name: &str,
    ) -> Result<(), XcdrError>;
}
pub trait SerializeAppendableStruct {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        name: &str,
    ) -> Result<(), XcdrError>;
}
pub trait SerializeMutableStruct {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        pid: u16,
        name: &str,
    ) -> Result<(), XcdrError>;
    fn end(self) -> Result<(), XcdrError>;
}
pub trait SerializeCollection {
    fn serialize_element<T: XTypesSerialize>(&mut self, value: &T) -> Result<(), XcdrError>;
}

/// A trait representing an object with the capability of serializing a value into a CDR format.
pub trait XTypesSerializer {
    /// Start serializing a type with final extensibility.
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XcdrError>;

    /// Start serializing a type with appendable extensibility.
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XcdrError>;

    /// Start serializing a type with mutable extensibility.
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XcdrError>;

    /// Start serializing a sequence with a given length.
    fn serialize_sequence(self, len: usize) -> Result<impl SerializeCollection, XcdrError>;

    /// Start serializing a sequence
    fn serialize_array(self) -> Result<impl SerializeCollection, XcdrError>;

    /// Serialize a [`bool`] value.
    fn serialize_boolean(self, v: bool) -> Result<(), XcdrError>;

    /// Serialize an [`i8`] value.
    fn serialize_int8(self, v: i8) -> Result<(), XcdrError>;

    /// Serialize an [`i16`] value.
    fn serialize_int16(self, v: i16) -> Result<(), XcdrError>;

    /// Serialize an [`i32`] value.
    fn serialize_int32(self, v: i32) -> Result<(), XcdrError>;

    /// Serialize an [`i64`] value.
    fn serialize_int64(self, v: i64) -> Result<(), XcdrError>;

    /// Serialize a [`u8`] value.
    fn serialize_uint8(self, v: u8) -> Result<(), XcdrError>;

    /// Serialize a [`u16`] value.
    fn serialize_uint16(self, v: u16) -> Result<(), XcdrError>;

    /// Serialize a [`u32`] value.
    fn serialize_uint32(self, v: u32) -> Result<(), XcdrError>;

    /// Serialize a [`u64`] value.
    fn serialize_uint64(self, v: u64) -> Result<(), XcdrError>;

    /// Serialize an [`f32`] value.
    fn serialize_float32(self, v: f32) -> Result<(), XcdrError>;

    /// Serialize an [`f64`] value.
    fn serialize_float64(self, v: f64) -> Result<(), XcdrError>;

    /// Serialize a [`char`] value.
    fn serialize_char8(self, v: char) -> Result<(), XcdrError>;

    /// Serialize a [`str`] value.
    fn serialize_string(self, v: &str) -> Result<(), XcdrError>;

    /// Serialize a variable sized sequence of bytes.
    fn serialize_byte_sequence(self, v: &[u8]) -> Result<(), XcdrError>;

    /// Serialize an array of bytes.
    fn serialize_byte_array<const N: usize>(self, v: &[u8; N]) -> Result<(), XcdrError>;
}
