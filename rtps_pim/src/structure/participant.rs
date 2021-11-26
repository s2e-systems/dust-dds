use super::{
    entity::RtpsEntity,
    types::{Guid, ProtocolVersion, VendorId},
};

pub struct RtpsParticipant<L> {
    pub entity: RtpsEntity,
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub default_unicast_locator_list: L,
    pub default_multicast_locator_list: L,
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
