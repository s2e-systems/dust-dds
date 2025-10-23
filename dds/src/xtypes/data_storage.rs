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
    Sequence(Vec<DataStorage>),
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

impl<T: DataStorageMapping> DataStorageMapping for Vec<T> {
    fn into_storage(self) -> DataStorage {
        DataStorage::Sequence(
            self.into_iter()
                .map(DataStorageMapping::into_storage)
                .collect(),
        )
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Sequence(x) => Ok(x
                .into_iter()
                .map(|x| DataStorageMapping::try_from_storage(x).unwrap())
                .collect()),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<T: DataStorageMapping, const N: usize> DataStorageMapping for [T; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::Sequence(
            self.into_iter()
                .map(DataStorageMapping::into_storage)
                .collect(),
        )
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::Sequence(x) => Self::try_from(
                x.into_iter()
                    .map(|x| DataStorageMapping::try_from_storage(x).unwrap())
                    .collect::<Vec<T>>(),
            )
            .map_err(|_| XTypesError::InvalidType),
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
