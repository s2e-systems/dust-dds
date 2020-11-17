use crate::types::{ProtocolVersion, VendorId};
use crate::structure::RtpsEntity;

use rust_dds_interface::types::DomainId;

pub struct RtpsParticipant {
    pub entity: RtpsEntity,
    pub domain_id: DomainId,
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
}

impl RtpsParticipant {
    pub fn new(
        entity: RtpsEntity,
        domain_id: DomainId,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
    ) -> Self {
        Self {
            entity,
            domain_id,
            protocol_version,
            vendor_id,
        }
    }
}
