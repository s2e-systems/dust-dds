use super::types::{Guid, ProtocolVersion, VendorId};

pub struct RtpsParticipant<L> {
    // Attributes from Entity:
    pub guid: Guid,
    // Attributes:
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub default_unicast_locator_list: L,
    pub default_multicast_locator_list: L,
}