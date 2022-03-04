use super::types::{Guid, Locator, ProtocolVersion, VendorId};

pub trait RtpsParticipantAttributes {
    fn default_unicast_locator_list(&self) -> &[Locator];
    fn default_multicast_locator_list(&self) -> &[Locator];
    fn protocol_version(&self) -> ProtocolVersion;
    fn vendor_id(&self) -> VendorId;
}

pub trait RtpsParticipantConstructor {
    fn new(
        guid: Guid,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
    ) -> Self;
}
