use alloc::vec::Vec;
use dust_dds_derive::TypeSupport;

use crate::xtypes::{
    data_storage::{DataStorage, DataStorageMapping},
    dynamic_type::{DynamicType, ExtensibilityKind, TypeDescriptor, TypeKind},
    error::{XTypesError, XTypesResult},
    type_support::Type,
};

// #[derive(Debug, PartialEq, Eq, Clone, Copy, TypeSupport)]
// pub struct Bytes<'a>(pub &'a [u8]);

#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
pub struct ByteBuf(pub Vec<u8>);

/// Represents an IDL octet / XTypes byte.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Byte(pub u8);

impl core::ops::Deref for Byte {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Byte {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u8> for Byte {
    fn from(v: u8) -> Self {
        Self(v)
    }
}

impl From<Byte> for u8 {
    fn from(v: Byte) -> Self {
        v.0
    }
}

impl AsRef<u8> for Byte {
    fn as_ref(&self) -> &u8 {
        &self.0
    }
}

impl AsMut<u8> for Byte {
    fn as_mut(&mut self) -> &mut u8 {
        &mut self.0
    }
}

impl core::borrow::Borrow<u8> for Byte {
    fn borrow(&self) -> &u8 {
        &self.0
    }
}

impl PartialEq<u8> for Byte {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Byte> for u8 {
    fn eq(&self, other: &Byte) -> bool {
        *self == other.0
    }
}

impl core::fmt::Display for Byte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}

impl core::fmt::LowerHex for Byte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::LowerHex::fmt(&self.0, f)
    }
}

impl core::fmt::UpperHex for Byte {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::UpperHex::fmt(&self.0, f)
    }
}

impl Type for Byte {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::BYTE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: &[],
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
            is_autoid_hash: false,
        },
        member_list: &[],
    };
}

impl DataStorageMapping for Byte {
    fn into_storage(self) -> DataStorage {
        DataStorage::UInt8(self.0)
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::UInt8(x) => Ok(Self(x)),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl Type for Vec<Byte> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::SEQUENCE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: &[0],
            element_type: Some(Byte::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
            is_autoid_hash: false,
        },
        member_list: &[],
    };
}

impl DataStorageMapping for Vec<Byte> {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt8(self.into_iter().map(|b| b.0).collect())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt8(x) => Ok(x.into_iter().map(Byte).collect()),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl<const N: usize> DataStorageMapping for [Byte; N] {
    fn into_storage(self) -> DataStorage {
        DataStorage::SequenceUInt8(self.into_iter().map(|b| b.0).collect())
    }

    fn try_from_storage(data_storage: DataStorage) -> XTypesResult<Self> {
        match data_storage {
            DataStorage::SequenceUInt8(x) => {
                let vec_byte: Vec<Byte> = x.into_iter().map(Byte).collect();
                vec_byte.try_into().map_err(|_| XTypesError::InvalidType)
            }
            _ => Err(XTypesError::InvalidType),
        }
    }
}

impl Type for &[Byte] {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &TypeDescriptor {
            kind: TypeKind::ARRAY,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: &[u32::MAX],
            element_type: Some(Byte::TYPE),
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: false,
            is_autoid_hash: false,
        },
        member_list: &[],
    };
}
