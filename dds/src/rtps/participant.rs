use crate::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
    },
    domain::domain_participant_factory::DomainId,
    implementation::{
        actor::{ActorAddress, Mail, MailHandler},
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
            spdp_discovered_participant_data::{ParticipantProxy, SpdpDiscoveredParticipantData},
        },
    },
    rtps::{
        cache_change::RtpsCacheChange,
        discovery_types::{
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        },
        message_receiver::MessageReceiver,
        stateful_writer::{RtpsStatefulWriter, WriterHistoryCache},
        types::{SequenceNumber, ENTITYID_UNKNOWN},
    },
    topic_definition::type_support::{DdsDeserialize, DdsSerialize},
};

use super::{
    behavior_types::{Duration, InstanceHandle},
    discovery_types::{
        BuiltinEndpointQos, BuiltinEndpointSet, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
        ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
    },
    entity::RtpsEntity,
    error::RtpsResult,
    message_sender::MessageSender,
    messages::overall_structure::RtpsMessageRead,
    reader::{ReaderHistoryCache, RtpsStatefulReader, RtpsStatelessReader},
    stateless_writer::RtpsStatelessWriter,
    types::{
        DurabilityKind, Guid, Locator, ProtocolVersion, ReliabilityKind, VendorId,
        PROTOCOLVERSION_2_4, VENDOR_ID_S2E,
    },
};

pub struct RtpsParticipant {
    entity: RtpsEntity,
    domain_id: DomainId,
    domain_tag: String,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    builtin_stateless_writer_list: Vec<RtpsStatelessWriter>,
    builtin_stateful_writer_list: Vec<RtpsStatefulWriter>,
    builtin_stateless_reader_list: Vec<RtpsStatelessReader>,
    builtin_stateful_reader_list: Vec<RtpsStatefulReader>,
    user_defined_writer_list: Vec<RtpsStatefulWriter>,
    user_defined_reader_list: Vec<RtpsStatefulReader>,
    message_sender: MessageSender,
    discovered_participant_list: Vec<InstanceHandle>,
}

impl RtpsParticipant {
    pub fn new(
        guid: Guid,
        domain_id: DomainId,
        domain_tag: String,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        metatraffic_unicast_locator_list: Vec<Locator>,
        metatraffic_multicast_locator_list: Vec<Locator>,
        spdp_builtin_participant_reader_history_cache: Box<dyn ReaderHistoryCache>,
        sedp_builtin_topics_reader_history_cache: Box<dyn ReaderHistoryCache>,
        sedp_builtin_publications_reader_history_cache: Box<dyn ReaderHistoryCache>,
        sedp_builtin_subscriptions_reader_history_cache: Box<dyn ReaderHistoryCache>,
    ) -> RtpsResult<Self> {
        let guid_prefix = guid.prefix();
        let message_sender =
            MessageSender::new(guid_prefix, std::net::UdpSocket::bind("0.0.0.0:0000")?);

        let mut spdp_builtin_participant_writer = RtpsStatelessWriter::new(
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER),
            message_sender.clone(),
        );
        for locator in &metatraffic_multicast_locator_list {
            spdp_builtin_participant_writer.reader_locator_add(locator.clone());
        }

