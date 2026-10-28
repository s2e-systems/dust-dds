use crate::{
    builtin_topics::{
        DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC,
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    dcps::{
        channels::rpc::rpc_channel,
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
        dcps_mail::{DcpsReply, DomainParticipantMail, ParticipantServiceMail},
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
    runtime::{Either, Either3, TaskHandle, select_future, select3_future},
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
        interface::{RtpsParticipant, Transport},
        types::Guid,
    },
    xtypes::type_support::Type,
};
use alloc::{boxed::Box, collections::BTreeSet, string::ToString, sync::Arc, vec::Vec};
use core::sync::atomic::AtomicBool;

struct ParticipantRecord {
    handle: InstanceHandle,
    domain_id: DomainId,
    sender: DcpsSender,
    run_loop: Arc<AtomicBool>,
    task_handle: Box<dyn TaskHandle>,
}

pub struct DcpsParticipantFactory {
    participant_list: Vec<ParticipantRecord>,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
    configuration: DustDdsConfiguration,
}

impl DcpsParticipantFactory {
    pub fn new(configuration: DustDdsConfiguration) -> Self {
        Self {
            participant_list: Default::default(),
            qos: Default::default(),
            default_participant_qos: Default::default(),
            configuration,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_participant<
        Auth,
        Access,
        Crypto,
        T: crate::transport::interface::Transport,
        Timer: crate::runtime::Timer + Clone + Send + 'static,
        Clock: crate::runtime::Clock + Clone + Send + 'static,
        Spawner: crate::runtime::Spawner + Clone + Send + 'static,
    >(
        &mut self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        listener_mask: StatusMask,
        transport: &mut T,
        now: Time,
        timer: &Timer,
        clock: &Clock,
        spawner: &Spawner,
        _security_plugins: &mut Option<DdsSecurityPlugins<Auth, Access, Crypto>>,
    ) -> DdsResult<(InstanceHandle, DcpsSender)>
    where
        Auth: Authentication,
        Access: AccessControl,
        Crypto: Cryptographic,
    {
        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let listener_sender = dcps_listener.map(|l| l.spawn(spawner));

        let mut transport_participant = transport.create_participant(domain_id);

        let guid = transport_participant.guid();
        let guid_prefix = guid.prefix();
        let security_data = None;

        let mut dcps_participant_transport_writer = RtpsStatelessWriter::new(Guid::new(
            guid_prefix,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
        ));
        for &discovery_locator in transport_participant.metatraffic_multicast_locator_list() {
            dcps_participant_transport_writer.reader_locator_add(discovery_locator);
        }
        let dcps_participant_writer = DataWriterEntity::new(
            InstanceHandle::new(dcps_participant_transport_writer.guid().into()),
            dcps_participant_transport_writer,
            Arc::from(DCPS_PARTICIPANT),
            spdp_writer_qos(),
            KeyHolderType::new(&ParticipantBuiltinTopicData::TYPE),
        );

        let fragment_size = transport_participant.fragment_size();

        let dcps_topics_transport_writer = RtpsStatefulWriter::new(
            Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER),
            fragment_size,
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
            fragment_size,
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
            fragment_size,
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
            fragment_size,
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
            fragment_size,
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

        let (participant_sender, participant_receiver) = rpc_channel();

        let mut dcps_participant = DcpsDomainParticipant {
            reader_counter: 0,
            writer_counter: 0,
            publisher_counter: 0,
            subscriber_counter: 0,
            domain_participant,
            dcps_sender: participant_sender.clone(),
            domain_tag: self.configuration.domain_tag().to_string(),
            enable_type_information: self.configuration.enable_type_information(),
        };

        if self.qos.entity_factory.autoenable_created_entities {
            dcps_participant.enable_domain_participant(
                now,
                self.configuration.domain_tag().to_string(),
                &mut transport_participant,
            )?;
        }

        let run_loop = Arc::new(AtomicBool::new(true));
        let run_loop_clone = run_loop.clone();
        let participant_announcement_interval =
            self.configuration.participant_announcement_interval();
        let domain_tag = self.configuration.domain_tag().to_string();
        let mut timer_handle = timer.clone();
        let clock_clone = clock.clone();
        let spawner_clone = spawner.clone();

        let task_handle = spawner.spawn(async move {
            let span = tracing::trace_span!("dds_participant_loop");
            let _enter = span.enter();
            while run_loop_clone.load(core::sync::atomic::Ordering::Relaxed) {
                let now = clock_clone.now();
                let next_event = dcps_participant
                    .time_until_next_event(now, participant_announcement_interval.into());

                if let Some((delay, event)) = next_event {
                    match select3_future(
                        participant_receiver.receive(),
                        timer_handle.delay(delay.into()),
                        transport_participant.read(),
                    )
                    .await
                    {
                        Either3::A((user_mail, reply_sender)) => {
                            let now = clock_clone.now();
                            let is_delete = matches!(
                                user_mail,
                                DomainParticipantMail::Participant(
                                    ParticipantServiceMail::DeleteParticipant
                                )
                            );
                            let reply = dcps_participant.handle(
                                user_mail,
                                now,
                                &spawner_clone,
                                &mut transport_participant,
                            );
                            let should_terminate =
                                is_delete && matches!(&reply, DcpsReply::Ok(Ok(())));
                            reply_sender.send(reply);
                            if should_terminate {
                                break;
                            }
                        }
                        Either3::B(_) => {
                            let now = clock_clone.now();
                            dcps_participant.handle_timed_event(
                                event,
                                now,
                                participant_announcement_interval.into(),
                                &domain_tag,
                                &mut transport_participant,
                            );
                        }
                        Either3::C(data) => {
                            let now = clock_clone.now();
                            dcps_participant.handle_data(data, now);
                            dcps_participant.process_builtin_cache_changes(
                                now,
                                domain_tag.clone(),
                                &mut transport_participant,
                            );
                            dcps_participant.notify_find_topic_senders(now);
                            dcps_participant.process_user_defined_received_cache_changes(now);
                            dcps_participant
                                .request_topic_type_representation(now, &mut transport_participant);
                            dcps_participant.write_messages(now, &mut transport_participant);
                        }
                    };
                } else {
                    match select_future(
                        participant_receiver.receive(),
                        transport_participant.read(),
                    )
                    .await
                    {
                        Either::A((user_mail, reply_sender)) => {
                            let now = clock_clone.now();
                            let is_delete = matches!(
                                user_mail,
                                DomainParticipantMail::Participant(
                                    ParticipantServiceMail::DeleteParticipant
                                )
                            );
                            let reply = dcps_participant.handle(
                                user_mail,
                                now,
                                &spawner_clone,
                                &mut transport_participant,
                            );
                            let should_terminate =
                                is_delete && matches!(&reply, DcpsReply::Ok(Ok(())));
                            reply_sender.send(reply);
                            if should_terminate {
                                break;
                            }
                        }
                        Either::B(data) => {
                            let now = clock_clone.now();
                            dcps_participant.handle_data(data, now);
                            dcps_participant.process_builtin_cache_changes(
                                now,
                                domain_tag.clone(),
                                &mut transport_participant,
                            );
                            dcps_participant.notify_find_topic_senders(now);
                            dcps_participant.process_user_defined_received_cache_changes(now);
                            dcps_participant
                                .request_topic_type_representation(now, &mut transport_participant);
                            dcps_participant.write_messages(now, &mut transport_participant);
                        }
                    };
                }
            }
        });

        self.participant_list.push(ParticipantRecord {
            handle: participant_handle,
            domain_id,
            sender: participant_sender.clone(),
            run_loop,
            task_handle: Box::new(task_handle),
        });

        Ok((participant_handle, participant_sender))
    }

    pub async fn delete_participant<T: Transport>(
        &mut self,
        participant_handle: &InstanceHandle,
        transport: &mut T,
    ) -> DdsResult<()> {
        let index = self
            .participant_list
            .iter()
            .position(|r| &r.handle == participant_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let record = &self.participant_list[index];
        let reply = record
            .sender
            .call(DomainParticipantMail::Participant(
                ParticipantServiceMail::DeleteParticipant,
            ))
            .await?;
        reply.expect_ok()?;

        let record = self.participant_list.remove(index);
        record
            .run_loop
            .store(false, core::sync::atomic::Ordering::Relaxed);
        record.task_handle.join();
        let guid = Guid::from(<[u8; 16]>::from(record.handle));
        transport.delete_participant(guid);
        Ok(())
    }

    pub fn lookup_participant(&self, domain_id: DomainId) -> Option<(InstanceHandle, DcpsSender)> {
        self.participant_list
            .iter()
            .find(|r| r.domain_id == domain_id)
            .map(|r| (r.handle, r.sender.clone()))
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
