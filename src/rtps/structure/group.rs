use crate::rtps::types::GUID;
use crate::rtps::structure::RtpsEntity;

pub struct RtpsGroup {
    pub entity: RtpsEntity,
}

impl RtpsGroup {
    pub fn new(guid: GUID) -> Self {
        let entity = RtpsEntity::new(guid);
        Self {
            entity
        }
    }
}