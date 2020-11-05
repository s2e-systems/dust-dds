use std::sync::{Arc, Mutex, };

use crate::types::{GUID, GuidPrefix, ProtocolVersion, VendorId};
use crate::types::constants::{
    ENTITYID_PARTICIPANT,
    PROTOCOL_VERSION_2_4,};

use super::{RtpsGroup, RtpsEntity};

use rust_dds_interface::types::DomainId;

pub struct RtpsParticipant {
    guid: GUID,
    domain_id: DomainId,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,

    groups: Vec<Arc<Mutex<RtpsGroup>>>,
}

impl RtpsParticipant {
    pub fn new(
        domain_id: DomainId,
        guid_prefix: GuidPrefix,
    ) -> Self {
        let protocol_version = PROTOCOL_VERSION_2_4;
        let vendor_id = [99,99];

        Self {
            guid: GUID::new(guid_prefix,ENTITYID_PARTICIPANT ),
            domain_id,
            protocol_version,
            vendor_id,
            groups: Vec::new(),
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
    
    pub fn mut_groups(&mut self) -> &mut Vec<Arc<Mutex<RtpsGroup>>> {
        &mut self.groups
    }
}

impl RtpsEntity for RtpsParticipant {
    fn guid(&self) -> GUID {
        self.guid
    }
}