use crate::types::{EntityId, GuidPrefix, Locator, ProtocolVersion, VendorId};

use super::RTPSEntity;

pub struct RTPSParticipant<
    GuidPrefixType: GuidPrefix,
    EntityIdType: EntityId,
    ProtocolVersionType: ProtocolVersion,
    VendorIdType: VendorId,
    LocatorType: Locator,
    LocatorListType: IntoIterator<Item = LocatorType>,
> {
    pub entity: RTPSEntity<GuidPrefixType, EntityIdType>,
    pub protocol_version: ProtocolVersionType,
    pub vendor_id: VendorIdType,
    pub default_unicast_locator_list: LocatorListType,
    pub default_multicast_locator_list: LocatorListType,
}

impl<
        GuidPrefixType: GuidPrefix,
        EntityIdType: EntityId,
        ProtocolVersionType: ProtocolVersion,
        VendorIdType: VendorId,
        LocatorType: Locator,
        LocatorListType: IntoIterator<Item = LocatorType>,
    > core::ops::Deref
    for RTPSParticipant<
        GuidPrefixType,
        EntityIdType,
        ProtocolVersionType,
        VendorIdType,
        LocatorType,
        LocatorListType,
    >
{
    type Target = RTPSEntity<GuidPrefixType, EntityIdType>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}
