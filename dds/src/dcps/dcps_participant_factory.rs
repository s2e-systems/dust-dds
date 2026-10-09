use crate::{
    builtin_topics::{
        DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC,
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    dcps::{
        data_representation_builtin_endpoints::type_lookup::{TypeLookupReply, TypeLookupRequest},
        dcps_domain_participant::{
            builtin_constants::{
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
                ENTITYID_TL_SVC_REPLY_WRITER, ENTITYID_TL_SVC_REQ_WRITER,
                TYPE_LOOKUP_REPLY_TOPIC_NAME, TYPE_LOOKUP_REQUEST_TOPIC_NAME,
                TYPE_LOOKUP_WRITER_QOS,
            },
            builtin_publisher::BuiltinPublisher,
            builtin_subscriber::BuiltinSubscriber,
            data_writer_entity::DataWriterEntity,
            participant_entity::{DcpsDomainParticipant, DomainParticipantEntity},
            type_register::TypeRegister,
        },
        listeners::domain_participant_listener::DcpsDomainParticipantListener,
        status_mask::StatusMask,
        xtypes_glue::key_and_instance_handle::KeyHolderType,
    },
    dds_async::domain_participant_factory::DcpsSender,
    infrastructure::{
        configuration::DustDdsConfiguration,
        domain::DomainId,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{
            DataWriterQos, DomainParticipantFactoryQos, DomainParticipantQos, PublisherQos,
            QosKind, SubscriberQos, TopicQos,
        },
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        time::{Duration, DurationKind, Time},
    },
    rtps::{stateful_writer::RtpsStatefulWriter, stateless_writer::RtpsStatelessWriter},
    runtime::DdsRuntime,
    security::{
        plugins::{
            access_control::AccessControl, authentication::Authentication,
            cryptographic::Cryptographic, types::DdsSecurityPlugins,
        },
        types::{
            PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_DISCOVERY_PROTECTED,
            PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_KEY_REVISION_ENABLED,
            PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_LIVELINESS_PROTECTED,
            PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_RTPS_AXK_PROTECTED,
            PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_RTPS_PSK_PROTECTED,
            PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_VALID,
            PARTICIPANT_SECURITY_OPT_ATTRIBUTES_FLAG_ALLOW_UNAUTHENTICATED_PARTICIPANTS,
            PARTICIPANT_SECURITY_OPT_ATTRIBUTES_FLAG_IS_ACCESS_PROTECTED,
            ParticipantSecurityAttributesMaskExt, ParticipantSecurityConfig,
            ParticipantSecurityProtectionInfo,
        },
    },
    transport::{
        interface::RtpsParticipant,
        types::{ENTITYID_PARTICIPANT, Guid, GuidPrefix},
    },
    xtypes::type_support::Type,
};
use alloc::{
    collections::BTreeSet,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

pub struct DcpsParticipantFactory {
    pub domain_participant_list: Vec<DcpsDomainParticipant>,
    pub qos: DomainParticipantFactoryQos,
    pub default_participant_qos: DomainParticipantQos,
    pub configuration: DustDdsConfiguration,
    pub dcps_sender: DcpsSender,
}

impl DcpsParticipantFactory {
    pub fn new(configuration: DustDdsConfiguration, dcps_sender: DcpsSender) -> Self {
        Self {
            domain_participant_list: Default::default(),
            qos: Default::default(),
            default_participant_qos: Default::default(),
            configuration,
            dcps_sender,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_participant<Auth, Access, Crypto>(
        &mut self,
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        listener_mask: StatusMask,
        transport_participant: RtpsParticipant,
        now: Time,
        runtime: &impl DdsRuntime,
        _security_plugins: &mut Option<DdsSecurityPlugins<Auth, Access, Crypto>>,
    ) -> DdsResult<InstanceHandle>
    where
        Auth: Authentication,
        Access: AccessControl,
        Crypto: Cryptographic,
    {
        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let listener_sender = dcps_listener.map(|l| l.spawn(&runtime.spawner()));

        let guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);
        let security_data = None;

        let guid_prefix = guid.prefix();

        let mut dcps_participant_transport_writer = RtpsStatelessWriter::new(Guid::new(
            guid_prefix,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
        ));
        for &discovery_locator in &transport_participant.metatraffic_multicast_locator_list {
            dcps_participant_transport_writer.reader_locator_add(discovery_locator);
        }
        let dcps_participant_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_participant_transport_writer.guid().into()),
            dcps_participant_transport_writer,
            Arc::from(DCPS_PARTICIPANT),
            spdp_writer_qos(),
            KeyHolderType::new(&ParticipantBuiltinTopicData::TYPE),
        );

        let dcps_topics_transport_writer = RtpsStatefulWriter::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER),
            transport_participant.fragment_size,
        );
        let dcps_topics_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_topics_transport_writer.guid().into()),
            dcps_topics_transport_writer,
            Arc::from(DCPS_TOPIC),
            sedp_data_writer_qos(),
            KeyHolderType::new(&TopicBuiltinTopicData::TYPE),
        );

        let dcps_publications_transport_writer = RtpsStatefulWriter::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER),
            transport_participant.fragment_size,
        );
        let dcps_publications_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_publications_transport_writer.guid().into()),
            dcps_publications_transport_writer,
            Arc::from(DCPS_PUBLICATION),
            sedp_data_writer_qos(),
            KeyHolderType::new(&PublicationBuiltinTopicData::TYPE),
        );

        let dcps_subscriptions_transport_writer = RtpsStatefulWriter::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER),
            transport_participant.fragment_size,
        );
        let dcps_subscriptions_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_subscriptions_transport_writer.guid().into()),
            dcps_subscriptions_transport_writer,
            Arc::from(DCPS_SUBSCRIPTION),
            sedp_data_writer_qos(),
            KeyHolderType::new(&SubscriptionBuiltinTopicData::TYPE),
        );

        let type_lookup_request_transport_writer = RtpsStatefulWriter::new(
            Guid::new(guid_prefix, ENTITYID_TL_SVC_REQ_WRITER),
            transport_participant.fragment_size,
        );
        let type_lookup_request_writer = DataWriterEntity::new(
            InstanceHandle::new(type_lookup_request_transport_writer.guid().into()),
            type_lookup_request_transport_writer,
            Arc::from(TYPE_LOOKUP_REQUEST_TOPIC_NAME),
            TYPE_LOOKUP_WRITER_QOS,
            KeyHolderType::new(&TypeLookupRequest::TYPE),
        );

        let type_lookup_reply_transport_writer = RtpsStatefulWriter::new(
            Guid::new(guid_prefix, ENTITYID_TL_SVC_REPLY_WRITER),
            transport_participant.fragment_size,
        );
        let type_lookup_reply_writer = DataWriterEntity::new(
            InstanceHandle::new(type_lookup_reply_transport_writer.guid().into()),
            type_lookup_reply_transport_writer,
            Arc::from(TYPE_LOOKUP_REPLY_TOPIC_NAME),
            TYPE_LOOKUP_WRITER_QOS,
            KeyHolderType::new(&TypeLookupReply::TYPE),
        );

        let builtin_publisher = BuiltinPublisher {
            dcps_participant_writer,
            dcps_topics_writer,
            dcps_publications_writer,
            dcps_subscriptions_writer,
            type_lookup_request_writer,
            type_lookup_reply_writer,
            enabled: false,
        };

        let participant_handle = InstanceHandle::new(guid.into());

        let builtin_subscriber = BuiltinSubscriber::new(guid.prefix());

        let domain_participant = DomainParticipantEntity {
            domain_id,
            instance_handle: participant_handle,
            topic_counter: 0,
            qos: domain_participant_qos,
            builtin_subscriber,
            builtin_publisher,
            user_defined_subscriber_list: Vec::new(),
            default_subscriber_qos: SubscriberQos::const_default(),
            user_defined_publisher_list: Vec::new(),
            default_publisher_qos: PublisherQos::const_default(),
            locally_created_topic_list: Vec::new(),
            content_filtered_topic_list: Vec::new(),
            type_register: TypeRegister::new(),
            default_topic_qos: TopicQos::const_default(),
            discovered_participant_list: Vec::new(),
            discovered_topic_list: Vec::new(),
            discovered_reader_list: Vec::new(),
            discovered_writer_list: Vec::new(),
            enabled: false,
            ignored_participants: BTreeSet::new(),
            ignored_publications: BTreeSet::new(),
            ignored_subscriptions: BTreeSet::new(),
            _ignored_topic_list: BTreeSet::new(),
            listener_sender,
            listener_mask,
            find_topic_sender_list: Vec::new(),
            last_announcement_timestamp: None,
            security_data,
        };

        let mut dcps_participant = DcpsDomainParticipant {
            reader_counter: 0,
            writer_counter: 0,
            publisher_counter: 0,
            subscriber_counter: 0,
            domain_participant,
            dcps_sender: self.dcps_sender.clone(),
        };

        let participant_handle = *dcps_participant.get_instance_handle();

        if self.qos.entity_factory.autoenable_created_entities {
            dcps_participant
                .enable_domain_participant(now, self.configuration.domain_tag().to_string())?;
        }

        self.domain_participant_list.push(dcps_participant);

        Ok(participant_handle)
    }

    pub fn delete_participant(
        &mut self,
        participant_handle: &InstanceHandle,
        now: Time,
    ) -> DdsResult<()> {
        let index = self
            .domain_participant_list
            .iter()
            .position(|h| h.get_instance_handle() == participant_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        if !self.domain_participant_list[index].is_participant_empty() {
            return Err(DdsError::PreconditionNotMet(String::from(
                "Domain participant still contains other entities",
            )));
        }
        let mut participant = self.domain_participant_list.remove(index);
        participant.announce_deleted_participant(now);
        Ok(())
    }

    pub fn find_participant(
        &mut self,
        participant_handle: &InstanceHandle,
    ) -> DdsResult<&mut DcpsDomainParticipant> {
        self.domain_participant_list
            .iter_mut()
            .find(|x| x.get_instance_handle() == participant_handle)
            .ok_or(DdsError::AlreadyDeleted)
    }

    pub fn set_default_participant_qos(
        &mut self,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_participant_qos = qos;

        Ok(())
    }

    pub fn get_default_participant_qos(&mut self) -> DomainParticipantQos {
        self.default_participant_qos.clone()
    }

    pub fn set_qos(&mut self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;
        Ok(())
    }

    pub fn get_qos(&mut self) -> DomainParticipantFactoryQos {
        self.qos.clone()
    }

    pub fn time_until_next_event(&self, now: Time) -> Option<Duration> {
        self.domain_participant_list
            .iter()
            .filter_map(|x| {
                x.time_until_next_event(
                    now,
                    self.configuration
                        .participant_announcement_interval()
                        .into(),
                )
            })
            .min()
    }
}

