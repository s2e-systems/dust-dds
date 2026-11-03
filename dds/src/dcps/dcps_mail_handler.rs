use alloc::boxed::Box;

use crate::{
    dcps::{
        dcps_domain_participant::participant_entity::DcpsDomainParticipant,
        dcps_mail::{DcpsReply, DomainParticipantFactoryMail, DomainParticipantMail},
        dcps_participant_factory::DcpsParticipantFactory,
    },
    infrastructure::{error::DdsError, time::Time},
    runtime::Spawner,
    security::plugins::{
        access_control::AccessControl, authentication::Authentication,
        cryptographic::Cryptographic, types::DdsSecurityPlugins,
    },
    transport::interface::RtpsParticipant,
};

impl DcpsParticipantFactory {
    #[allow(clippy::too_many_arguments)]
    pub async fn handle_factory_mail<
        Auth,
        Access,
        Crypto,
        T: crate::transport::interface::Transport,
        Timer: crate::runtime::Timer + Clone + Send + 'static,
        Clock: crate::runtime::Clock + Clone + Send + 'static,
        Spawner: crate::runtime::Spawner + Clone + Send + 'static,
    >(
        &mut self,
        message: DomainParticipantFactoryMail,
        now: Time,
        timer: &Timer,
        clock: &Clock,
        spawner: &Spawner,
        security: &mut Option<DdsSecurityPlugins<Auth, Access, Crypto>>,
        transport: &mut T,
    ) -> DcpsReply
    where
        Auth: Authentication,
        Access: AccessControl,
        Crypto: Cryptographic,
    {
        match message {
            DomainParticipantFactoryMail::CreateParticipant(p) => {
                DcpsReply::CreatedParticipant(self.create_participant(
                    p.domain_id,
                    p.qos,
                    p.dcps_listener,
                    p.listener_mask,
                    transport,
                    now,
                    timer,
                    clock,
                    spawner,
                    security,
                ))
            }
            DomainParticipantFactoryMail::DeleteParticipant { participant_handle } => {
                DcpsReply::Ok(
                    self.delete_participant(&participant_handle, transport)
                        .await,
                )
            }
            DomainParticipantFactoryMail::SetDefaultParticipantQos { qos } => {
                DcpsReply::Ok(self.set_default_participant_qos(*qos))
            }
            DomainParticipantFactoryMail::GetDefaultParticipantQos => {
                DcpsReply::ParticipantQos(Ok(self.get_default_participant_qos()))
            }
            DomainParticipantFactoryMail::SetQos { qos } => DcpsReply::Ok(self.set_qos(*qos)),
            DomainParticipantFactoryMail::GetQos => DcpsReply::FactoryQos(Ok(self.get_qos())),
            DomainParticipantFactoryMail::LookupParticipant { domain_id } => {
                DcpsReply::OptionCreatedParticipant(Ok(self.lookup_participant(domain_id)))
            }
        }
    }
}