        let spdp_builtin_participant_reader = RtpsStatelessReader::new(
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER),
            spdp_builtin_participant_reader_history_cache,
        );

        let sedp_builtin_topics_writer = RtpsStatefulWriter::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER),
            message_sender.clone(),
        );

        let sedp_builtin_topics_reader = RtpsStatefulReader::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR),
            sedp_builtin_topics_reader_history_cache,
            message_sender.clone(),
        );

        let sedp_builtin_publications_writer = RtpsStatefulWriter::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER),
            message_sender.clone(),
        );

        let sedp_builtin_publications_reader = RtpsStatefulReader::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
            sedp_builtin_publications_reader_history_cache,
            message_sender.clone(),
        );

        let sedp_builtin_subscriptions_writer = RtpsStatefulWriter::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER),
            message_sender.clone(),
        );

        let sedp_builtin_subscriptions_reader = RtpsStatefulReader::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR),
            sedp_builtin_subscriptions_reader_history_cache,
            message_sender.clone(),
        );

        let builtin_stateless_writer_list = vec![spdp_builtin_participant_writer];
        let builtin_stateless_reader_list = vec![spdp_builtin_participant_reader];
        let builtin_stateful_writer_list = vec![
            sedp_builtin_topics_writer,
            sedp_builtin_publications_writer,
            sedp_builtin_subscriptions_writer,
        ];
        let builtin_stateful_reader_list = vec![
            sedp_builtin_topics_reader,
            sedp_builtin_publications_reader,
            sedp_builtin_subscriptions_reader,
        ];
        let user_defined_writer_list = Vec::new();
        let user_defined_reader_list = Vec::new();

        Ok(Self {
            entity: RtpsEntity::new(guid),
            domain_id,
            domain_tag,
            protocol_version: PROTOCOLVERSION_2_4,
            vendor_id: VENDOR_ID_S2E,
            default_unicast_locator_list,
            default_multicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            builtin_stateless_writer_list,
            builtin_stateful_writer_list,
            builtin_stateless_reader_list,
            builtin_stateful_reader_list,
            user_defined_writer_list,
            user_defined_reader_list,
            message_sender,
            discovered_participant_list: Vec::new(),
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

    pub fn add_discovered_participant(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        // Check that the domainId of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // AND
        // Check that the domainTag of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // IN CASE no domain id was transmitted the a local domain id is assumed
        // (as specified in Table 9.19 - ParameterId mapping and default values)
        let is_domain_id_matching = discovered_participant_data
            .participant_proxy
            .domain_id
            .unwrap_or(self.domain_id)
            == self.domain_id;
        let is_domain_tag_matching =
            discovered_participant_data.participant_proxy.domain_tag == self.domain_tag;
        let discovered_participant_handle =
            InstanceHandle(discovered_participant_data.dds_participant_data.key().value);

        let is_participant_discovered = self
            .discovered_participant_list
            .contains(&discovered_participant_handle);
        if is_domain_id_matching && is_domain_tag_matching && !is_participant_discovered {
            self.add_matched_publications_detector(&discovered_participant_data);
            self.add_matched_publications_announcer(&discovered_participant_data);
            self.add_matched_subscriptions_detector(&discovered_participant_data);
            self.add_matched_subscriptions_announcer(&discovered_participant_data);
            self.add_matched_topics_detector(&discovered_participant_data);
            self.add_matched_topics_announcer(&discovered_participant_data);
        }
    }

    fn add_matched_publications_detector(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            if let Some(w) = self
                .builtin_stateful_writer_list
                .iter_mut()
                .find(|w| w.guid().entity_id() == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER)
            {
                w.add_matched_reader(
                    reader_proxy,
                    ReliabilityKind::Reliable,
                    DurabilityKind::TransientLocal,
                );
            }
        }
    }

    fn add_matched_publications_announcer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = Default::default();

            let writer_proxy = WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                data_max_size_serialized,
            };
            if let Some(r) = self
                .builtin_stateful_reader_list
                .iter_mut()
                .find(|w| w.guid().entity_id() == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR)
            {
                r.add_matched_writer(
                    writer_proxy,
                    ReliabilityKind::Reliable,
                    DurabilityKind::TransientLocal,
                );
            }
        }
    }

    fn add_matched_subscriptions_detector(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            if let Some(w) = self
                .builtin_stateful_writer_list
                .iter_mut()
                .find(|w| w.guid().entity_id() == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER)
            {
                w.add_matched_reader(
                    reader_proxy,
                    ReliabilityKind::Reliable,
                    DurabilityKind::TransientLocal,
                );
            }
        }
    }

    fn add_matched_subscriptions_announcer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = Default::default();

            let writer_proxy = WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                data_max_size_serialized,
            };
            if let Some(r) = self
                .builtin_stateful_reader_list
                .iter_mut()
                .find(|w| w.guid().entity_id() == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR)
            {
                r.add_matched_writer(
                    writer_proxy,
                    ReliabilityKind::Reliable,
                    DurabilityKind::TransientLocal,
                );
            }
        }
    }

    fn add_matched_topics_detector(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                expects_inline_qos,
            };
            if let Some(w) = self
                .builtin_stateful_writer_list
                .iter_mut()
                .find(|w| w.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER)
            {
                w.add_matched_reader(
                    reader_proxy,
                    ReliabilityKind::Reliable,
                    DurabilityKind::TransientLocal,
                );
            }
        }
    }

    fn add_matched_topics_announcer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let data_max_size_serialized = Default::default();

            let writer_proxy = WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                data_max_size_serialized,
            };
            if let Some(r) = self
                .builtin_stateful_reader_list
                .iter_mut()
                .find(|w| w.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR)
            {
                r.add_matched_writer(
                    writer_proxy,
                    ReliabilityKind::Reliable,
                    DurabilityKind::TransientLocal,
                );
            }
        }
    }

    pub fn participant_proxy(&self) -> ParticipantProxy {
        ParticipantProxy {
            domain_id: Some(self.domain_id),
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

    pub fn create_writer(&mut self, writer_guid: Guid) -> Guid {
        let writer = RtpsStatefulWriter::new(writer_guid, self.message_sender.clone());
        self.user_defined_writer_list.push(writer);
        writer_guid
    }

    pub fn delete_writer(&mut self, writer_guid: Guid) {
        self.user_defined_writer_list
            .retain(|x| x.guid() != writer_guid);
    }

    pub fn create_reader(
        &mut self,
        reader_guid: Guid,
        reader_history_cache: Box<dyn ReaderHistoryCache>,
    ) {
        let reader = RtpsStatefulReader::new(
            reader_guid,
            reader_history_cache,
            self.message_sender.clone(),
        );
        self.user_defined_reader_list.push(reader);
    }

    pub fn delete_reader(&mut self, reader_guid: Guid) {
        self.user_defined_reader_list
            .retain(|x| x.guid() != reader_guid);
    }

    pub fn process_builtin_rtps_message(&mut self, message: RtpsMessageRead) {
        MessageReceiver::new(message).process_message(
            &mut self.builtin_stateless_reader_list,
            &mut self.builtin_stateful_reader_list,
            &mut self.builtin_stateful_writer_list,
        );
    }

    pub fn process_user_defined_rtps_message(&mut self, message: RtpsMessageRead) {
        MessageReceiver::new(message).process_message(
            &mut [],
            &mut self.user_defined_reader_list,
            &mut self.user_defined_writer_list,
        );
    }
}

pub struct ProcessBuiltinRtpsMessage {
    pub rtps_message: RtpsMessageRead,
}
impl Mail for ProcessBuiltinRtpsMessage {
    type Result = ();
}
impl MailHandler<ProcessBuiltinRtpsMessage> for RtpsParticipant {
    fn handle(
        &mut self,
        message: ProcessBuiltinRtpsMessage,
    ) -> <ProcessBuiltinRtpsMessage as Mail>::Result {
        self.process_builtin_rtps_message(message.rtps_message);
    }
}

pub struct ProcessUserDefinedRtpsMessage {
    pub rtps_message: RtpsMessageRead,
}
impl Mail for ProcessUserDefinedRtpsMessage {
    type Result = ();
}
impl MailHandler<ProcessUserDefinedRtpsMessage> for RtpsParticipant {
    fn handle(
        &mut self,
        message: ProcessUserDefinedRtpsMessage,
    ) -> <ProcessUserDefinedRtpsMessage as Mail>::Result {
        self.process_user_defined_rtps_message(message.rtps_message);
    }
}

pub struct SendHeartbeat;
impl Mail for SendHeartbeat {
    type Result = ();
}
impl MailHandler<SendHeartbeat> for RtpsParticipant {
    fn handle(&mut self, _: SendHeartbeat) -> <SendHeartbeat as Mail>::Result {
        for builtin_writer in self.builtin_stateful_writer_list.iter_mut() {
            builtin_writer.send_message();
        }
        for user_defined_writer in self.user_defined_writer_list.iter_mut() {
            user_defined_writer.send_message();
        }
    }
}

pub struct CreateWriter {
    pub writer_guid: Guid,
    pub rtps_participant_address: ActorAddress<RtpsParticipant>,
}

impl Mail for CreateWriter {
    type Result = Box<dyn WriterHistoryCache>;
}
impl MailHandler<CreateWriter> for RtpsParticipant {
    fn handle(&mut self, message: CreateWriter) -> <CreateWriter as Mail>::Result {
        let guid = self.create_writer(message.writer_guid);

        struct RtpsUserDefinedWriterHistoryCache {
            rtps_participant_address: ActorAddress<RtpsParticipant>,
            guid: Guid,
        }
        impl WriterHistoryCache for RtpsUserDefinedWriterHistoryCache {
            fn guid(&self) -> [u8; 16] {
                self.guid.into()
            }

            fn add_change(&mut self, cache_change: RtpsCacheChange) {
                self.rtps_participant_address
                    .send_actor_mail(AddUserDefinedCacheChange {
                        guid: self.guid,
                        cache_change,
                    })
                    .ok();
            }

            fn remove_change(&mut self, sequence_number: SequenceNumber) {
                self.rtps_participant_address
                    .send_actor_mail(RemoveUserDefinedCacheChange {
                        guid: self.guid,
                        sequence_number,
                    })
                    .ok();
            }

            fn are_all_changes_acknowledged(&self) -> bool {
                todo!()
            }
        }

        Box::new(RtpsUserDefinedWriterHistoryCache {
            rtps_participant_address: message.rtps_participant_address,
            guid,
        })
    }
}

pub struct CreateReader {
    pub reader_guid: Guid,
    pub reader_history_cache: Box<dyn ReaderHistoryCache>,
}

impl Mail for CreateReader {
    type Result = ();
}
impl MailHandler<CreateReader> for RtpsParticipant {
    fn handle(&mut self, message: CreateReader) -> <CreateReader as Mail>::Result {
        self.create_reader(message.reader_guid, message.reader_history_cache)
    }
}

pub struct AddParticipantDiscoveryCacheChange {
    pub cache_change: RtpsCacheChange,
}
impl Mail for AddParticipantDiscoveryCacheChange {
    type Result = ();
}
impl MailHandler<AddParticipantDiscoveryCacheChange> for RtpsParticipant {
    fn handle(
        &mut self,
        message: AddParticipantDiscoveryCacheChange,
    ) -> <AddParticipantDiscoveryCacheChange as Mail>::Result {
        let participant_proxy = self.participant_proxy();
        if let Some(w) = self
            .builtin_stateless_writer_list
            .iter_mut()
            .find(|dw| dw.guid().entity_id() == ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER)
        {
            if let Ok(dds_participant_data) = ParticipantBuiltinTopicData::deserialize_data(
                message.cache_change.data_value.as_ref(),
            ) {
                let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
                    dds_participant_data,
                    participant_proxy,
                    lease_duration: Duration::new(100, 0).into(),
                    discovered_participant_list: vec![],
                };

                let mut cache_change = message.cache_change;
                cache_change.data_value = spdp_discovered_participant_data
                    .serialize_data()
                    .unwrap()
                    .into();
                w.add_change(cache_change);
            }
        }
    }
}

