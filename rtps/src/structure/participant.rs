use crate::types::{Locator, ProtocolVersion, VendorId};

use super::Entity;

pub trait Participant: Entity {
    fn default_unicast_locator_list(&self) -> &[Locator];
    fn default_multicast_locator_list(&self) -> &[Locator];
    fn protocol_version(&self) -> ProtocolVersion;
    fn vendor_id(&self) -> VendorId;
}
