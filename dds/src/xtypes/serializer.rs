use crate::xtypes::{
    data_representation::DataKind,
    dynamic_type::{DynamicData, DynamicType},
    error::XTypesError,
};

pub trait SerializeFinalStruct {
    fn serialize_field(&mut self, value: &DataKind, name: &str) -> Result<(), XTypesError>;
    fn serialize_optional_field(
        &mut self,
        value: &Option<DynamicData>,
        name: &str,
    ) -> Result<(), XTypesError>;
}
pub trait SerializeAppendableStruct {
    fn serialize_field(&mut self, value: &DataKind, name: &str) -> Result<(), XTypesError>;
}
pub trait SerializeMutableStruct {
    fn serialize_field(
        &mut self,
        value: &DataKind,
        pid: u32,
        name: &str,
    ) -> Result<(), XTypesError>;
    fn serialize_collection(
        &mut self,
        values: &[DynamicData],
        pid: u32,
        name: &str,
    ) -> Result<(), XTypesError> {
        todo!()
        //self.serialize_field(values, pid, name)
    }
    fn end(self) -> Result<(), XTypesError>;
}

/// A trait representing an object with the capability of serializing a value into a CDR format.
pub trait XTypesSerializer {
    /// Start serializing a type with final extensibility.
    fn serialize_final_struct(self) -> Result<impl SerializeFinalStruct, XTypesError>;

    /// Start serializing a type with appendable extensibility.
    fn serialize_appendable_struct(self) -> Result<impl SerializeAppendableStruct, XTypesError>;

    /// Start serializing a type with mutable extensibility.
    fn serialize_mutable_struct(self) -> Result<impl SerializeMutableStruct, XTypesError>;

    /// Serializing a sequence with a dynamic length
    fn serialize_sequence(self, vs: &[DynamicData]) -> Result<(), XTypesError>;

    /// Serializing a sequence with a static length
    fn serialize_array(self, vs: &[DynamicData]) -> Result<(), XTypesError>;

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

    /// Serialize a list of [`bool`] values.
    fn serialize_boolean_list(self, vs: &[bool]) -> Result<(), XTypesError>;

    /// Serialize a list of [`u8`] values.
    fn serialize_uint8_list(self, vs: &[u8]) -> Result<(), XTypesError>;

    /// Serialize a list of [`i8`] values.
    fn serialize_int8_list(self, vs: &[i8]) -> Result<(), XTypesError>;

    /// Serialize a list of [`i16`] values.
    fn serialize_int16_list(self, vs: &[i16]) -> Result<(), XTypesError>;

    /// Serialize a list of [`i32`] values.
    fn serialize_int32_list(self, vs: &[i32]) -> Result<(), XTypesError>;

    /// Serialize a list of [`i64`] values.
    fn serialize_int64_list(self, vs: &[i64]) -> Result<(), XTypesError>;

    /// Serialize a list of [`u16`] values.
    fn serialize_uint16_list(self, vs: &[u16]) -> Result<(), XTypesError>;

    /// Serialize a list of [`u32`] values.
    fn serialize_uint32_list(self, vs: &[u32]) -> Result<(), XTypesError>;

    /// Serialize a list of [`u64`] values.
    fn serialize_uint64_list(self, vs: &[u64]) -> Result<(), XTypesError>;

    /// Serialize a list of [`f32`] values.
    fn serialize_float32_list(self, vs: &[f32]) -> Result<(), XTypesError>;

    /// Serialize a list of [`f64`] values.
    fn serialize_float64_list(self, vs: &[f64]) -> Result<(), XTypesError>;

    /// Serialize a list of [`char`] values.
    fn serialize_char8_list(self, vs: &[char]) -> Result<(), XTypesError>;

    /// Serialize a list of [`String`] values.
    fn serialize_string_list(self, vs: &[String]) -> Result<(), XTypesError>;


    /// Serialize a list of [`bool`] values.
    fn serialize_boolean_array(self, vs: &[bool]) -> Result<(), XTypesError>;

    /// Serialize an array of [`u8`] values.
    fn serialize_uint8_array(self, vs: &[u8]) -> Result<(), XTypesError>;

    /// Serialize an array of [`i8`] values.
    fn serialize_int8_array(self, vs: &[i8]) -> Result<(), XTypesError>;

    /// Serialize an array of [`i16`] values.
    fn serialize_int16_array(self, vs: &[i16]) -> Result<(), XTypesError>;

    /// Serialize an array of [`i32`] values.
    fn serialize_int32_array(self, vs: &[i32]) -> Result<(), XTypesError>;

    /// Serialize an array of [`i64`] values.
    fn serialize_int64_array(self, vs: &[i64]) -> Result<(), XTypesError>;

    /// Serialize an array of [`u16`] values.
    fn serialize_uint16_array(self, vs: &[u16]) -> Result<(), XTypesError>;

    /// Serialize an array of [`u32`] values.
    fn serialize_uint32_array(self, vs: &[u32]) -> Result<(), XTypesError>;

    /// Serialize an array of [`u64`] values.
    fn serialize_uint64_array(self, vs: &[u64]) -> Result<(), XTypesError>;

    /// Serialize an array of [`f32`] values.
    fn serialize_float32_array(self, vs: &[f32]) -> Result<(), XTypesError>;

    /// Serialize an array of [`f64`] values.
    fn serialize_float64_array(self, vs: &[f64]) -> Result<(), XTypesError>;

    /// Serialize an array of [`char`] values.
    fn serialize_char8_array(self, vs: &[char]) -> Result<(), XTypesError>;

    /// Serialize an array of [`String`] values.
    fn serialize_string_array(self, vs: &[String]) -> Result<(), XTypesError>;
}
