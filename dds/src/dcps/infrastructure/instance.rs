use crate::infrastructure::{
    error::DdsResult,
    type_support::{DdsDeserialize, DdsSerialize},
};

use crate::xtypes::{
    deserialize::XTypesDeserialize, serialize::XTypesSerialize, xcdr_serializer::Xcdr1BeSerializer,
};
use alloc::vec::Vec;

/// Type for the instance handle representing an Entity
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    Hash,
    PartialOrd,
    Ord,
    XTypesSerialize,
    XTypesDeserialize,
    DdsSerialize,
    DdsDeserialize,
)]
pub struct InstanceHandle([u8; 16]);

impl InstanceHandle {
    /// InstanceHandle constructor
    pub const fn new(bytes: [u8; 16]) -> Self {
        InstanceHandle(bytes)
    }

    /// Construct InstanceHandle from key
    pub fn try_from_key(foo_key: &impl XTypesSerialize) -> DdsResult<Self> {
        let mut serialized_key = Vec::new();
        let mut serializer = Xcdr1BeSerializer::new(&mut serialized_key);
        XTypesSerialize::serialize(foo_key, &mut serializer)?;
        let handle = if serialized_key.len() <= 16 {
            let mut h = [0; 16];
            h[..serialized_key.len()].clone_from_slice(serialized_key.as_slice());
            h
        } else {
            <[u8; 16]>::from(md5::compute(serialized_key.as_slice()))
        };
        Ok(Self(handle))
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

/// Special constant value representing a 'nil' [`InstanceHandle`]
pub const HANDLE_NIL: InstanceHandle = InstanceHandle([0; 16]);

impl From<InstanceHandle> for [u8; 16] {
    fn from(x: InstanceHandle) -> Self {
        x.0
    }
}
