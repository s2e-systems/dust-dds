use core::ops::Index;
use dust_dds_derive::TypeSupport;

/// Type for the instance handle representing an Entity
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, TypeSupport)]
pub struct InstanceHandle([u8; 16]);

impl InstanceHandle {
    /// InstanceHandle constructor
    pub const fn new(bytes: [u8; 16]) -> Self {
        InstanceHandle(bytes)
    }
}

impl From<crate::rtps::behavior_types::InstanceHandle> for InstanceHandle {
    fn from(value: crate::rtps::behavior_types::InstanceHandle) -> Self {
        Self(value.0)
    }
}

impl From<InstanceHandle> for crate::rtps::behavior_types::InstanceHandle {
    fn from(value: InstanceHandle) -> Self {
        crate::rtps::behavior_types::InstanceHandle(value.0)
    }
}

impl AsRef<[u8; 16]> for InstanceHandle {
    fn as_ref(&self) -> &[u8; 16] {
        &self.0
    }
}

impl Default for InstanceHandle {
    fn default() -> Self {
        HANDLE_NIL
    }
}

impl Index<usize> for InstanceHandle {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

/// Special constant value representing a 'nil' [`InstanceHandle`]
pub const HANDLE_NIL: InstanceHandle = InstanceHandle([0; 16]);

impl From<InstanceHandle> for [u8; 16] {
    fn from(x: InstanceHandle) -> Self {
        x.0
    }
}
