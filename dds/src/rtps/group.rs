use super::{entity::RtpsEntity, types::Guid};

pub struct RtpsGroup {
    entity: RtpsEntity,
}

impl RtpsGroup {
    pub fn new(guid: Guid) -> Self {
        Self {
            entity: RtpsEntity::new(guid),
        }
    }
}

impl RtpsGroup {
    pub fn guid(&self) -> Guid {
        self.entity.guid()
    }
}
