use std::{
    collections::HashMap,
    net::UdpSocket,
    sync::{Arc, Mutex},
};

use crate::{
    implementation::data_representation_builtin_endpoints::spdp_discovered_participant_data::ParticipantProxy,
    rtps::stateful_writer::RtpsStatefulWriter,
};

use super::{
    discovery_types::{BuiltinEndpointQos, BuiltinEndpointSet},
    entity::RtpsEntity,
    error::RtpsResult,
    message_sender::MessageSender,
    stateful_writer::TransportWriter,
    stateless_writer::RtpsStatelessWriter,
    types::{
        Guid, GuidPrefix, Locator, ProtocolVersion, VendorId, ENTITYID_PARTICIPANT,
        PROTOCOLVERSION_2_4, VENDOR_ID_S2E,
    },
};

pub struct RtpsParticipant {
    entity: RtpsEntity,
    domain_tag: String,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    builtin_stateless_writer_list: Vec<Arc<Mutex<RtpsStatelessWriter>>>,
    builtin_stateful_writer_list: Vec<Arc<Mutex<RtpsStatefulWriter>>>,
    user_defined_writer_list: HashMap<[u8; 16], Arc<Mutex<RtpsStatefulWriter>>>,
    sender_socket: UdpSocket,
}

impl RtpsParticipant {
    pub fn new(
        guid_prefix: GuidPrefix,
        domain_tag: String,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        metatraffic_unicast_locator_list: Vec<Locator>,
        metatraffic_multicast_locator_list: Vec<Locator>,
        protocol_version: ProtocolVersion,
        vendor_id: VendorId,
    ) -> RtpsResult<Self> {
        let sender_socket = std::net::UdpSocket::bind("0.0.0.0:0000")?;
        Ok(Self {
            entity: RtpsEntity::new(Guid::new(guid_prefix, ENTITYID_PARTICIPANT)),
            domain_tag,
            protocol_version,
            vendor_id,
            default_unicast_locator_list,
            default_multicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            builtin_stateless_writer_list: Vec::new(),
            builtin_stateful_writer_list: Vec::new(),
            user_defined_writer_list: HashMap::new(),
            sender_socket,
        })
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

    pub fn create_builtin_stateless_writer(
        &mut self,
        writer_guid: Guid,
    ) -> Arc<Mutex<RtpsStatelessWriter>> {
        let socket = self
            .sender_socket
            .try_clone()
            .expect("Should always be clone");
        let message_sender = MessageSender::new(self.entity.guid().prefix(), socket);
        let writer = Arc::new(Mutex::new(RtpsStatelessWriter::new(
            writer_guid,
            message_sender,
        )));
        self.builtin_stateless_writer_list.push(writer.clone());
        writer
    }

    pub fn create_builtin_stateful_writer(
        &mut self,
        writer_guid: Guid,
    ) -> Arc<Mutex<RtpsStatefulWriter>> {
        let socket = self
            .sender_socket
            .try_clone()
            .expect("Should always be clone");
        let message_sender = MessageSender::new(self.entity.guid().prefix(), socket);
        let writer = Arc::new(Mutex::new(RtpsStatefulWriter::new(
            writer_guid,
            message_sender,
        )));
        self.builtin_stateful_writer_list.push(writer.clone());
        writer
    }
}

impl RtpsParticipant {
    pub fn participant_proxy(&self) -> ParticipantProxy {
        ParticipantProxy {
            domain_id: None,
            domain_tag: self.domain_tag.clone(),
            protocol_version: PROTOCOLVERSION_2_4,
            guid_prefix: self.entity.guid().prefix(),
            vendor_id: VENDOR_ID_S2E,
            expects_inline_qos: false,
            metatraffic_unicast_locator_list: self.metatraffic_unicast_locator_list.clone(),
            metatraffic_multicast_locator_list: self.metatraffic_multicast_locator_list.clone(),
            default_unicast_locator_list: self.default_unicast_locator_list.clone(),
            default_multicast_locator_list: self.default_multicast_locator_list.clone(),
            available_builtin_endpoints: BuiltinEndpointSet::default(),
            manual_liveliness_count: 0,
            builtin_endpoint_qos: BuiltinEndpointQos::default(),
        }
    }

    pub fn create_writer(
        &mut self,
        writer_guid: Guid,
    ) -> Arc<Mutex<dyn TransportWriter + Send + Sync + 'static>> {
        let socket = self
            .sender_socket
            .try_clone()
            .expect("Should always be clone");
        let message_sender = MessageSender::new(self.entity.guid().prefix(), socket);
        let writer = Arc::new(Mutex::new(RtpsStatefulWriter::new(
            writer_guid,
            message_sender,
        )));
        self.user_defined_writer_list
            .insert(writer_guid.into(), writer.clone());
        writer
    }

    pub fn delete_writer(&mut self, writer_guid: Guid) {
        self.user_defined_writer_list
            .remove(&<[u8; 16]>::from(writer_guid));
    }
}
