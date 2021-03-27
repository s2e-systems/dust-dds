use crate::RtpsPsm;

use super::RTPSEntity;

pub struct RTPSGroup<PSM: RtpsPsm> {
    pub entity: RTPSEntity<PSM>,
}

impl<PSM: RtpsPsm> core::ops::Deref for RTPSGroup<PSM> {
    type Target = RTPSEntity<PSM>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}
