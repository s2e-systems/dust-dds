use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::rtps::writer::RtpsWriter;

use super::{
    entity::RtpsEntity,
    types::{Guid, GuidPrefix, Locator, ProtocolVersion, VendorId, ENTITYID_PARTICIPANT},
    writer::TransportWriter,
};

pub struct RtpsParticipant {
    entity: RtpsEntity,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    writer_list: HashMap<[u8; 16], Arc<Mutex<RtpsWriter>>>,
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
            writer_list: HashMap::new(),
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

    pub fn set_default_unicast_locator_list(&mut self, list: Vec<Locator>) {
        self.default_unicast_locator_list = list;
    }

    pub fn default_multicast_locator_list(&self) -> &[Locator] {
        self.default_multicast_locator_list.as_slice()
    }

    pub fn set_default_multicast_locator_list(&mut self, list: Vec<Locator>) {
        self.default_multicast_locator_list = list;
    }

    pub fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        self.metatraffic_unicast_locator_list.as_ref()
    }

    pub fn set_metatraffic_unicast_locator_list(&mut self, list: Vec<Locator>) {
        self.metatraffic_unicast_locator_list = list;
    }

    pub fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        self.metatraffic_multicast_locator_list.as_ref()
    }

    pub fn set_metatraffic_multicast_locator_list(&mut self, list: Vec<Locator>) {
        self.metatraffic_multicast_locator_list = list;
    }
}

impl RtpsParticipant {
    pub fn create_writer(&mut self, writer_guid: Guid) -> Arc<Mutex<dyn TransportWriter>> {
        let writer = Arc::new(Mutex::new(RtpsWriter::new(writer_guid)));
        self.writer_list.insert(writer_guid.into(), writer.clone());
        writer
    }

    pub fn delete_writer(&mut self, writer_guid: Guid) {
        self.writer_list.remove(&<[u8; 16]>::from(writer_guid));
    }
}
