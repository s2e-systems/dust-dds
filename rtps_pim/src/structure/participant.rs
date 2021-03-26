use crate::RtpsPim;

use super::RTPSEntity;

pub struct RTPSParticipant<PSM: RtpsPim> {
    pub entity: RTPSEntity<PSM>,
    pub protocol_version: PSM::ProtocolVersion,
    pub vendor_id: PSM::VendorId,
    pub default_unicast_locator_list: PSM::LocatorList,
    pub default_multicast_locator_list: PSM::LocatorList,
}

impl<PSM: RtpsPim> core::ops::Deref for RTPSParticipant<PSM> {
    type Target = RTPSEntity<PSM>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}
