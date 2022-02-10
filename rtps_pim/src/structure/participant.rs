use super::{types::{Locator, ProtocolVersion, VendorId, Guid}, entity::RtpsEntityAttributes};

pub trait RtpsParticipantAttributes : RtpsEntityAttributes {
    fn protocol_version(&self) -> &ProtocolVersion;
    fn vendor_id(&self) -> &VendorId;
    fn default_unicast_locator_list(&self) -> &[Locator];
    fn default_multicast_locator_list(&self) -> &[Locator];
}

pub trait RtpsParticipantConstructor {
    fn new(
        guid: Guid,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
    ) -> Self;
}