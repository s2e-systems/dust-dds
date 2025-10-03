use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::{
        dynamic_type::{DynamicData, DynamicType, DynamicTypeBuilderFactory, MemberId, TypeKind},
        error::XTypesResult,
    },
};

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

pub trait XTypesBinding {
    fn get_dynamic_type() -> DynamicType;

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()>;
}

impl XTypesBinding for u8 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::UINT8)
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_uint8_value(id, self)
    }
}
impl XTypesBinding for i8 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::INT8)
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_int8_value(id, self)
    }
}

impl XTypesBinding for u16 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::UINT16)
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_uint16_value(id, self)
    }
}

impl XTypesBinding for i16 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::INT16)
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_int16_value(id, self)
    }
}

impl XTypesBinding for u32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::UINT32)
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_uint32_value(id, self)
    }
}

impl XTypesBinding for i32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::INT32)
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_int32_value(id, self)
    }
}

impl XTypesBinding for u64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::UINT64)
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_uint64_value(id, self)
    }
}

impl XTypesBinding for i64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::INT64)
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_int64_value(id, self)
    }
}

impl XTypesBinding for String {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_string_type(u32::MAX).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_string_value(id, self)
    }
}

impl XTypesBinding for bool {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::BOOLEAN)
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_boolean_value(id, self)
    }
}

impl XTypesBinding for f32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::FLOAT32)
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_float32_value(id, self)
    }
}

impl XTypesBinding for f64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::FLOAT64)
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_float64_value(id, self)
    }
}

impl XTypesBinding for char {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TypeKind::CHAR8)
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_char8_value(id, self)
    }
}

impl<const N: usize> XTypesBinding for [u8; N] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_array_type(u8::get_dynamic_type(), vec![N as u32]).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_uint8_values(id, self.to_vec())
    }
}

impl<const N: usize> XTypesBinding for [i16; N] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_array_type(i16::get_dynamic_type(), vec![N as u32])
            .build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_int16_values(id, self.to_vec())
    }
}

impl XTypesBinding for &'_ [u8] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u8::get_dynamic_type(), u32::MAX).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_uint8_values(id, self.to_vec())
    }
}

impl XTypesBinding for Vec<u8> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u8::get_dynamic_type(), u32::MAX).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_uint8_values(id, self)
    }
}

impl XTypesBinding for Vec<u16> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u16::get_dynamic_type(), u32::MAX).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_uint16_values(id, self)
    }
}

impl XTypesBinding for Vec<u32> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u32::get_dynamic_type(), u32::MAX).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_uint32_values(id, self)
    }
}

impl XTypesBinding for Vec<u64> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u64::get_dynamic_type(), u32::MAX).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_uint64_values(id, self)
    }
}

impl XTypesBinding for Vec<i8> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(i8::get_dynamic_type(), u32::MAX).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_int8_values(id, self)
    }
}

impl XTypesBinding for Vec<i16> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(i16::get_dynamic_type(), u32::MAX).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_int16_values(id, self)
    }
}

impl XTypesBinding for Vec<i32> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(i32::get_dynamic_type(), u32::MAX).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_int32_values(id, self)
    }
}

impl XTypesBinding for Vec<i64> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(i64::get_dynamic_type(), u32::MAX).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_int64_values(id, self)
    }
}

impl XTypesBinding for Vec<f32> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(f32::get_dynamic_type(), u32::MAX).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_float32_values(id, self)
    }
}

impl XTypesBinding for Vec<f64> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(f64::get_dynamic_type(), u32::MAX).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_float64_values(id, self)
    }
}

impl XTypesBinding for Vec<String> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(String::get_dynamic_type(), u32::MAX)
            .build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_string_values(id, self)
    }
}

impl<T: TypeSupport> XTypesBinding for T {
    fn get_dynamic_type() -> DynamicType {
        T::get_type()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_complex_value(id, self.create_dynamic_sample())
    }
}

impl<T: TypeSupport, const N: usize> XTypesBinding for [T; N] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_array_type(T::get_type(), vec![N as u32]).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_complex_values(
            id,
            self.into_iter()
                .map(TypeSupport::create_dynamic_sample)
                .collect(),
        )
    }
}

impl<T: TypeSupport> XTypesBinding for Vec<T> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(T::get_type(), u32::MAX).build()
    }

    fn insert_value(self, dynamic_data: &mut DynamicData, id: MemberId) -> XTypesResult<()> {
        dynamic_data.set_complex_values(
            id,
            self.into_iter()
                .map(TypeSupport::create_dynamic_sample)
                .collect(),
        )
    }
}
