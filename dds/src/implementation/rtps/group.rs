use super::{entity::RtpsEntity, types::Guid};

pub struct RtpsGroupImpl {
    entity: RtpsEntity,
}

impl RtpsGroupImpl {
    pub fn new(guid: Guid) -> Self {
        Self {
            entity: RtpsEntity::new(guid),
        }
    }
}

impl RtpsGroupImpl {
    pub fn guid(&self) -> Guid {
        self.entity.guid()
    }
}
