use core::ops::{Deref, DerefMut};

use super::{types::Guid, RtpsEntity};

pub struct RtpsGroup {
    entity: RtpsEntity,
}

impl Deref for RtpsGroup {
    type Target = RtpsEntity;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl DerefMut for RtpsGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

impl RtpsGroup {
    pub fn new(guid: Guid) -> Self {
        Self {
            entity: RtpsEntity::new(guid),
        }
    }
}
