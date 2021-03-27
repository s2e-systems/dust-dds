use crate::RtpsPsm;

use super::RTPSEntity;

pub struct RTPSEndpoint<PSM: RtpsPsm> {
    pub entity: RTPSEntity<PSM>,
    pub topic_kind: PSM::TopicKind,
    pub reliability_level: PSM::ReliabilityKind,
    pub unicast_locator_list: PSM::LocatorList,
    pub multicast_locator_list: PSM::LocatorList,
}

impl<PSM: RtpsPsm> core::ops::Deref for RTPSEndpoint<PSM> {
    type Target = RTPSEntity<PSM>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}
