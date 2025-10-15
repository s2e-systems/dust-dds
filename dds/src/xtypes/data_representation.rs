use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::{
        bytes::Bytes, dynamic_type::{DynamicData, ExtensibilityKind, TypeKind}, serializer::{
            SerializeAppendableStruct, SerializeFinalStruct, SerializeMutableStruct,
            XTypesSerializer,
        }, xcdr_serializer::serialize_nested
    },
};
use alloc::{string::String, vec::Vec};

impl DynamicData {
    pub fn serialize(
        &self,
        serializer: impl XTypesSerializer,
    ) -> Result<(), super::error::XTypesError> {
        // todo header CDR type
        serialize_nested(self, serializer)
        // sentinel ?
    }
}

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

impl<T: TypeSupport> From<T> for DataKind {
    fn from(value: T) -> Self {
        Self::ComplexValue(value.create_dynamic_sample())
    }
}

impl<const N: usize> From<[u8; N]> for DataKind {
    fn from(value: [u8; N]) -> Self {
        Self::UInt8Array(value.to_vec())
    }
}

impl<const N: usize> From<[i8; N]> for DataKind {
    fn from(value: [i8; N]) -> Self {
        Self::Int8Array(value.to_vec())
    }
}

impl<const N: usize> From<[u16; N]> for DataKind {
    fn from(value: [u16; N]) -> Self {
        Self::UInt16Array(value.to_vec())
    }
}

impl<const N: usize> From<[i16; N]> for DataKind {
    fn from(value: [i16; N]) -> Self {
        Self::Int16Array(value.to_vec())
    }
}

impl<const N: usize> From<[u32; N]> for DataKind {
    fn from(value: [u32; N]) -> Self {
        Self::UInt32Array(value.to_vec())
    }
}

impl<const N: usize> From<[i32; N]> for DataKind {
    fn from(value: [i32; N]) -> Self {
        Self::Int32Array(value.to_vec())
    }
}

impl<const N: usize> From<[u64; N]> for DataKind {
    fn from(value: [u64; N]) -> Self {
        Self::UInt64Array(value.to_vec())
    }
}

impl<const N: usize> From<[i64; N]> for DataKind {
    fn from(value: [i64; N]) -> Self {
        Self::Int64Array(value.to_vec())
    }
}

impl<const N: usize> From<[f32; N]> for DataKind {
    fn from(value: [f32; N]) -> Self {
        Self::Float32Array(value.to_vec())
    }
}

impl<const N: usize> From<[f64; N]> for DataKind {
    fn from(value: [f64; N]) -> Self {
        Self::Float64Array(value.to_vec())
    }
}

impl<const N: usize> From<[char; N]> for DataKind {
    fn from(value: [char; N]) -> Self {
        Self::Char8Array(value.to_vec())
    }
}

impl<const N: usize> From<[bool; N]> for DataKind {
    fn from(value: [bool; N]) -> Self {
        Self::BooleanArray(value.to_vec())
    }
}

impl<const N: usize> From<[String; N]> for DataKind {
    fn from(value: [String; N]) -> Self {
        Self::StringArray(value.to_vec())
    }
}

impl<const N: usize, T: TypeSupport> From<[T; N]> for DataKind {
    fn from(value: [T; N]) -> Self {
        Self::ComplexValueArray(
            value
                .into_iter()
                .map(TypeSupport::create_dynamic_sample)
                .collect(),
        )
    }
}

impl From<Vec<u8>> for DataKind {
    fn from(value: Vec<u8>) -> Self {
        Self::UInt8List(value)
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
        Self::UInt8List(value.to_vec())
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
