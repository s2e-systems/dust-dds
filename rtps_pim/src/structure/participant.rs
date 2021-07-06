use super::{
    types::{Locator, VendorId},
    RTPSEntity,
};

pub trait RTPSParticipant: RTPSEntity {
    type ProtocolVersionType;
    fn protocol_version(&self) -> &Self::ProtocolVersionType;
    fn vendor_id(&self) -> &VendorId;
    fn default_unicast_locator_list(&self) -> &[Locator];
    fn default_multicast_locator_list(&self) -> &[Locator];
}
