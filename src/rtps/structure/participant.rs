use crate::rtps::types::{ProtocolVersion, VendorId, GuidPrefix, GUID};
use crate::rtps::types::constants::ENTITYID_PARTICIPANT;
use crate::rtps::structure::Entity;

use crate::types::DomainId;

pub struct Participant {
    pub entity: Entity,
    pub domain_id: DomainId,
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
}

impl Participant {
    pub fn new(
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
    ) -> Self {
        let guid = GUID::new(guid_prefix,ENTITYID_PARTICIPANT);
        let entity = Entity::new(guid);
        Self {
            entity,
            domain_id,
            protocol_version,
            vendor_id,
        }
    }
}
