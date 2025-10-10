use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::{
        dynamic_type::{DynamicData, ExtensibilityKind, TypeKind},
        serializer::{
            SerializeAppendableStruct, SerializeFinalStruct, SerializeMutableStruct,
            XTypesSerializer,
        },
    },
};
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum DataKind {
    UInt8(u8),
    Int8(i8),
    UInt16(u16),
    Int16(i16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    Char8(char),
    Boolean(bool),
    String(String),
    ComplexValue(DynamicData),
    UInt8List(Vec<u8>),
    Int8List(Vec<i8>),
    UInt16List(Vec<u16>),
    Int16List(Vec<i16>),
    Int32List(Vec<i32>),
    UInt32List(Vec<u32>),
    Int64List(Vec<i64>),
    UInt64List(Vec<u64>),
    Float32List(Vec<f32>),
    Float64List(Vec<f64>),
    Char8List(Vec<char>),
    BooleanList(Vec<bool>),
    StringList(Vec<String>),
    ComplexValueList(Vec<DynamicData>),
    UInt8Array(Vec<u8>),
    Int8Array(Vec<i8>),
    UInt16Array(Vec<u16>),
    Int16Array(Vec<i16>),
    Int32Array(Vec<i32>),
    UInt32Array(Vec<u32>),
    Int64Array(Vec<i64>),
    UInt64Array(Vec<u64>),
    Float32Array(Vec<f32>),
    Float64Array(Vec<f64>),
    Char8Array(Vec<char>),
    BooleanArray(Vec<bool>),
    StringArray(Vec<String>),
    ComplexValueArray(Vec<DynamicData>),
}

impl DataKind {
    pub fn serialize<T>(&self, serializer: &mut T) -> Result<(), super::error::XTypesError>
    where
        for<'a> &'a mut T: XTypesSerializer,
    {
        match self {
            DataKind::UInt8(v) => serializer.serialize_uint8(*v),
            DataKind::Int8(v) => serializer.serialize_int8(*v),
            DataKind::UInt16(v) => serializer.serialize_uint16(*v),
            DataKind::Int16(v) => serializer.serialize_int16(*v),
            DataKind::Int32(v) => serializer.serialize_int32(*v),
            DataKind::UInt32(v) => serializer.serialize_uint32(*v),
            DataKind::Int64(v) => serializer.serialize_int64(*v),
            DataKind::UInt64(v) => serializer.serialize_uint64(*v),
            DataKind::Float32(v) => serializer.serialize_float32(*v),
            DataKind::Float64(v) => serializer.serialize_float64(*v),
            DataKind::Char8(v) => serializer.serialize_char8(*v),
            DataKind::Boolean(v) => serializer.serialize_boolean(*v),
            DataKind::String(v) => serializer.serialize_string(v),
            DataKind::ComplexValue(dynamic_data) => dynamic_data.serialize_nested(serializer),
            DataKind::UInt8List(items) => serializer.serialize_uint8_list(items),
            DataKind::Int8List(items) => serializer.serialize_int8_list(items),
            DataKind::UInt16List(items) => serializer.serialize_uint16_list(items),
            DataKind::Int16List(items) => serializer.serialize_int16_list(items),
            DataKind::Int32List(items) => serializer.serialize_int32_list(items),
            DataKind::UInt32List(items) => serializer.serialize_uint32_list(items),
            DataKind::Int64List(items) => serializer.serialize_int64_list(items),
            DataKind::UInt64List(items) => serializer.serialize_uint64_list(items),
            DataKind::Float32List(items) => serializer.serialize_float32_list(items),
            DataKind::Float64List(items) => serializer.serialize_float64_list(items),
            DataKind::Char8List(items) => serializer.serialize_char8_list(items),
            DataKind::BooleanList(items) => serializer.serialize_boolean_list(items),
            DataKind::StringList(items) => serializer.serialize_string_list(items),
            DataKind::ComplexValueList(items) => serializer.serialize_sequence(items),
            DataKind::UInt8Array(items) => serializer.serialize_uint8_array(items),
            DataKind::Int8Array(items) => serializer.serialize_int8_array(items),
            DataKind::UInt16Array(items) => serializer.serialize_uint16_array(items),
            DataKind::Int16Array(items) => serializer.serialize_int16_array(items),
            DataKind::Int32Array(items) => serializer.serialize_int32_array(items),
            DataKind::UInt32Array(items) => serializer.serialize_uint32_array(items),
            DataKind::Int64Array(items) => serializer.serialize_int64_array(items),
            DataKind::UInt64Array(items) => serializer.serialize_uint64_array(items),
            DataKind::Float32Array(items) => serializer.serialize_float32_array(items),
            DataKind::Float64Array(items) => serializer.serialize_float64_array(items),
            DataKind::Char8Array(items) => serializer.serialize_char8_array(items),
            DataKind::BooleanArray(items) => serializer.serialize_boolean_array(items),
            DataKind::StringArray(items) => serializer.serialize_string_array(items),
            DataKind::ComplexValueArray(items) => serializer.serialize_array(items),
        }
    }
}

impl From<u8> for DataKind {
    fn from(value: u8) -> Self {
        Self::UInt8(value)
    }
}

impl From<i8> for DataKind {
    fn from(value: i8) -> Self {
        Self::Int8(value)
    }
}

impl From<u16> for DataKind {
    fn from(value: u16) -> Self {
        Self::UInt16(value)
    }
}

impl From<i16> for DataKind {
    fn from(value: i16) -> Self {
        Self::Int16(value)
    }
}

impl From<u32> for DataKind {
    fn from(value: u32) -> Self {
        Self::UInt32(value)
    }
}

impl From<i32> for DataKind {
    fn from(value: i32) -> Self {
        Self::Int32(value)
    }
}

impl From<u64> for DataKind {
    fn from(value: u64) -> Self {
        Self::UInt64(value)
    }
}

impl From<i64> for DataKind {
    fn from(value: i64) -> Self {
        Self::Int64(value)
    }
}

impl From<f32> for DataKind {
    fn from(value: f32) -> Self {
        Self::Float32(value)
    }
}

impl From<f64> for DataKind {
    fn from(value: f64) -> Self {
        Self::Float64(value)
    }
}

impl From<char> for DataKind {
    fn from(value: char) -> Self {
        Self::Char8(value)
    }
}

impl From<bool> for DataKind {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<String> for DataKind {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

// impl<T: TypeSupport> From<T> for DataKind {
//     fn from(value: T) -> Self {
//         Self::ComplexValue(value.create_dynamic_sample())
//     }
// }

impl<const N: usize> From<[u8; N]> for DataKind {
    fn from(value: [u8; N]) -> Self {
        Self::UInt8Array(value.to_vec())
    }
}

impl<const N: usize> From<[i8; N]> for DataKind {
    fn from(value: [i8; N]) -> Self {
        Self::Int8List(value.to_vec())
    }
}

impl<const N: usize> From<[u16; N]> for DataKind {
    fn from(value: [u16; N]) -> Self {
        Self::UInt16List(value.to_vec())
    }
}

impl<const N: usize> From<[i16; N]> for DataKind {
    fn from(value: [i16; N]) -> Self {
        Self::Int16List(value.to_vec())
    }
}

impl<const N: usize> From<[u32; N]> for DataKind {
    fn from(value: [u32; N]) -> Self {
        Self::UInt32List(value.to_vec())
    }
}

impl<const N: usize> From<[i32; N]> for DataKind {
    fn from(value: [i32; N]) -> Self {
        Self::Int32List(value.to_vec())
    }
}

impl<const N: usize> From<[u64; N]> for DataKind {
    fn from(value: [u64; N]) -> Self {
        Self::UInt64List(value.to_vec())
    }
}

impl<const N: usize> From<[i64; N]> for DataKind {
    fn from(value: [i64; N]) -> Self {
        Self::Int64List(value.to_vec())
    }
}

impl<const N: usize> From<[f32; N]> for DataKind {
    fn from(value: [f32; N]) -> Self {
        Self::Float32List(value.to_vec())
    }
}

impl<const N: usize> From<[f64; N]> for DataKind {
    fn from(value: [f64; N]) -> Self {
        Self::Float64List(value.to_vec())
    }
}

impl<const N: usize> From<[char; N]> for DataKind {
    fn from(value: [char; N]) -> Self {
        Self::Char8List(value.to_vec())
    }
}

impl<const N: usize> From<[bool; N]> for DataKind {
    fn from(value: [bool; N]) -> Self {
        Self::BooleanList(value.to_vec())
    }
}

impl<const N: usize> From<[String; N]> for DataKind {
    fn from(value: [String; N]) -> Self {
        Self::StringList(value.to_vec())
    }
}

impl<const N: usize, T: TypeSupport> From<[T; N]> for DataKind {
    fn from(value: [T; N]) -> Self {
        Self::ComplexValueList(
            value
                .into_iter()
                .map(TypeSupport::create_dynamic_sample)
                .collect(),
        )
    }
}

impl From<Vec<u8>> for DataKind {
    fn from(value: Vec<u8>) -> Self {
        Self::UInt8Array(value)
    }
}

impl From<Vec<i8>> for DataKind {
    fn from(value: Vec<i8>) -> Self {
        Self::Int8List(value)
    }
}

impl From<Vec<u16>> for DataKind {
    fn from(value: Vec<u16>) -> Self {
        Self::UInt16List(value)
    }
}

impl From<Vec<i16>> for DataKind {
    fn from(value: Vec<i16>) -> Self {
        Self::Int16List(value)
    }
}

impl From<Vec<u32>> for DataKind {
    fn from(value: Vec<u32>) -> Self {
        Self::UInt32List(value)
    }
}

impl From<Vec<i32>> for DataKind {
    fn from(value: Vec<i32>) -> Self {
        Self::Int32List(value)
    }
}

impl From<Vec<u64>> for DataKind {
    fn from(value: Vec<u64>) -> Self {
        Self::UInt64List(value)
    }
}

impl From<Vec<i64>> for DataKind {
    fn from(value: Vec<i64>) -> Self {
        Self::Int64List(value)
    }
}

impl From<Vec<f32>> for DataKind {
    fn from(value: Vec<f32>) -> Self {
        Self::Float32List(value)
    }
}

impl From<Vec<f64>> for DataKind {
    fn from(value: Vec<f64>) -> Self {
        Self::Float64List(value)
    }
}

impl From<Vec<char>> for DataKind {
    fn from(value: Vec<char>) -> Self {
        Self::Char8List(value)
    }
}

impl From<Vec<bool>> for DataKind {
    fn from(value: Vec<bool>) -> Self {
        Self::BooleanList(value)
    }
}

impl From<Vec<String>> for DataKind {
    fn from(value: Vec<String>) -> Self {
        Self::StringList(value)
    }
}

impl From<&[u8]> for DataKind {
    fn from(value: &[u8]) -> Self {
        Self::UInt8Array(value.to_vec())
    }
}

impl<T: TypeSupport> From<Vec<T>> for DataKind {
    fn from(value: Vec<T>) -> Self {
        Self::ComplexValueList(
            value
                .into_iter()
                .map(TypeSupport::create_dynamic_sample)
                .collect(),
        )
    }
}

impl<T: TypeSupport> From<T> for DataKind {
    fn from(value: T) -> Self {
        DataKind::ComplexValue(value.create_dynamic_sample())
    }
}

impl DynamicData {
    pub fn serialize<T>(&self, serializer: &mut T) -> Result<(), super::error::XTypesError>
    where
        for<'a> &'a mut T: XTypesSerializer,
    {
        self.serialize_nested(serializer)
    }

    pub fn serialize_nested<T>(&self, serializer: &mut T) -> Result<(), super::error::XTypesError>
    where
        for<'a> &'a mut T: XTypesSerializer,
    {
        match self.type_ref().get_kind() {
            TypeKind::ENUM => {
                self.get_value(0)?.serialize(&mut *serializer)?;
            }
            TypeKind::STRUCTURE => match self.type_ref().get_descriptor().extensibility_kind {
                ExtensibilityKind::Final => {
                    let mut final_serializer = serializer.serialize_final_struct()?;
                    for field_index in 0..self.get_item_count() {
                        let member_id = self.get_member_id_at_index(field_index)?;
                        let member_descriptor = self.get_descriptor(member_id)?;
                        final_serializer
                            .serialize_field(self.get_value(member_id)?, &member_descriptor.name)?;
                    }
                }
                ExtensibilityKind::Appendable => {
                    let mut appendable_serializer = serializer.serialize_appendable_struct()?;
                    for field_index in 0..self.get_item_count() {
                        let member_id = self.get_member_id_at_index(field_index)?;
                        let member_descriptor = self.get_descriptor(member_id)?;
                        appendable_serializer
                            .serialize_field(self.get_value(member_id)?, &member_descriptor.name)?;
                    }
                }
                ExtensibilityKind::Mutable => {
                    let mut mutable_serializer = serializer.serialize_mutable_struct()?;
                    for field_index in 0..self.get_item_count() {
                        let member_id = self.get_member_id_at_index(field_index)?;
                        let member_descriptor = self.get_descriptor(member_id)?;
                        let value = self.get_value(member_id)?;
                        if member_descriptor.is_optional {
                            if let Some(default_value) = &member_descriptor.default_value {
                                if value == default_value {
                                    continue;
                                }
                            }
                        }
                        mutable_serializer.serialize_field(
                            value,
                            member_id,
                            &member_descriptor.name,
                        )?;
                    }
                    mutable_serializer.end()?;
                }
            },
            kind => todo!("Noy yet implemented for {kind:?}"),
        }
        Ok(())
    }
}
