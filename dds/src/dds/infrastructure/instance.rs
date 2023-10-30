use crate::{builtin_topics::BuiltInTopicKey, implementation::rtps::types::Guid};

/// Type for the instance handle representing an Entity
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, derive_more::Constructor)]
pub struct InstanceHandle([u8; 16]);

impl Default for InstanceHandle {
    fn default() -> Self {
        HANDLE_NIL
    }
}

/// Special constant value representing a 'nil' [`InstanceHandle`]
pub const HANDLE_NIL: InstanceHandle = InstanceHandle([0; 16]);

impl AsRef<[u8]> for InstanceHandle {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&[u8]> for InstanceHandle {
    fn from(x: &[u8]) -> Self {
        let handle = if x.len() <= 16 {
            let mut h = [0; 16];
            h[..x.len()].clone_from_slice(x);
            h
        } else {
            <[u8; 16]>::from(md5::compute(x))
        };
        Self(handle)
    }
}

impl From<InstanceHandle> for [u8; 16] {
    fn from(x: InstanceHandle) -> Self {
        x.0
    }
}

impl From<Guid> for InstanceHandle {
    fn from(x: Guid) -> Self {
        InstanceHandle(x.into())
    }
}

impl From<BuiltInTopicKey> for InstanceHandle {
    fn from(x: BuiltInTopicKey) -> Self {
        InstanceHandle(x.value)
    }
}
