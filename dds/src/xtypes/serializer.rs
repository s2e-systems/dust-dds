use super::{error::XTypesError, serialize::XTypesSerialize};

pub trait SerializeFinalStruct {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        name: &str,
    ) -> Result<(), XTypesError>;
    fn serialize_optional_field<T: XTypesSerialize>(
        &mut self,
        value: &Option<T>,
        name: &str,
    ) -> Result<(), XTypesError>;
}
pub trait SerializeAppendableStruct {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        name: &str,
    ) -> Result<(), XTypesError>;
}
pub trait SerializeMutableStruct {
    fn serialize_field<T: XTypesSerialize>(
        &mut self,
        value: &T,
        pid: u32,
        name: &str,
    ) -> Result<(), XTypesError>;
    fn serialize_collection<T: XTypesSerialize>(
        &mut self,
        values: &[T],
        pid: u32,
        name: &str,
    ) -> Result<(), XTypesError>{
        self.serialize_field(&values, pid, name)
    }
    fn end(self) -> Result<(), XTypesError>;
}
pub trait SerializeCollection {
    fn serialize_element<T: XTypesSerialize>(&mut self, value: &T) -> Result<(), XTypesError>;
}

/// A trait representing an object with the capability of serializing a value into a CDR format.
pub trait XTypesSerializer {
    /// Start serializing a type with final extensibility.
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError>;

    /// Start serializing a type with appendable extensibility.
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError>;

    /// Start serializing a type with mutable extensibility.
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError>;

    /// Start serializing a sequence with a given length.
    fn serialize_sequence(self, len: usize) -> Result<impl SerializeCollection, XTypesError>;

    /// Start serializing a sequence
    fn serialize_array(self) -> Result<impl SerializeCollection, XTypesError>;

    /// Serialize a [`bool`] value.
    fn serialize_boolean(self, v: bool) -> Result<(), XTypesError>;

    /// Serialize an [`i8`] value.
    fn serialize_int8(self, v: i8) -> Result<(), XTypesError>;

    /// Serialize an [`i16`] value.
    fn serialize_int16(self, v: i16) -> Result<(), XTypesError>;

    /// Serialize an [`i32`] value.
    fn serialize_int32(self, v: i32) -> Result<(), XTypesError>;

    /// Serialize an [`i64`] value.
    fn serialize_int64(self, v: i64) -> Result<(), XTypesError>;

    /// Serialize a [`u8`] value.
    fn serialize_uint8(self, v: u8) -> Result<(), XTypesError>;

    /// Serialize a [`u16`] value.
    fn serialize_uint16(self, v: u16) -> Result<(), XTypesError>;

    /// Serialize a [`u32`] value.
    fn serialize_uint32(self, v: u32) -> Result<(), XTypesError>;

    /// Serialize a [`u64`] value.
    fn serialize_uint64(self, v: u64) -> Result<(), XTypesError>;

    /// Serialize an [`f32`] value.
    fn serialize_float32(self, v: f32) -> Result<(), XTypesError>;

    /// Serialize an [`f64`] value.
    fn serialize_float64(self, v: f64) -> Result<(), XTypesError>;

    /// Serialize a [`char`] value.
    fn serialize_char8(self, v: char) -> Result<(), XTypesError>;

    /// Serialize a [`str`] value.
    fn serialize_string(self, v: &str) -> Result<(), XTypesError>;

    /// Serialize a variable sized sequence of bytes.
    fn serialize_byte_sequence(self, v: &[u8]) -> Result<(), XTypesError>;

    /// Serialize an array of bytes.
    fn serialize_byte_array<const N: usize>(self, v: &[u8; N]) -> Result<(), XTypesError>;
}