pub struct RemoveParticipantDiscoveryCacheChange {
    pub sequence_number: SequenceNumber,
}
impl Mail for RemoveParticipantDiscoveryCacheChange {
    type Result = ();
}
impl MailHandler<RemoveParticipantDiscoveryCacheChange> for RtpsParticipant {
    fn handle(
        &mut self,
        message: RemoveParticipantDiscoveryCacheChange,
    ) -> <RemoveParticipantDiscoveryCacheChange as Mail>::Result {
        if let Some(w) = self
            .builtin_stateless_writer_list
            .iter_mut()
            .find(|dw| dw.guid().entity_id() == ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER)
        {
            w.remove_change(message.sequence_number);
        }
    }
}

pub struct AddTopicsDiscoveryCacheChange {
    pub cache_change: RtpsCacheChange,
}
impl Mail for AddTopicsDiscoveryCacheChange {
    type Result = ();
}
impl MailHandler<AddTopicsDiscoveryCacheChange> for RtpsParticipant {
    fn handle(
        &mut self,
        message: AddTopicsDiscoveryCacheChange,
    ) -> <AddTopicsDiscoveryCacheChange as Mail>::Result {
        if let Some(w) = self
            .builtin_stateful_writer_list
            .iter_mut()
            .find(|dw| dw.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER)
        {
            w.add_change(message.cache_change);
        }
    }
}

