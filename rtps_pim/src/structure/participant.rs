use super::{
    types::{
        EntityIdType, GUIDType, GuidPrefixType, LocatorType, ProtocolVersionType, VendorIdType,
    },
    RTPSEntity,
};

pub trait RTPSParticipant<
    PSM: GuidPrefixType + EntityIdType + ProtocolVersionType + VendorIdType + LocatorType + GUIDType<PSM>,
>: RTPSEntity<PSM>
{
    fn protocol_version(&self) -> PSM::ProtocolVersion;
    fn vendor_id(&self) -> PSM::VendorId;
    fn default_unicast_locator_list(&self) -> &[PSM::Locator];
    fn default_multicast_locator_list(&self) -> &[PSM::Locator];
}
