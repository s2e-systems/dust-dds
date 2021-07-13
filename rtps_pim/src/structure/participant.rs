use super::types::{Locator, ProtocolVersion, VendorId};

pub trait RTPSParticipant {
    fn protocol_version(&self) -> &ProtocolVersion;
    fn vendor_id(&self) -> &VendorId;
    fn default_unicast_locator_list(&self) -> &[Locator];
    fn default_multicast_locator_list(&self) -> &[Locator];
}
