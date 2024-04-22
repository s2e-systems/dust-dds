use super::{
    entity::RtpsEntity,
    types::{Guid, GuidPrefix, Locator, ProtocolVersion, VendorId, ENTITYID_PARTICIPANT},
};

pub struct RtpsParticipant {
    entity: RtpsEntity,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
}

impl RtpsParticipant {
    pub fn new(
        guid_prefix: GuidPrefix,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        metatraffic_unicast_locator_list: Vec<Locator>,
        metatraffic_multicast_locator_list: Vec<Locator>,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
    ) -> Self {
        Self {
            entity: RtpsEntity::new(Guid::new(guid_prefix, ENTITYID_PARTICIPANT)),
            protocol_version,
            vendor_id,
            default_unicast_locator_list,
            default_multicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
        }
    }

    pub fn guid(&self) -> Guid {
        self.entity.guid()
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    pub fn vendor_id(&self) -> VendorId {
        self.vendor_id
    }

    pub fn default_unicast_locator_list(&self) -> &[Locator] {
        self.default_unicast_locator_list.as_slice()
    }

    pub fn default_multicast_locator_list(&self) -> &[Locator] {
        self.default_multicast_locator_list.as_slice()
    }

    pub fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        self.metatraffic_unicast_locator_list.as_ref()
    }

    pub fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        self.metatraffic_multicast_locator_list.as_ref()
    }
}