pub struct RemoveTopicsDiscoveryCacheChange {
    pub sequence_number: SequenceNumber,
}
impl Mail for RemoveTopicsDiscoveryCacheChange {
    type Result = ();
}
impl MailHandler<RemoveTopicsDiscoveryCacheChange> for RtpsParticipant {
    fn handle(
        &mut self,
        message: RemoveTopicsDiscoveryCacheChange,
    ) -> <RemoveTopicsDiscoveryCacheChange as Mail>::Result {
        if let Some(w) = self
            .builtin_stateless_writer_list
            .iter_mut()
            .find(|dw| dw.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER)
        {
            w.remove_change(message.sequence_number);
        }
    }
}

pub struct AddPublicationsDiscoveryCacheChange {
    pub cache_change: RtpsCacheChange,
}
impl Mail for AddPublicationsDiscoveryCacheChange {
    type Result = ();
}
impl MailHandler<AddPublicationsDiscoveryCacheChange> for RtpsParticipant {
    fn handle(
        &mut self,
        message: AddPublicationsDiscoveryCacheChange,
    ) -> <AddPublicationsDiscoveryCacheChange as Mail>::Result {
        if let Some(w) = self
            .builtin_stateful_writer_list
            .iter_mut()
            .find(|dw| dw.guid().entity_id() == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER)
        {
            if let Ok(dds_publication_data) = PublicationBuiltinTopicData::deserialize_data(
                message.cache_change.data_value.as_ref(),
            ) {
                if let Some(writer_proxy) = self
                    .user_defined_writer_list
                    .iter()
                    .find(|w| w.guid() == dds_publication_data.key.value.into())
                    .map(|w| w.writer_proxy())
                {
                    let mut cache_change = message.cache_change;
                    let discovered_writer_data = DiscoveredWriterData {
                        dds_publication_data,
                        writer_proxy,
                    };
                    cache_change.data_value =
                        discovered_writer_data.serialize_data().unwrap().into();
                    w.add_change(cache_change);
                }
            }
        }
    }
}

