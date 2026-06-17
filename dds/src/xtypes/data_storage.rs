use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::{
        dynamic_type::{DynamicData, DynamicDataFactory},
        error::{XTypesError, XTypesResult},
    },
};
use alloc::{string::String, vec::Vec};
use dust_dds::xtypes::dynamic_type::{DynamicType, ExtensibilityKind, TypeDescriptor, TypeKind};

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
    Float128(i128),
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
    SequenceFloat128(Vec<i128>),
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

// impl<T: TypeSupport> DataStorageMapping for T {
//     fn into_storage(self) -> DataStorage {
//         let mut data = DynamicDataFactory::create_data(T::TYPE);
//         self.create_dynamic_sample(&mut data);
//         DataStorage::ComplexValue(data)
//     }

//     fn try_from_storage(mut data_storage: DataStorage) -> XTypesResult<Self> {
//         match &mut data_storage {
//             DataStorage::ComplexValue(x) => Ok(T::create_sample(x)),
//             _ => Err(XTypesError::InvalidType),
//         }
//     }
// }

// impl<T: DataStorageMapping> DataStorageMapping for Option<T> {
//     fn into_storage(self) -> DataStorage {
//         T::into_storage(self.expect("Only options with value are converted. This usually indicated a member annotation #[dust_dds(optional)] is missing."))
//     }

//     fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
//         Ok(Some(T::try_from_storage(data_storage)?))
//     }
// }

