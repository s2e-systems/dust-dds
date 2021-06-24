use super::{
    types::{LocatorPIM, ProtocolVersionPIM, VendorIdPIM},
    RTPSEntity,
};

pub trait RTPSParticipant<PSM>: RTPSEntity
where
    PSM: ProtocolVersionPIM + VendorIdPIM + LocatorPIM,
{
    fn protocol_version(&self) -> &PSM::ProtocolVersionType;
    fn vendor_id(&self) -> &PSM::VendorIdType;
    fn default_unicast_locator_list(&self) -> &[PSM::LocatorType];
    fn default_multicast_locator_list(&self) -> &[PSM::LocatorType];
}