pub struct RemovePublicationsDiscoveryCacheChange {
    pub sequence_number: SequenceNumber,
}
impl Mail for RemovePublicationsDiscoveryCacheChange {
    type Result = ();
}
impl MailHandler<RemovePublicationsDiscoveryCacheChange> for RtpsParticipant {
    fn handle(
        &mut self,
        message: RemovePublicationsDiscoveryCacheChange,
    ) -> <RemovePublicationsDiscoveryCacheChange as Mail>::Result {
        if let Some(w) = self
            .builtin_stateful_writer_list
            .iter_mut()
            .find(|dw| dw.guid().entity_id() == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER)
        {
            w.remove_change(message.sequence_number);
        }
    }
}

pub struct AddSubscriptionsDiscoveryCacheChange {
    pub cache_change: RtpsCacheChange,
}
impl Mail for AddSubscriptionsDiscoveryCacheChange {
    type Result = ();
}
impl MailHandler<AddSubscriptionsDiscoveryCacheChange> for RtpsParticipant {
    fn handle(
        &mut self,
        message: AddSubscriptionsDiscoveryCacheChange,
    ) -> <AddSubscriptionsDiscoveryCacheChange as Mail>::Result {
        if let Some(w) = self
            .builtin_stateful_writer_list
            .iter_mut()
            .find(|dw| dw.guid().entity_id() == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER)
        {
            if let Ok(dds_subscription_data) = SubscriptionBuiltinTopicData::deserialize_data(
                message.cache_change.data_value.as_ref(),
            ) {
                if let Some(reader_proxy) = self
                    .user_defined_reader_list
                    .iter()
                    .find(|r| r.guid() == dds_subscription_data.key.value.into())
                    .map(|r| r.reader_proxy())
                {
                    let mut cache_change = message.cache_change;
                    let discovered_reader_data = DiscoveredReaderData {
                        dds_subscription_data,
                        reader_proxy,
                    };
                    cache_change.data_value =
                        discovered_reader_data.serialize_data().unwrap().into();
                    w.add_change(cache_change);
                }
            }
        }
    }
}

