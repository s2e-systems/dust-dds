use crate::structure::RtpsEntity;

pub struct RtpsGroup {
    pub entity: RtpsEntity,
}

impl RtpsGroup {
    pub fn new(entity: RtpsEntity) -> Self {
        Self {
            entity
        }
    }
}