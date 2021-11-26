use super::{entity::RtpsEntity, types::Guid};

pub struct RtpsGroup {
    pub entity: RtpsEntity,
}

impl RtpsGroup {
    pub fn new(guid: Guid) -> Self {
        Self {
            entity: RtpsEntity::new(guid),
        }
    }
}
