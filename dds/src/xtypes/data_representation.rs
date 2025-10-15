use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::{
        dynamic_type::DynamicData, error::XTypesError, serializer::XTypesSerializer,
        xcdr_serializer::serialize_nested,
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
    Sequence(Vec<DataKind>),
    Array(Vec<DataKind>),
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

impl<const N: usize, T: Into<DataKind>> From<[T; N]> for DataKind {
    fn from(value: [T; N]) -> Self {
        Self::Array(value.into_iter().map(T::into).collect())
    }
}

impl<T: Into<DataKind>> From<Vec<T>> for DataKind {
    fn from(value: Vec<T>) -> Self {
        Self::Sequence(value.into_iter().map(T::into).collect())
    }
}

impl From<&[u8]> for DataKind {
    fn from(value: &[u8]) -> Self {
        value.to_vec().into()
    }
}

impl TryFrom<DataKind> for u8 {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::UInt8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataKind> for i8 {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::Int8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataKind> for u16 {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::UInt16(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataKind> for i16 {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::Int16(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataKind> for u32 {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::UInt32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataKind> for i32 {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::Int32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataKind> for u64 {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::UInt64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataKind> for i64 {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::Int64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataKind> for f32 {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::Float32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataKind> for f64 {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::Float64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataKind> for char {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::Char8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataKind> for bool {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::Boolean(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataKind> for String {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::String(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataKind> for DynamicData {
    type Error = XTypesError;

    fn try_from(value: DataKind) -> Result<Self, Self::Error> {
        match value {
            DataKind::ComplexValue(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}
