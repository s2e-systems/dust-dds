use std::convert::TryFrom;

use crate::{
    implementation::rtps::types::{EntityId, Guid, GuidPrefix},
    topic_definition::type_support::DdsSerializedKey,
};

/// Type for the instance handle representing an Entity
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct InstanceHandle([u8; 16]);

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

impl From<InstanceHandle> for Guid {
    fn from(x: InstanceHandle) -> Self {
        let prefix = GuidPrefix::from(<[u8; 12]>::try_from(&x.0[0..12]).expect("Invalid length"));
        let entity_id = EntityId::new(
            <[u8; 3]>::try_from(&x.0[12..15]).expect("Invalid length"),
            x.0[15].into(),
        );
        Guid::new(prefix, entity_id)
    }
}
