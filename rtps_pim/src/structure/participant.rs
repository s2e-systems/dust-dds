use crate::types::{Locator, ProtocolVersion, VendorId};

use super::RTPSEntity;

pub trait RTPSParticipant: RTPSEntity {
    type Locator: Locator;
    type ProtocolVersion: ProtocolVersion;
    type VendorId: VendorId;

    fn default_unicast_locator_list(&self) -> &[Self::Locator];
    fn default_multicast_locator_list(&self) -> &[Self::Locator];
    fn protocol_version(&self) -> Self::ProtocolVersion;
    fn vendor_id(&self) -> Self::VendorId;
}
