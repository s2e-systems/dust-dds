use core::ops::{Deref, DerefMut};

use super::{
    types::{Guid, ProtocolVersion, VendorId},
    RtpsEntity,
};

pub struct RtpsParticipant<L> {
    entity: RtpsEntity,
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub default_unicast_locator_list: L,
    pub default_multicast_locator_list: L,
}

impl<L> Deref for RtpsParticipant<L> {
    type Target = RtpsEntity;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<L> DerefMut for RtpsParticipant<L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

impl<L> RtpsParticipant<L> {
    pub fn new(
        guid: Guid,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
        default_unicast_locator_list: L,
        default_multicast_locator_list: L,
    ) -> Self {
        Self {
            entity: RtpsEntity::new(guid),
            protocol_version,
            vendor_id,
            default_unicast_locator_list,
            default_multicast_locator_list,
        }
    }
}