impl<T: TypeSupport + ComplexData> DataStorageMapping for Vec<T> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceComplexValue(
            self.into_iter()
                .map(|x| {
                    let mut data = DynamicDataFactory::create_data(T::TYPE);
                    x.create_dynamic_sample(&mut data);
                    data
                })
                .collect(),
        )
    }

    fn try_from_storage(mut data_storage: DataStorage) -> XTypesResult<Self> {
        match &mut data_storage {
            DataStorage::SequenceComplexValue(x) => {
                Ok(x.iter_mut().map(T::create_sample).collect())
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

pub trait ComplexData {}

impl<T> ComplexData for Box<T> {}
impl<T> ComplexData for Option<T> {}

impl<const N: usize, T: TypeSupport + ComplexData> DataStorageMapping for [T; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceComplexValue(
            self.into_iter()
                .map(|x| {
                    let mut data = DynamicDataFactory::create_data(T::TYPE);
                    x.create_dynamic_sample(&mut data);
                    data
                })
                .collect(),
        )
    }

    fn try_from_storage(mut data_storage: DataStorage) -> XTypesResult<Self> {
        match &mut data_storage {
            DataStorage::SequenceComplexValue(x) => {
                Self::try_from(x.iter_mut().map(T::create_sample).collect::<Vec<_>>())
                    .map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<T: ComplexData + TypeSupport> DataStorageMapping for T {
    fn into_storage(self) -> DataStorage {
        let mut data = DynamicDataFactory::create_data(T::TYPE);
        self.create_dynamic_sample(&mut data);
        DataStorage::ComplexValue(data)
    }

    fn try_from_storage(mut data_storage: DataStorage) -> XTypesResult<Self> {
        match &mut data_storage {
            DataStorage::ComplexValue(x) => Ok(T::create_sample(x)),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<T: TypeSupport> TypeSupport for Box<T> {
    const r#TYPE: DynamicType = T::TYPE;

    fn create_sample(src: &mut super::dynamic_type::DynamicData) -> Self {
        Box::new(T::create_sample(src))
    }

    fn create_dynamic_sample(self, data: &mut super::dynamic_type::DynamicData) {
        T::create_dynamic_sample(*self, data);
    }
}

impl<T: TypeSupport> TypeSupport for Option<T> {
    const r#TYPE: DynamicType = T::TYPE;

    fn create_sample(src: &mut super::dynamic_type::DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }

    fn create_dynamic_sample(self, data: &mut super::dynamic_type::DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

impl TypeSupport for &str {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::STRING8,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(_src: &mut DynamicData) -> Self {
        todo!()
    }
    fn create_dynamic_sample(self, _data: &mut DynamicData) {
        todo!()
    }
}

impl TypeSupport for &[u8] {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::STRING8,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(_src: &mut DynamicData) -> Self {
        todo!()
    }
    fn create_dynamic_sample(self, _data: &mut DynamicData) {
        todo!()
    }
}

impl TypeSupport for bool {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::BOOLEAN,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

impl TypeSupport for u8 {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::UINT8,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

impl TypeSupport for i8 {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::INT8,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

impl TypeSupport for u16 {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::UINT16,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorage::UInt16(self));
    }
}

impl TypeSupport for i16 {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::INT16,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

impl TypeSupport for i32 {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::INT32,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

impl TypeSupport for u32 {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::UINT32,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

impl TypeSupport for i64 {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::INT64,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}
impl TypeSupport for u64 {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::UINT64,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

impl TypeSupport for f32 {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::FLOAT32,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}
impl TypeSupport for f64 {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::FLOAT64,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

impl TypeSupport for char {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::CHAR8,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

impl TypeSupport for String {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::STRING8,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

impl<T: TypeSupport, const N: usize> TypeSupport for [T; N]
where
    Self: DataStorageMapping,
{
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::ARRAY,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(N as u32),
            element_type: Some(T::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

impl<T: TypeSupport + ComplexData> TypeSupport for Vec<T> {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(T::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(
            0,
            DataStorage::SequenceComplexValue(
                self.into_iter()
                    .map(|x| {
                        let mut data = DynamicDataFactory::create_data(T::TYPE);
                        x.create_dynamic_sample(&mut data);
                        data
                    })
                    .collect(),
            ),
        );
    }
}

impl TypeSupport for Vec<u8> {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(u8::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}
impl TypeSupport for Vec<u32> {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(u32::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}
impl TypeSupport for Vec<i32> {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(i32::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}
impl TypeSupport for Vec<u16> {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(u16::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

impl TypeSupport for Vec<String> {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: Some(u32::MAX),
            element_type: Some(String::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
    fn create_sample(src: &mut DynamicData) -> Self {
        DataStorageMapping::try_from_storage(src.remove_value(0).expect("Must exist"))
            .expect("Must match")
    }
    fn create_dynamic_sample(self, data: &mut DynamicData) {
        data.set_value(0, DataStorageMapping::into_storage(self));
    }
}

#[test]
fn basic_type_should_create_sample() {
    #[derive(Debug, PartialEq, TypeSupport)]
    pub struct Inner {
        x: u16,
    }
    let mut inner = DynamicDataFactory::create_data(Inner::TYPE);
    inner.set_uint16_value(0, 2).unwrap();

    let v = Inner { x: 2 };
    assert_eq!(v.really_create_dynamic_sample(), inner);
}

#[test]
fn complex_type_should_create_sample() {
    #[derive(Debug, PartialEq, TypeSupport)]
    pub struct Inner {
        x: u16,
    }
    #[derive(Debug, PartialEq, TypeSupport)]
    pub struct Outer {
        y: Inner,
    }

    let v = Outer { y: Inner { x: 2 } };
    let mut inner = DynamicDataFactory::create_data(Inner::TYPE);
    inner.set_uint16_value(0, 2).unwrap();
    let mut outer = DynamicDataFactory::create_data(Outer::TYPE);
    outer.set_complex_value(0, inner).unwrap();

    assert_eq!(v.really_create_dynamic_sample(), outer);
}

#[test]
fn vector_of_basic_type_should_create_sample() {
    #[derive(Debug, PartialEq, TypeSupport)]
    pub struct Inner {
        x: Vec<u16>,
    }
    let mut inner = DynamicDataFactory::create_data(Inner::TYPE);
    inner.set_uint16_values(0, vec![2]).unwrap();

    let v = Inner { x: vec![2] };
    assert_eq!(v.really_create_dynamic_sample(), inner);
}

#[test]
fn vector_of_complex_type_should_create_sample() {
    #[derive(Debug, PartialEq, TypeSupport)]
    pub struct Inner {
        x: u16,
    }
    #[derive(Debug, PartialEq, TypeSupport)]
    pub struct Outer {
        list: Vec<Inner>,
    }

    let v = Outer {
        list: vec![Inner { x: 2 }],
    };
    let mut inner = DynamicDataFactory::create_data(Inner::TYPE);
    inner.set_uint16_value(0, 2).unwrap();
    let mut outer = DynamicDataFactory::create_data(Outer::TYPE);
    outer.set_complex_values(0, vec![inner]).unwrap();

    assert_eq!(v.really_create_dynamic_sample(), outer);
}
