use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::{dynamic_type::DynamicData, error::XTypesError},
};

#[derive(Debug, Clone, PartialEq)]
pub enum DataStorage {
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
    Sequence(Vec<DataStorage>),
}

impl From<u8> for DataStorage {
    fn from(value: u8) -> Self {
        Self::UInt8(value)
    }
}

impl From<i8> for DataStorage {
    fn from(value: i8) -> Self {
        Self::Int8(value)
    }
}

impl From<u16> for DataStorage {
    fn from(value: u16) -> Self {
        Self::UInt16(value)
    }
}

impl From<i16> for DataStorage {
    fn from(value: i16) -> Self {
        Self::Int16(value)
    }
}

impl From<u32> for DataStorage {
    fn from(value: u32) -> Self {
        Self::UInt32(value)
    }
}

impl From<i32> for DataStorage {
    fn from(value: i32) -> Self {
        Self::Int32(value)
    }
}

impl From<u64> for DataStorage {
    fn from(value: u64) -> Self {
        Self::UInt64(value)
    }
}

impl From<i64> for DataStorage {
    fn from(value: i64) -> Self {
        Self::Int64(value)
    }
}

impl From<f32> for DataStorage {
    fn from(value: f32) -> Self {
        Self::Float32(value)
    }
}

impl From<f64> for DataStorage {
    fn from(value: f64) -> Self {
        Self::Float64(value)
    }
}

impl From<char> for DataStorage {
    fn from(value: char) -> Self {
        Self::Char8(value)
    }
}

impl From<bool> for DataStorage {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<String> for DataStorage {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl<T: TypeSupport> From<T> for DataStorage {
    fn from(value: T) -> Self {
        Self::ComplexValue(value.create_dynamic_sample())
    }
}

impl<const N: usize, T: Into<DataStorage>> From<[T; N]> for DataStorage {
    fn from(value: [T; N]) -> Self {
        Self::Sequence(value.into_iter().map(T::into).collect())
    }
}

impl<T: Into<DataStorage>> From<Vec<T>> for DataStorage {
    fn from(value: Vec<T>) -> Self {
        Self::Sequence(value.into_iter().map(T::into).collect())
    }
}

impl From<&[u8]> for DataStorage {
    fn from(value: &[u8]) -> Self {
        value.to_vec().into()
    }
}

impl From<&str> for DataStorage {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl TryFrom<DataStorage> for u8 {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::UInt8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataStorage> for i8 {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::Int8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataStorage> for u16 {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::UInt16(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataStorage> for i16 {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::Int16(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataStorage> for u32 {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::UInt32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataStorage> for i32 {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::Int32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataStorage> for u64 {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::UInt64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataStorage> for i64 {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::Int64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataStorage> for f32 {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::Float32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataStorage> for f64 {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::Float64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataStorage> for char {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::Char8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataStorage> for bool {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::Boolean(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataStorage> for String {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::String(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl TryFrom<DataStorage> for DynamicData {
    type Error = XTypesError;

    fn try_from(value: DataStorage) -> Result<Self, Self::Error> {
        match value {
            DataStorage::ComplexValue(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}
