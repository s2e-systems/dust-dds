use crate::types::{Locator, ProtocolVersion, VendorId};

use super::RTPSEntity;

pub trait RTPSParticipant: RTPSEntity {
    fn default_unicast_locator_list(&self) -> &[Locator];
    fn default_multicast_locator_list(&self) -> &[Locator];
    fn protocol_version(&self) -> ProtocolVersion;
    fn vendor_id(&self) -> VendorId;
}