impl From<&ParticipantSecurityConfig> for ParticipantSecurityProtectionInfo {
    fn from(config: &ParticipantSecurityConfig) -> Self {
        let mut mask = PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_VALID;
        if config.is_rtps_axk_protected {
            mask |= PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_RTPS_AXK_PROTECTED;
        }
        if config.is_discovery_protected {
            mask |= PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_DISCOVERY_PROTECTED;
        }
        if config.is_liveliness_protected {
            mask |= PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_LIVELINESS_PROTECTED;
        }
        if config.is_key_revision_enabled {
            mask |= PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_KEY_REVISION_ENABLED;
        }
        if config.is_rtps_psk_protected {
            mask |= PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_RTPS_PSK_PROTECTED;
        }

        let mut opt_mask: u16 = 0;
        let mut opt_is_set: u16 = 0;

        opt_is_set |= PARTICIPANT_SECURITY_OPT_ATTRIBUTES_FLAG_ALLOW_UNAUTHENTICATED_PARTICIPANTS;
        if config.allow_unauthenticated_participants {
            opt_mask |= PARTICIPANT_SECURITY_OPT_ATTRIBUTES_FLAG_ALLOW_UNAUTHENTICATED_PARTICIPANTS;
        }

        opt_is_set |= PARTICIPANT_SECURITY_OPT_ATTRIBUTES_FLAG_IS_ACCESS_PROTECTED;
        if config.is_access_protected {
            opt_mask |= PARTICIPANT_SECURITY_OPT_ATTRIBUTES_FLAG_IS_ACCESS_PROTECTED;
        }

        Self {
            participant_security_attributes: mask,
            plugin_participant_security_attributes: config.plugin_participant_attributes,
            participant_security_optional_attributes: ParticipantSecurityAttributesMaskExt {
                is_set: opt_is_set,
                value: opt_mask,
            },
        }
    }
}

fn spdp_writer_qos() -> DataWriterQos {
    DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::BestEffort,
            max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
        },
        ..Default::default()
    }
}

fn sedp_data_writer_qos() -> DataWriterQos {
    DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(0, 0)),
        },
        ..Default::default()
    }
}
