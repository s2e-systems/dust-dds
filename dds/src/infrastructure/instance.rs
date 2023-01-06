use crate::{implementation::rtps::types::Guid, topic_definition::type_support::DdsSerializedKey};

/// Type for the instance handle representing an Entity
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct InstanceHandle([u8; 16]);

impl Default for InstanceHandle {
    fn default() -> Self {
        HANDLE_NIL
    }
}

/// Special constant value representing a 'nil' [`InstanceHandle`]
pub const HANDLE_NIL: InstanceHandle = InstanceHandle([0; 16]);

impl From<DdsSerializedKey> for InstanceHandle {
    fn from(x: DdsSerializedKey) -> Self {
        let data = x.as_ref();
        let handle = if data.len() <= 16 {
            let mut h = [0; 16];
            h[..data.len()].clone_from_slice(data);
            h
        } else {
            <[u8; 16]>::from(md5::compute(data))
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
