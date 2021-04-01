use crate::structure;

use super::RTPSEntity;

pub struct RTPSParticipant<PSM: structure::Types> {
    pub entity: RTPSEntity<PSM>,
    pub protocol_version: PSM::ProtocolVersion,
    pub vendor_id: PSM::VendorId,
    pub default_unicast_locator_list: PSM::LocatorVector,
    pub default_multicast_locator_list: PSM::LocatorVector,
}

impl<PSM: structure::Types> core::ops::Deref for RTPSParticipant<PSM> {
    type Target = RTPSEntity<PSM>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}