pub struct RemoveSubscriptionsDiscoveryCacheChange {
    pub sequence_number: SequenceNumber,
}
impl Mail for RemoveSubscriptionsDiscoveryCacheChange {
    type Result = ();
}
impl MailHandler<RemoveSubscriptionsDiscoveryCacheChange> for RtpsParticipant {
    fn handle(
        &mut self,
        message: RemoveSubscriptionsDiscoveryCacheChange,
    ) -> <RemoveSubscriptionsDiscoveryCacheChange as Mail>::Result {
        if let Some(w) = self
            .builtin_stateful_writer_list
            .iter_mut()
            .find(|dw| dw.guid().entity_id() == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER)
        {
            w.remove_change(message.sequence_number);
        }
    }
}

pub struct AddUserDefinedCacheChange {
    pub guid: Guid,
    pub cache_change: RtpsCacheChange,
}
impl Mail for AddUserDefinedCacheChange {
    type Result = ();
}
impl MailHandler<AddUserDefinedCacheChange> for RtpsParticipant {
    fn handle(
        &mut self,
        message: AddUserDefinedCacheChange,
    ) -> <AddUserDefinedCacheChange as Mail>::Result {
        if let Some(w) = self
            .user_defined_writer_list
            .iter_mut()
            .find(|dw| dw.guid() == message.guid)
        {
            w.add_change(message.cache_change);
        }
    }
}

pub struct RemoveUserDefinedCacheChange {
    pub guid: Guid,
    pub sequence_number: SequenceNumber,
}
impl Mail for RemoveUserDefinedCacheChange {
    type Result = ();
}
impl MailHandler<RemoveUserDefinedCacheChange> for RtpsParticipant {
    fn handle(
        &mut self,
        message: RemoveUserDefinedCacheChange,
    ) -> <RemoveUserDefinedCacheChange as Mail>::Result {
        if let Some(w) = self
            .user_defined_writer_list
            .iter_mut()
            .find(|dw| dw.guid() == message.guid)
        {
            w.remove_change(message.sequence_number);
        }
    }
}

pub struct AddDiscoveredParticipant {
    pub discovered_participant_data: SpdpDiscoveredParticipantData,
}
impl Mail for AddDiscoveredParticipant {
    type Result = ();
}
impl MailHandler<AddDiscoveredParticipant> for RtpsParticipant {
    fn handle(
        &mut self,
        message: AddDiscoveredParticipant,
    ) -> <AddDiscoveredParticipant as Mail>::Result {
        self.add_discovered_participant(&message.discovered_participant_data);
    }
}
