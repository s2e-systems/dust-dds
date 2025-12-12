use crate::transport::types::Guid;
use core::{borrow::Borrow, ops::Index};
use dust_dds_derive::TypeSupport;

/// Special constant value representing a 'nil' [`InstanceHandle`].
pub const HANDLE_NIL: InstanceHandle = InstanceHandle([0; 16]);

/// Type for the instance handle representing an Entity.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, TypeSupport)]
pub struct InstanceHandle([u8; 16]);

impl InstanceHandle {
    /// Constructs a new `InstanceHandle`.
    #[inline]
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

impl From<[u8; 16]> for InstanceHandle {
    #[inline]
    fn from(value: [u8; 16]) -> Self {
        Self(value)
    }
}

impl From<InstanceHandle> for [u8; 16] {
    #[inline]
    fn from(value: InstanceHandle) -> Self {
        value.0
    }
}

impl From<Guid> for InstanceHandle {
    #[inline]
    fn from(value: Guid) -> Self {
        Self(value.into())
    }
}

impl From<InstanceHandle> for Guid {
    #[inline]
    fn from(value: InstanceHandle) -> Self {
        value.0.into()
    }
}

impl From<crate::rtps::behavior_types::InstanceHandle> for InstanceHandle {
    #[inline]
    fn from(value: crate::rtps::behavior_types::InstanceHandle) -> Self {
        Self(value.0)
    }
}

impl From<InstanceHandle> for crate::rtps::behavior_types::InstanceHandle {
    #[inline]
    fn from(value: InstanceHandle) -> Self {
        Self(value.0)
    }
}

impl PartialEq<[u8; 16]> for InstanceHandle {
    #[inline]
    fn eq(&self, other: &[u8; 16]) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<InstanceHandle> for [u8; 16] {
    #[inline]
    fn eq(&self, other: &InstanceHandle) -> bool {
        self.eq(&other.0)
    }
}

impl AsRef<[u8; 16]> for InstanceHandle {
    #[inline]
    fn as_ref(&self) -> &[u8; 16] {
        &self.0
    }
}

impl Borrow<[u8; 16]> for InstanceHandle {
    #[inline]
    fn borrow(&self) -> &[u8; 16] {
        &self.0
    }
}

impl Default for InstanceHandle {
    #[inline]
    fn default() -> Self {
        HANDLE_NIL
    }
}

impl Index<usize> for InstanceHandle {
    type Output = u8;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
