use super::{
    types::{EntityIdPIM, GuidPrefixPIM, LocatorPIM, ProtocolVersionPIM, VendorIdPIM, GUIDPIM},
    RTPSEntity,
};

pub trait RTPSParticipant<
    PSM: GuidPrefixPIM + EntityIdPIM + ProtocolVersionPIM + VendorIdPIM + LocatorPIM + GUIDPIM<PSM>,
>: RTPSEntity<PSM>
{
    fn protocol_version(&self) -> PSM::ProtocolVersionType;
    fn vendor_id(&self) -> PSM::VendorIdType;
    fn default_unicast_locator_list(&self) -> &[PSM::LocatorType];
    fn default_multicast_locator_list(&self) -> &[PSM::LocatorType];
}
