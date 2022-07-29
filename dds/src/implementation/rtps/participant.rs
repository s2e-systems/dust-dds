use dds_transport::types::Locator;

use super::{
    entity::RtpsEntityImpl,
    types::{Guid, ProtocolVersion, VendorId},
};

pub struct RtpsParticipantImpl {
    entity: RtpsEntityImpl,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
}

impl RtpsParticipantImpl {
    pub fn guid(&self) -> Guid {
        self.entity.guid()
    }
}

impl RtpsParticipantImpl {
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
}

impl RtpsParticipantImpl {
    pub fn new(
        guid: Guid,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
    ) -> Self {
        Self {
            entity: RtpsEntityImpl::new(guid),
            protocol_version,
            vendor_id,
            default_unicast_locator_list: default_unicast_locator_list.to_vec(),
            default_multicast_locator_list: default_multicast_locator_list.to_vec(),
        }
    }
}
