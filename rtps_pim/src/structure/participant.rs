use super::{types::Locator, RTPSEntity};

pub trait RTPSParticipant: RTPSEntity {
    type ProtocolVersionType;
    type VendorIdType;

    fn protocol_version(&self) -> &Self::ProtocolVersionType;
    fn vendor_id(&self) -> &Self::VendorIdType;
    fn default_unicast_locator_list(&self) -> &[Locator];
    fn default_multicast_locator_list(&self) -> &[Locator];
}
