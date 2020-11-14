use crate::types::{GUID, GuidPrefix, ProtocolVersion, VendorId};
use crate::types::constants::ENTITYID_PARTICIPANT;
use crate::structure::RtpsEntity;

use rust_dds_interface::types::DomainId;

pub struct RtpsParticipant {
    guid: GUID,
    domain_id: DomainId,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
}

impl RtpsParticipant {
    pub fn new(
        domain_id: DomainId,
        guid_prefix: GuidPrefix,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
    ) -> Self {
        Self {
            guid: GUID::new(guid_prefix,ENTITYID_PARTICIPANT ),
            domain_id,
            protocol_version,
            vendor_id,
        }
    }

    pub fn domain_id(&self) -> DomainId {
        self.domain_id
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    pub fn vendor_id(&self) -> VendorId {
        self.vendor_id
    }
}

impl RtpsEntity for RtpsParticipant {
    fn guid(&self) -> GUID {
        self.guid
    }
}