use crate::types::{EntityId, GuidPrefix, GUID};

pub struct RTPSEntity<GuidPrefixType: GuidPrefix, EntityIdType: EntityId> {
    pub guid: GUID<GuidPrefixType, EntityIdType>,
}
