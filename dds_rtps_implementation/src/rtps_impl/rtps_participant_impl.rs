use rust_rtps_pim::structure::{
    entity::RtpsEntityAttributes,
    participant::{RtpsParticipantAttributes, RtpsParticipantConstructor},
    types::{Guid, Locator, ProtocolVersion, VendorId},
};

pub struct RtpsParticipantImpl {
    guid: Guid,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
}

impl RtpsEntityAttributes for RtpsParticipantImpl {
    fn guid(&self) -> &rust_rtps_pim::structure::types::Guid {
        &self.guid
    }
}

impl RtpsParticipantAttributes for RtpsParticipantImpl {
    fn guid(&self) -> Guid {
        self.guid
    }

    fn protocol_version(&self) -> &ProtocolVersion {
        &self.protocol_version
    }

    fn vendor_id(&self) -> &VendorId {
        &self.vendor_id
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
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
    ) -> Self {
        Self {
            guid,
            protocol_version,
            vendor_id,
            default_unicast_locator_list: default_unicast_locator_list.to_vec(),
            default_multicast_locator_list: default_multicast_locator_list.to_vec(),
        }
    }
}