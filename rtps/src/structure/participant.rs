use crate::types::{ProtocolVersion, VendorId, GuidPrefix, GUID};
use crate::types::constants::ENTITYID_PARTICIPANT;
use crate::structure::RtpsEntity;

use rust_dds_api::types::DomainId;

pub struct RtpsParticipant {
    pub entity: RtpsEntity,
    pub domain_id: DomainId,
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
}

impl RtpsParticipant {
    pub fn new(
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
    ) -> Self {
        let guid = GUID::new(guid_prefix,ENTITYID_PARTICIPANT);
        let entity = RtpsEntity::new(guid);
        Self {
            entity,
            domain_id,
            protocol_version,
            vendor_id,
        }
    }
}