impl DcpsDomainParticipant {
    pub fn handle(
        &mut self,
        message: DomainParticipantMail,
        now: Time,
        spawner: &impl Spawner,
        transport_participant: &mut impl RtpsParticipant,
    ) -> DcpsReply {
        match message {
            // Participant
            DomainParticipantMail::DeleteParticipant => {
                if !self.is_participant_empty() {
                    DcpsReply::Ok(Err(DdsError::PreconditionNotMet(
                        alloc::string::String::from(
                            "Domain participant still contains other entities",
                        ),
                    )))
                } else {
                    self.announce_deleted_participant(now, transport_participant);
                    DcpsReply::Ok(Ok(()))
                }
            }
            DomainParticipantMail::CreateUserDefinedPublisher(p) => {
                DcpsReply::InstanceHandle(self.create_user_defined_publisher(
                    p.qos,
                    p.dcps_listener,
                    p.listener_mask,
                    spawner,
                ))
            }
            DomainParticipantMail::DeleteUserDefinedPublisher(p) => {
                DcpsReply::Ok(self.delete_user_defined_publisher(
                    &p.parent_participant_handle,
                    &p.publisher_handle,
                ))
            }
            DomainParticipantMail::CreateUserDefinedSubscriber(p) => {
                DcpsReply::InstanceHandle(self.create_user_defined_subscriber(
                    p.qos,
                    p.dcps_listener,
                    p.listener_mask,
                    spawner,
                ))
            }
            DomainParticipantMail::DeleteUserDefinedSubscriber(p) => {
                DcpsReply::Ok(self.delete_user_defined_subscriber(
                    &p.parent_participant_handle,
                    &p.subscriber_handle,
                ))
            }
            DomainParticipantMail::CreateTopic(p) => DcpsReply::InstanceHandle(self.create_topic(
                p.topic_name,
                p.type_name,
                p.qos,
                p.dcps_listener,
                p.listener_mask,
                p.type_support,
                spawner,
                now,
                self.enable_type_information,
                transport_participant,
            )),
            DomainParticipantMail::DeleteUserDefinedTopic(p) => {
                DcpsReply::Ok(self.delete_user_defined_topic(&p.participant_handle, p.topic_name))
            }
            DomainParticipantMail::CreateContentFilteredTopic(p) => {
                DcpsReply::InstanceHandle(self.create_content_filtered_topic(
                    &p.participant_handle,
                    p.name,
                    p.related_topic_name,
                    p.filter_expression,
                    p.expression_parameters,
                ))
            }
            DomainParticipantMail::DeleteContentFilteredTopic(p) => {
                DcpsReply::Ok(self.delete_content_filtered_topic(&p.participant_handle, p.name))
            }
            DomainParticipantMail::FindTopic(p) => {
                self.find_topic(p.topic_name, p.type_support, p.timeout, now, p.reply_sender);
                DcpsReply::Ok(Ok(()))
            }
            DomainParticipantMail::LookupTopicdescription(p) => {
                DcpsReply::TopicDescription(self.lookup_topicdescription(p.topic_name))
            }
            DomainParticipantMail::IgnoreParticipant(p) => {
                DcpsReply::Ok(self.ignore_participant(&p.handle, now))
            }
            DomainParticipantMail::IgnoreSubscription(p) => {
                DcpsReply::Ok(self.ignore_subscription(&p.handle))
            }
            DomainParticipantMail::IgnorePublication(p) => {
                DcpsReply::Ok(self.ignore_publication(&p.handle))
            }
            DomainParticipantMail::DeleteContainedEntities(_) => DcpsReply::Ok(
                self.delete_participant_contained_entities(now, transport_participant),
            ),
            DomainParticipantMail::SetDefaultPublisherQos(p) => {
                DcpsReply::Ok(self.set_default_publisher_qos(*p.qos))
            }
            DomainParticipantMail::GetDefaultPublisherQos(_) => {
                DcpsReply::PublisherQos(self.get_default_publisher_qos())
            }
            DomainParticipantMail::SetDefaultSubscriberQos(p) => {
                DcpsReply::Ok(self.set_default_subscriber_qos(*p.qos))
            }
            DomainParticipantMail::GetDefaultSubscriberQos(_) => {
                DcpsReply::SubscriberQos(self.get_default_subscriber_qos())
            }
            DomainParticipantMail::SetDefaultTopicQos(p) => {
                DcpsReply::Ok(self.set_default_topic_qos(*p.qos))
            }
            DomainParticipantMail::GetDefaultTopicQos(_) => {
                DcpsReply::TopicQos(self.get_default_topic_qos().map(Box::new))
            }
            DomainParticipantMail::GetCurrentTime(_) => DcpsReply::Time(Ok(now)),
            DomainParticipantMail::GetDiscoveredParticipants(_) => {
                DcpsReply::InstanceHandleList(self.get_discovered_participants())
            }
            DomainParticipantMail::GetDiscoveredParticipantData(p) => {
                DcpsReply::ParticipantBuiltinTopicData(
                    self.get_discovered_participant_data(&p.discovered_participant_handle)
                        .map(Box::new),
                )
            }
            DomainParticipantMail::GetDiscoveredTopics(_) => {
                DcpsReply::InstanceHandleList(self.get_discovered_topics())
            }
            DomainParticipantMail::GetDiscoveredTopicData(p) => DcpsReply::TopicBuiltinTopicData(
                self.get_discovered_topic_data(&p.topic_handle)
                    .map(Box::new),
            ),
            DomainParticipantMail::SetParticipantQos(p) => {
                DcpsReply::Ok(self.set_domain_participant_qos(
                    *p.qos,
                    now,
                    self.domain_tag.clone(),
                    transport_participant,
                ))
            }
            DomainParticipantMail::GetParticipantQos(_) => {
                DcpsReply::ParticipantQos(self.get_domain_participant_qos())
            }
            DomainParticipantMail::SetParticipantListener(p) => DcpsReply::Ok(
                self.set_domain_participant_listener(p.dcps_listener, p.listener_mask, spawner),
            ),
            DomainParticipantMail::EnableParticipant(_) => DcpsReply::Ok(
                self.enable_domain_participant(now, self.domain_tag.clone(), transport_participant),
            ),

            // Topic
            DomainParticipantMail::GetInconsistentTopicStatus(p) => {
                DcpsReply::InconsistentTopicStatus(self.get_inconsistent_topic_status(p.topic_name))
            }
            DomainParticipantMail::SetTopicQos(p) => {
                DcpsReply::Ok(self.set_topic_qos(p.topic_name, *p.topic_qos))
            }
            DomainParticipantMail::GetTopicQos(p) => {
                DcpsReply::TopicQos(self.get_topic_qos(p.topic_name).map(Box::new))
            }
            DomainParticipantMail::EnableTopic(p) => DcpsReply::Ok(self.enable_topic(
                p.topic_name,
                now,
                self.enable_type_information,
                transport_participant,
            )),
            DomainParticipantMail::GetTopicTypeSupport(p) => {
                DcpsReply::DynamicType(self.get_type_support(p.topic_name))
            }

            // Publisher
            DomainParticipantMail::CreateDataWriter(p) => {
                DcpsReply::InstanceHandle(self.create_data_writer(
                    &p.publisher_handle,
                    p.topic_name,
                    p.qos,
                    p.dcps_listener,
                    p.listener_mask,
                    spawner,
                    now,
                    self.enable_type_information,
                    transport_participant,
                ))
            }
            DomainParticipantMail::DeleteDataWriter(p) => DcpsReply::Ok(self.delete_data_writer(
                &p.publisher_handle,
                &p.datawriter_handle,
                now,
                transport_participant,
            )),
            DomainParticipantMail::GetDefaultDataWriterQos(p) => DcpsReply::DataWriterQos(
                self.get_default_datawriter_qos(&p.publisher_handle)
                    .map(Box::new),
            ),
            DomainParticipantMail::SetDefaultDataWriterQos(p) => {
                DcpsReply::Ok(self.set_default_datawriter_qos(&p.publisher_handle, *p.qos))
            }
            DomainParticipantMail::GetPublisherQos(p) => {
                DcpsReply::PublisherQos(self.get_publisher_qos(&p.publisher_handle))
            }
            DomainParticipantMail::SetPublisherQos(p) => {
                DcpsReply::Ok(self.set_publisher_qos(&p.publisher_handle, *p.qos))
            }
            DomainParticipantMail::SetPublisherListener(p) => {
                DcpsReply::Ok(self.set_publisher_listener(
                    &p.publisher_handle,
                    p.dcps_listener,
                    p.listener_mask,
                    spawner,
                ))
            }

            // Subscriber
            DomainParticipantMail::CreateDataReader(p) => {
                DcpsReply::InstanceHandle(self.create_data_reader(
                    &p.subscriber_handle,
                    p.topic_name,
                    p.qos,
                    p.dcps_listener,
                    p.listener_mask,
                    spawner,
                    now,
                    self.enable_type_information,
                    transport_participant,
                ))
            }
            DomainParticipantMail::DeleteDataReader(p) => DcpsReply::Ok(self.delete_data_reader(
                &p.subscriber_handle,
                &p.datareader_handle,
                now,
                transport_participant,
            )),
            DomainParticipantMail::LookupDataReader(p) => DcpsReply::OptionInstanceHandle(
                self.lookup_data_reader(&p.subscriber_handle, p.topic_name),
            ),
            DomainParticipantMail::SetDefaultDataReaderQos(p) => {
                DcpsReply::Ok(self.set_default_data_reader_qos(&p.subscriber_handle, *p.qos))
            }
            DomainParticipantMail::GetDefaultDataReaderQos(p) => DcpsReply::DataReaderQos(
                self.get_default_data_reader_qos(&p.subscriber_handle)
                    .map(Box::new),
            ),
            DomainParticipantMail::SetSubscriberQos(p) => {
                DcpsReply::Ok(self.set_subscriber_qos(&p.subscriber_handle, *p.qos))
            }
            DomainParticipantMail::GetSubscriberQos(p) => {
                DcpsReply::SubscriberQos(self.get_subscriber_qos(&p.subscriber_handle))
            }
            DomainParticipantMail::SetSubscriberListener(p) => {
                DcpsReply::Ok(self.set_subscriber_listener(
                    &p.subscriber_handle,
                    p.dcps_listener,
                    p.listener_mask,
                    spawner,
                ))
            }

            // Writer
            DomainParticipantMail::SetDataWriterListener(m) => {
                DcpsReply::Ok(self.set_listener_data_writer(
                    &m.publisher_handle,
                    &m.data_writer_handle,
                    m.dcps_listener,
                    m.listener_mask,
                    spawner,
                ))
            }
            DomainParticipantMail::GetDataWriterQos(m) => DcpsReply::DataWriterQos(
                self.get_data_writer_qos(&m.publisher_handle, &m.data_writer_handle)
                    .map(Box::new),
            ),
            DomainParticipantMail::GetMatchedSubscriptions(m) => DcpsReply::InstanceHandleList(
                self.get_matched_subscriptions(&m.publisher_handle, &m.data_writer_handle),
            ),
            DomainParticipantMail::GetMatchedSubscriptionData(m) => {
                DcpsReply::SubscriptionBuiltinTopicData(
                    self.get_matched_subscription_data(
                        &m.publisher_handle,
                        &m.data_writer_handle,
                        &m.subscription_handle,
                    )
                    .map(Box::new),
                )
            }
            DomainParticipantMail::GetPublicationMatchedStatus(m) => {
                DcpsReply::PublicationMatchedStatus(
                    self.get_publication_matched_status(&m.publisher_handle, &m.data_writer_handle),
                )
            }
            DomainParticipantMail::RegisterInstance(m) => {
                let timestamp = m.timestamp.unwrap_or(now);
                DcpsReply::OptionInstanceHandle(self.register_instance(
                    &m.publisher_handle,
                    &m.data_writer_handle,
                    &m.dynamic_data,
                    timestamp,
                ))
            }
            DomainParticipantMail::UnregisterInstance(m) => {
                let timestamp = m.timestamp.unwrap_or(now);
                DcpsReply::Ok(self.unregister_instance(
                    &m.publisher_handle,
                    &m.data_writer_handle,
                    &m.dynamic_data,
                    timestamp,
                    now,
                    transport_participant,
                ))
            }
            DomainParticipantMail::LookupInstance(m) => DcpsReply::OptionInstanceHandle(
                self.lookup_instance(&m.publisher_handle, &m.data_writer_handle, &m.dynamic_data),
            ),
            DomainParticipantMail::WriteWTimestamp(m) => {
                let timestamp = m.timestamp.unwrap_or(now);
                match self.write_w_timestamp(
                    &m.publisher_handle,
                    &m.data_writer_handle,
                    &m.dynamic_data,
                    timestamp,
                    now,
                    transport_participant,
                ) {
                    Ok(None) => DcpsReply::Ok(Ok(())),
                    Ok(Some(receiver)) => DcpsReply::PendingWrite(receiver),
                    Err(e) => DcpsReply::Ok(Err(e)),
                }
            }
            DomainParticipantMail::DisposeWTimestamp(m) => {
                let timestamp = m.timestamp.unwrap_or(now);
                DcpsReply::Ok(self.dispose_w_timestamp(
                    &m.publisher_handle,
                    &m.data_writer_handle,
                    &m.dynamic_data,
                    timestamp,
                    now,
                    transport_participant,
                ))
            }
            DomainParticipantMail::GetOfferedDeadlineMissedStatus(m) => {
                DcpsReply::OfferedDeadlineMissedStatus(
                    self.get_offered_deadline_missed_status(
                        &m.publisher_handle,
                        &m.data_writer_handle,
                    ),
                )
            }
            DomainParticipantMail::EnableDataWriter(m) => DcpsReply::Ok(self.enable_data_writer(
                &m.publisher_handle,
                &m.data_writer_handle,
                now,
                self.enable_type_information,
                transport_participant,
            )),
            DomainParticipantMail::SetDataWriterQos(m) => DcpsReply::Ok(self.set_data_writer_qos(
                &m.publisher_handle,
                &m.data_writer_handle,
                *m.qos,
                now,
                self.enable_type_information,
                transport_participant,
            )),

            // Reader
            DomainParticipantMail::EnableDataReader(m) => DcpsReply::Ok(self.enable_data_reader(
                &m.subscriber_handle,
                &m.data_reader_handle,
                now,
                self.enable_type_information,
                transport_participant,
            )),
            DomainParticipantMail::Read(p) => DcpsReply::Samples(self.read(
                &p.subscriber_handle,
                &p.data_reader_handle,
                p.max_samples,
                p.sample_state_mask,
                p.view_state_mask,
                p.instance_state_mask,
                &p.specific_instance_handle,
            )),
            DomainParticipantMail::Take(p) => DcpsReply::Samples(self.take(
                &p.subscriber_handle,
                &p.data_reader_handle,
                p.max_samples,
                p.sample_state_mask,
                p.view_state_mask,
                p.instance_state_mask,
                &p.specific_instance_handle,
            )),
            DomainParticipantMail::ReadNextInstance(p) => {
                DcpsReply::Samples(self.read_next_instance(
                    &p.subscriber_handle,
                    &p.data_reader_handle,
                    p.max_samples,
                    &p.previous_handle,
                    p.sample_state_mask,
                    p.view_state_mask,
                    p.instance_state_mask,
                ))
            }
            DomainParticipantMail::TakeNextInstance(p) => {
                DcpsReply::Samples(self.take_next_instance(
                    &p.subscriber_handle,
                    &p.data_reader_handle,
                    p.max_samples,
                    &p.previous_handle,
                    p.sample_state_mask,
                    p.view_state_mask,
                    p.instance_state_mask,
                ))
            }
            DomainParticipantMail::GetSubscriptionMatchedStatus(m) => {
                DcpsReply::SubscriptionMatchedStatus(
                    self.get_subscription_matched_status(
                        &m.subscriber_handle,
                        &m.data_reader_handle,
                    ),
                )
            }
            DomainParticipantMail::GetMatchedPublicationData(m) => {
                DcpsReply::PublicationBuiltinTopicData(
                    self.get_matched_publication_data(
                        &m.subscriber_handle,
                        &m.data_reader_handle,
                        &m.publication_handle,
                    )
                    .map(Box::new),
                )
            }
            DomainParticipantMail::GetMatchedPublications(m) => DcpsReply::InstanceHandleList(
                self.get_matched_publications(&m.subscriber_handle, &m.data_reader_handle),
            ),
            DomainParticipantMail::SetDataReaderQos(m) => DcpsReply::Ok(self.set_data_reader_qos(
                &m.subscriber_handle,
                &m.data_reader_handle,
                *m.qos,
                now,
                self.enable_type_information,
                transport_participant,
            )),
            DomainParticipantMail::GetDataReaderQos(m) => DcpsReply::DataReaderQos(
                self.get_data_reader_qos(&m.subscriber_handle, &m.data_reader_handle)
                    .map(Box::new),
            ),
            DomainParticipantMail::SetDataReaderListener(m) => {
                DcpsReply::Ok(self.set_data_reader_listener(
                    &m.subscriber_handle,
                    &m.data_reader_handle,
                    m.dcps_listener,
                    m.listener_mask,
                    spawner,
                ))
            }

            // StatusCondition
            DomainParticipantMail::GetStatusConditionEnabledStatuses(m) => {
                DcpsReply::StatusMask(self.get_status_condition_enabled_statuses(m.entity))
            }
            DomainParticipantMail::GetStatusConditionTriggerValue(m) => {
                DcpsReply::TriggerValue(self.get_status_condition_trigger_value(m.entity))
            }
            DomainParticipantMail::RegisterStatusConditionNotification(m) => {
                DcpsReply::Ok(self.register_notification(m.entity, m.notification_sender))
            }
            DomainParticipantMail::SetStatusConditionEnabledStatuses(m) => {
                DcpsReply::Ok(self.set_status_condition_enabled_statuses(m.entity, m.status_mask))
            }

            // Message
            DomainParticipantMail::NotifyAcknowledgments(m) => {
                self.notify_acknowledgments(
                    &m.publisher_handle,
                    &m.data_writer_handle,
                    m.reply_sender,
                );
                DcpsReply::Ok(Ok(()))
            }
            DomainParticipantMail::NotifyHistoricalData(m) => {
                self.notify_historical_data(
                    &m.subscriber_handle,
                    &m.data_reader_handle,
                    m.reply_sender,
                );
                DcpsReply::Ok(Ok(()))
            }
        }
    }
}
