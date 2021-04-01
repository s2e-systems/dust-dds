use crate::structure;

use super::RTPSEntity;

pub struct RTPSEndpoint<PSM: structure::Types> {
    pub entity: RTPSEntity<PSM>,
    pub topic_kind: PSM::TopicKind,
    pub reliability_level: PSM::ReliabilityKind,
    pub unicast_locator_list: PSM::LocatorVector,
    pub multicast_locator_list: PSM::LocatorVector,
}

impl<PSM: structure::Types> core::ops::Deref for RTPSEndpoint<PSM> {
    type Target = RTPSEntity<PSM>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}
