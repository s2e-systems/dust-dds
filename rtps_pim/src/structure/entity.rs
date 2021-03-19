use crate::types::{EntityId, GuidPrefix, GUID};

pub trait RTPSGUID: GUID {
    type GuidPrefix: GuidPrefix;
    type EntityId: EntityId;

    fn prefix(&self) -> &Self::GuidPrefix;
    fn entity_id(&self) -> &Self::EntityId;
}

pub trait RTPSEntity {
    type GUID: RTPSGUID;
    fn guid(&self) -> &Self::GUID;
}
