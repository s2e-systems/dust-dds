use crate::types::{
    constants::ENTITYID_PARTICIPANT, GuidPrefix, Locator, ProtocolVersion, VendorId, GUID,
};

use super::Entity;

pub struct Participant {
    pub entity: Entity,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
}

impl Participant {
    pub fn new(
        guid_prefix: GuidPrefix,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
    ) -> Self {
        let guid = GUID::new(guid_prefix, ENTITYID_PARTICIPANT);
        let entity = Entity::new(guid);
        Self {
            entity,
            default_unicast_locator_list,
            default_multicast_locator_list,
            protocol_version,
            vendor_id,
        }
    }
}
