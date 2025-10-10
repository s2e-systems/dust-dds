use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::{
        dynamic_type::{DynamicData, DynamicType, DynamicTypeBuilderFactory, TypeKind},
        serialize::{SerializeCollection, XTypesSerialize},
        serializer::SerializeFinalStruct,
    },
};
use alloc::{string::String, vec, vec::Vec};

use super::serialize::XTypesSerializer;

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
    UInt8Array(Vec<u8>),
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
}

impl DataKind {
    pub fn serialize<T>(&self, serializer: &mut T) -> Result<(), super::error::XTypesError>
    where
        for<'a> &'a mut T: XTypesSerializer,
    {
        match self {
            DataKind::UInt8(v) => serializer.serialize_uint8(*v),
            DataKind::Int8(_) => todo!(),
            DataKind::UInt16(v) => serializer.serialize_uint16(*v),
            DataKind::Int16(v) => serializer.serialize_int16(*v),
            DataKind::Int32(v) => serializer.serialize_int32(*v),
            DataKind::UInt32(v) => serializer.serialize_uint32(*v),
            DataKind::Int64(_) => todo!(),
            DataKind::UInt64(_) => todo!(),
            DataKind::Float32(_) => todo!(),
            DataKind::Float64(_) => todo!(),
            DataKind::Char8(_) => todo!(),
            DataKind::Boolean(v) => serializer.serialize_boolean(*v),
            DataKind::String(v) => serializer.serialize_string(v),
            DataKind::ComplexValue(dynamic_data) => dynamic_data.serialize_nested(serializer),
            DataKind::UInt8Array(items) => serializer.serialize_byte_array(items),
            DataKind::Int8List(items) => todo!(),
            DataKind::UInt16List(items) => todo!(),
            DataKind::Int16List(items) => todo!(),
            DataKind::Int32List(items) => todo!(),
            DataKind::UInt32List(items) => todo!(),
            DataKind::Int64List(items) => todo!(),
            DataKind::UInt64List(items) => todo!(),
            DataKind::Float32List(items) => todo!(),
            DataKind::Float64List(items) => todo!(),
            DataKind::Char8List(items) => todo!(),
            DataKind::BooleanList(items) => todo!(),
            DataKind::StringList(items) => serializer.serialize_string_list(items),
            DataKind::ComplexValueList(dynamic_datas) => {
                serializer.serialize_sequence(dynamic_datas)
            }
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

pub trait XTypesBinding {
    fn get_dynamic_type() -> DynamicType;
}

impl XTypesBinding for u8 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::UINT8)
    }
}
impl XTypesBinding for i8 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::INT8)
    }
}

impl XTypesBinding for u16 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::UINT16)
    }
}

impl XTypesBinding for i16 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::INT16)
    }
}

impl XTypesBinding for u32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::UINT32)
    }
}

impl XTypesBinding for i32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::INT32)
    }
}

impl XTypesBinding for u64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::UINT64)
    }
}

impl XTypesBinding for i64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::INT64)
    }
}

impl XTypesBinding for String {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_string_type(u32::MAX).build()
    }
}

impl XTypesBinding for bool {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::BOOLEAN)
    }
}

impl XTypesBinding for f32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::FLOAT32)
    }
}

impl XTypesBinding for f64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::FLOAT64)
    }
}

impl XTypesBinding for char {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::CHAR8)
    }
}

impl<const N: usize> XTypesBinding for [u8; N] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_array_type(u8::get_dynamic_type(), vec![N as u32]).build()
    }
}

impl<const N: usize> XTypesBinding for [i16; N] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_array_type(i16::get_dynamic_type(), vec![N as u32])
            .build()
    }
}

impl XTypesBinding for &'_ [u8] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u8::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<u8> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u8::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<u16> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u16::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<u32> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u32::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<u64> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u64::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<i8> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(i8::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<i16> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(i16::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<i32> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(i32::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<i64> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(i64::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<f32> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(f32::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<f64> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(f64::get_dynamic_type(), u32::MAX).build()
    }
}

impl XTypesBinding for Vec<String> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(String::get_dynamic_type(), u32::MAX)
            .build()
    }
}

impl<T: TypeSupport> XTypesBinding for T {
    fn get_dynamic_type() -> DynamicType {
        T::get_type()
    }
}

// impl<T: TypeSupport, const N: usize> XTypesBinding for [T; N] {
//     fn get_dynamic_type() -> DynamicType {
//         DynamicTypeBuilderFactory::create_array_type(T::get_type(), vec![N as u32]).build()
//     }
// }

impl<T: TypeSupport> XTypesBinding for Vec<T> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(T::get_type(), u32::MAX).build()
    }
}

impl<T: XTypesBinding> XTypesBinding for Option<T> {
    fn get_dynamic_type() -> DynamicType {
        T::get_dynamic_type()
    }
}
