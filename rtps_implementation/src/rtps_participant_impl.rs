use rust_rtps_pim::structure::{
    entity::RtpsEntityAttributes,
    participant::{RtpsParticipantAttributes, RtpsParticipantConstructor},
    types::{Guid, Locator, ProtocolVersion, VendorId},
};

use super::rtps_entity_impl::RtpsEntityImpl;

pub struct RtpsParticipantImpl {
    pub entity: RtpsEntityImpl,
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
}

impl RtpsEntityAttributes for RtpsParticipantImpl {
    fn guid(&self) -> Guid {
        self.entity.guid
    }
}

impl RtpsParticipantAttributes for RtpsParticipantImpl {
    fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    fn vendor_id(&self) -> VendorId {
        self.vendor_id
    }

    fn default_unicast_locator_list(&self) -> &[Locator] {
        self.default_unicast_locator_list.as_slice()
    }

    fn default_multicast_locator_list(&self) -> &[Locator] {
        self.default_multicast_locator_list.as_slice()
    }
}

impl RtpsParticipantConstructor for RtpsParticipantImpl {
    fn new(
        guid: Guid,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
    ) -> Self {
        Self {
            entity: RtpsEntityImpl{ guid },
            protocol_version,
            vendor_id,
            default_unicast_locator_list: default_unicast_locator_list.to_vec(),
            default_multicast_locator_list: default_multicast_locator_list.to_vec(),
        }
    }
}