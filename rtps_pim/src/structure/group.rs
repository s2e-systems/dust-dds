use crate::structure;

use super::RTPSEntity;

pub struct RTPSGroup<PSM: structure::Types> {
    pub entity: RTPSEntity<PSM>,
}

impl<PSM: structure::Types> core::ops::Deref for RTPSGroup<PSM> {
    type Target = RTPSEntity<PSM>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}
