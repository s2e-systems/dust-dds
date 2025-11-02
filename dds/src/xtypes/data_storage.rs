use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::{
        dynamic_type::DynamicData,
        error::{XTypesError, XTypesResult},
    },
};
use alloc::{string::String, vec::Vec};

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
    // Sequence
    SequenceUInt8(Vec<u8>),
    SequenceInt8(Vec<i8>),
    SequenceUInt16(Vec<u16>),
    SequenceInt16(Vec<i16>),
    SequenceInt32(Vec<i32>),
    SequenceUInt32(Vec<u32>),
    SequenceInt64(Vec<i64>),
    SequenceUInt64(Vec<u64>),
    SequenceFloat32(Vec<f32>),
    SequenceFloat64(Vec<f64>),
    SequenceChar8(Vec<char>),
    SequenceBoolean(Vec<bool>),
    SequenceString(Vec<String>),
    SequenceComplexValue(Vec<DynamicData>),
}

pub trait DataStorageMapping: Sized {
    fn into_storage(self) -> DataStorage;

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self>;
}

impl DataStorageMapping for u8 {
    fn into_storage(self) -> DataStorage {
        DataStorage::UInt8(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::UInt8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for i8 {
    fn into_storage(self) -> DataStorage {
        DataStorage::Int8(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Int8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for u16 {
    fn into_storage(self) -> DataStorage {
        DataStorage::UInt16(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::UInt16(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for i16 {
    fn into_storage(self) -> DataStorage {
        DataStorage::Int16(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Int16(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for u32 {
    fn into_storage(self) -> DataStorage {
        DataStorage::UInt32(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::UInt32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for i32 {
    fn into_storage(self) -> DataStorage {
        DataStorage::Int32(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Int32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for u64 {
    fn into_storage(self) -> DataStorage {
        DataStorage::UInt64(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::UInt64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for i64 {
    fn into_storage(self) -> DataStorage {
        DataStorage::Int64(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Int64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for f32 {
    fn into_storage(self) -> DataStorage {
        DataStorage::Float32(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Float32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for f64 {
    fn into_storage(self) -> DataStorage {
        DataStorage::Float64(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Float64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for bool {
    fn into_storage(self) -> DataStorage {
        DataStorage::Boolean(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Boolean(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for char {
    fn into_storage(self) -> DataStorage {
        DataStorage::Char8(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Char8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for String {
    fn into_storage(self) -> DataStorage {
        DataStorage::String(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::String(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for DynamicData {
    fn into_storage(self) -> DataStorage {
        DataStorage::ComplexValue(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::ComplexValue(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

// SequenceChar8(Vec<char>),
// SequenceBoolean(Vec<bool>),
// SequenceString(Vec<String>),
// SequenceComplexValue(Vec<DynamicData>),

impl DataStorageMapping for Vec<u8> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt8(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<i8> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt8(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<u16> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt16(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt16(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<i16> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt16(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt16(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<u32> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt32(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<i32> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt32(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<u64> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt64(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<i64> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt64(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<f32> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceFloat32(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceFloat32(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<f64> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceFloat64(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceFloat64(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<char> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceChar8(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceChar8(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<bool> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceBoolean(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceBoolean(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<String> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceString(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceString(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl DataStorageMapping for Vec<DynamicData> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceComplexValue(self)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceComplexValue(x) => Ok(x),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [u8; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt8(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt8(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [i8; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt8(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt8(x) => Self::try_from(x).map_err(|_| XTypesError::InvalidType),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [u16; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt16(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt16(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [i16; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt16(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt16(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [u32; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt32(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt32(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [i32; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt32(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt32(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [u64; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt64(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt64(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [i64; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceInt64(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceInt64(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [f32; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceFloat32(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceFloat32(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [f64; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceFloat64(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceFloat64(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [char; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceChar8(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceChar8(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [bool; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceBoolean(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceBoolean(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [String; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceString(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceString(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [DynamicData; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceComplexValue(self.to_vec())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceComplexValue(x) => {
                Self::try_from(x).map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<T: TypeSupport> DataStorageMapping for T {
    fn into_storage(self) -> DataStorage {
        DataStorage::ComplexValue(T::create_dynamic_sample(self))
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::ComplexValue(x) => Ok(T::create_sample(x)),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<T: TypeSupport> DataStorageMapping for Vec<T> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceComplexValue(self.into_iter().map(T::create_dynamic_sample).collect())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceComplexValue(x) => {
                Ok(x.into_iter().map(T::create_sample).collect())
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize, T: TypeSupport> DataStorageMapping for [T; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceComplexValue(self.into_iter().map(T::create_dynamic_sample).collect())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceComplexValue(x) => {
                Self::try_from(x.into_iter().map(T::create_sample).collect::<Vec<_>>())
                    .map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}
