use super::{
    types::{Locator, ProtocolVersionPIM, VendorIdPIM},
    RTPSEntity,
};

pub trait RTPSParticipant<PSM>: RTPSEntity
where
    PSM: ProtocolVersionPIM + VendorIdPIM,
{
    fn protocol_version(&self) -> &PSM::ProtocolVersionType;
    fn vendor_id(&self) -> &PSM::VendorIdType;
    fn default_unicast_locator_list(&self) -> &[Locator];
    fn default_multicast_locator_list(&self) -> &[Locator];
}
