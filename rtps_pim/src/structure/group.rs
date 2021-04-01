use crate::RtpsPim;

use super::RTPSEntity;

pub struct RTPSGroup<PSM: RtpsPim> {
    pub entity: RTPSEntity<PSM>,
}

impl<PSM: RtpsPim> core::ops::Deref for RTPSGroup<PSM> {
    type Target = RTPSEntity<PSM>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}
