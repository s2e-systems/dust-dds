use alloc::boxed::Box;

use crate::{
    dcps::{
        dcps_domain_participant::participant_entity::DcpsDomainParticipant,
        dcps_mail::{
            DcpsReply, DomainParticipantFactoryMail, DomainParticipantMail, MessageServiceMail,
            ParticipantServiceMail, PublisherServiceMail, ReaderServiceMail, StatusConditionMail,
            SubscriberServiceMail, TopicServiceMail, WriterServiceMail,
        },
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
            DomainParticipantMail::Participant(mail) => match mail {
                ParticipantServiceMail::DeleteParticipant => {
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
                ParticipantServiceMail::CreateUserDefinedPublisher {
                    qos,
                    participant_handle: _,
                    dcps_listener,
                    listener_mask,
                } => DcpsReply::InstanceHandle(self.create_user_defined_publisher(
                    qos,
                    dcps_listener,
                    listener_mask,
                    spawner,
                )),
                ParticipantServiceMail::DeleteUserDefinedPublisher {
                    participant_handle: _,
                    parent_participant_handle,
                    publisher_handle,
                } => {
                    DcpsReply::Ok(self.delete_user_defined_publisher(
                        &parent_participant_handle,
                        &publisher_handle,
                    ))
                }
                ParticipantServiceMail::CreateUserDefinedSubscriber {
                    participant_handle: _,
                    qos,
                    dcps_listener,
                    listener_mask,
                } => DcpsReply::InstanceHandle(self.create_user_defined_subscriber(
                    qos,
                    dcps_listener,
                    listener_mask,
                    spawner,
                )),
                ParticipantServiceMail::DeleteUserDefinedSubscriber {
                    participant_handle: _,
                    parent_participant_handle,
                    subscriber_handle,
                } => DcpsReply::Ok(self.delete_user_defined_subscriber(
                    &parent_participant_handle,
                    &subscriber_handle,
                )),
                ParticipantServiceMail::CreateTopic(p) => {
                    DcpsReply::InstanceHandle(self.create_topic(
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
                    ))
                }
                ParticipantServiceMail::DeleteUserDefinedTopic {
                    participant_handle,
                    parent_participant_handle: _,
                    topic_name,
                } => DcpsReply::Ok(self.delete_user_defined_topic(&participant_handle, topic_name)),
                ParticipantServiceMail::CreateContentFilteredTopic(p) => {
                    DcpsReply::InstanceHandle(self.create_content_filtered_topic(
                        &p.participant_handle,
                        p.name,
                        p.related_topic_name,
                        p.filter_expression,
                        p.expression_parameters,
                    ))
                }
                ParticipantServiceMail::DeleteContentFilteredTopic {
                    participant_handle,
                    name,
                } => DcpsReply::Ok(self.delete_content_filtered_topic(&participant_handle, name)),
                ParticipantServiceMail::FindTopic {
                    participant_handle: _,
                    topic_name,
                    type_support,
                    timeout,
                    reply_sender,
                } => {
                    self.find_topic(topic_name, type_support, timeout, now, reply_sender);
                    DcpsReply::Ok(Ok(()))
                }
                ParticipantServiceMail::LookupTopicdescription {
                    participant_handle: _,
                    topic_name,
                } => DcpsReply::TopicDescription(self.lookup_topicdescription(topic_name)),
                ParticipantServiceMail::IgnoreParticipant {
                    participant_handle: _,
                    handle,
                } => DcpsReply::Ok(self.ignore_participant(&handle, now)),
                ParticipantServiceMail::IgnoreSubscription {
                    participant_handle: _,
                    handle,
                } => DcpsReply::Ok(self.ignore_subscription(&handle)),
                ParticipantServiceMail::IgnorePublication {
                    participant_handle: _,
                    handle,
                } => DcpsReply::Ok(self.ignore_publication(&handle)),
                ParticipantServiceMail::DeleteContainedEntities {
                    participant_handle: _,
                } => DcpsReply::Ok(
                    self.delete_participant_contained_entities(now, transport_participant),
                ),
                ParticipantServiceMail::SetDefaultPublisherQos {
                    participant_handle: _,
                    qos,
                } => DcpsReply::Ok(self.set_default_publisher_qos(*qos)),
                ParticipantServiceMail::GetDefaultPublisherQos {
                    participant_handle: _,
                } => DcpsReply::PublisherQos(self.get_default_publisher_qos()),
                ParticipantServiceMail::SetDefaultSubscriberQos {
                    participant_handle: _,
                    qos,
                } => DcpsReply::Ok(self.set_default_subscriber_qos(*qos)),
                ParticipantServiceMail::GetDefaultSubscriberQos {
                    participant_handle: _,
                } => DcpsReply::SubscriberQos(self.get_default_subscriber_qos()),
                ParticipantServiceMail::SetDefaultTopicQos {
                    participant_handle: _,
                    qos,
                } => DcpsReply::Ok(self.set_default_topic_qos(*qos)),
                ParticipantServiceMail::GetDefaultTopicQos {
                    participant_handle: _,
                } => DcpsReply::TopicQos(self.get_default_topic_qos().map(Box::new)),
                ParticipantServiceMail::GetCurrentTime {
                    participant_handle: _,
                } => DcpsReply::Time(Ok(now)),
                ParticipantServiceMail::GetDiscoveredParticipants {
                    participant_handle: _,
                } => DcpsReply::InstanceHandleList(self.get_discovered_participants()),
                ParticipantServiceMail::GetDiscoveredParticipantData {
                    participant_handle: _,
                    discovered_participant_handle,
                } => DcpsReply::ParticipantBuiltinTopicData(
                    self.get_discovered_participant_data(&discovered_participant_handle)
                        .map(Box::new),
                ),
                ParticipantServiceMail::GetDiscoveredTopics {
                    participant_handle: _,
                } => DcpsReply::InstanceHandleList(self.get_discovered_topics()),
                ParticipantServiceMail::GetDiscoveredTopicData {
                    participant_handle: _,
                    topic_handle,
                } => DcpsReply::TopicBuiltinTopicData(
                    self.get_discovered_topic_data(&topic_handle).map(Box::new),
                ),
                ParticipantServiceMail::SetQos {
                    participant_handle: _,
                    qos,
                } => DcpsReply::Ok(self.set_domain_participant_qos(
                    *qos,
                    now,
                    self.domain_tag.clone(),
                    transport_participant,
                )),
                ParticipantServiceMail::GetQos {
                    participant_handle: _,
                } => DcpsReply::ParticipantQos(self.get_domain_participant_qos()),
                ParticipantServiceMail::SetListener {
                    participant_handle: _,
                    dcps_listener,
                    listener_mask,
                } => DcpsReply::Ok(self.set_domain_participant_listener(
                    dcps_listener,
                    listener_mask,
                    spawner,
                )),
                ParticipantServiceMail::Enable {
                    participant_handle: _,
                } => DcpsReply::Ok(self.enable_domain_participant(
                    now,
                    self.domain_tag.clone(),
                    transport_participant,
                )),
            },
            DomainParticipantMail::Topic(mail) => match mail {
                TopicServiceMail::GetInconsistentTopicStatus {
                    participant_handle: _,
                    topic_name,
                } => DcpsReply::InconsistentTopicStatus(
                    self.get_inconsistent_topic_status(topic_name),
                ),
                TopicServiceMail::SetQos {
                    participant_handle: _,
                    topic_name,
                    topic_qos,
                } => DcpsReply::Ok(self.set_topic_qos(topic_name, *topic_qos)),
                TopicServiceMail::GetQos {
                    participant_handle: _,
                    topic_name,
                } => DcpsReply::TopicQos(self.get_topic_qos(topic_name).map(Box::new)),
                TopicServiceMail::Enable {
                    participant_handle: _,
                    topic_name,
                } => DcpsReply::Ok(self.enable_topic(
                    topic_name,
                    now,
                    self.enable_type_information,
                    transport_participant,
                )),
                TopicServiceMail::GetTypeSupport {
                    participant_handle: _,
                    topic_name,
                } => DcpsReply::DynamicType(self.get_type_support(topic_name)),
            },
            DomainParticipantMail::Publisher(mail) => match mail {
                PublisherServiceMail::CreateDataWriter(p) => {
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
                PublisherServiceMail::DeleteDataWriter {
                    participant_handle: _,
                    publisher_handle,
                    datawriter_handle,
                } => DcpsReply::Ok(self.delete_data_writer(
                    &publisher_handle,
                    &datawriter_handle,
                    now,
                    transport_participant,
                )),
                PublisherServiceMail::GetDefaultDataWriterQos {
                    participant_handle: _,
                    publisher_handle,
                } => DcpsReply::DataWriterQos(
                    self.get_default_datawriter_qos(&publisher_handle)
                        .map(Box::new),
                ),
                PublisherServiceMail::SetDefaultDataWriterQos {
                    participant_handle: _,
                    publisher_handle,
                    qos,
                } => DcpsReply::Ok(self.set_default_datawriter_qos(&publisher_handle, *qos)),
                PublisherServiceMail::GetPublisherQos {
                    participant_handle: _,
                    publisher_handle,
                } => DcpsReply::PublisherQos(self.get_publisher_qos(&publisher_handle)),
                PublisherServiceMail::SetPublisherQos {
                    participant_handle: _,
                    publisher_handle,
                    qos,
                } => DcpsReply::Ok(self.set_publisher_qos(&publisher_handle, *qos)),
                PublisherServiceMail::SetPublisherListener {
                    participant_handle: _,
                    publisher_handle,
                    dcps_listener,
                    listener_mask,
                } => DcpsReply::Ok(self.set_publisher_listener(
                    &publisher_handle,
                    dcps_listener,
                    listener_mask,
                    spawner,
                )),
            },
            DomainParticipantMail::Writer(mail) => match mail {
                WriterServiceMail::SetListener {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                    dcps_listener,
                    listener_mask,
                } => DcpsReply::Ok(self.set_listener_data_writer(
                    &publisher_handle,
                    &data_writer_handle,
                    dcps_listener,
                    listener_mask,
                    spawner,
                )),
                WriterServiceMail::GetDataWriterQos {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                } => DcpsReply::DataWriterQos(
                    self.get_data_writer_qos(&publisher_handle, &data_writer_handle)
                        .map(Box::new),
                ),
                WriterServiceMail::GetMatchedSubscriptions {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                } => DcpsReply::InstanceHandleList(
                    self.get_matched_subscriptions(&publisher_handle, &data_writer_handle),
                ),
                WriterServiceMail::GetMatchedSubscriptionData {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                    subscription_handle,
                } => DcpsReply::SubscriptionBuiltinTopicData(
                    self.get_matched_subscription_data(
                        &publisher_handle,
                        &data_writer_handle,
                        &subscription_handle,
                    )
                    .map(Box::new),
                ),
                WriterServiceMail::GetPublicationMatchedStatus {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                } => DcpsReply::PublicationMatchedStatus(
                    self.get_publication_matched_status(&publisher_handle, &data_writer_handle),
                ),
                WriterServiceMail::RegisterInstance {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                    dynamic_data,
                    timestamp,
                } => {
                    let timestamp = timestamp.unwrap_or(now);
                    DcpsReply::OptionInstanceHandle(self.register_instance(
                        &publisher_handle,
                        &data_writer_handle,
                        &dynamic_data,
                        timestamp,
                    ))
                }
                WriterServiceMail::UnregisterInstance {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                    dynamic_data,
                    timestamp,
                } => {
                    let timestamp = timestamp.unwrap_or(now);
                    DcpsReply::Ok(self.unregister_instance(
                        &publisher_handle,
                        &data_writer_handle,
                        &dynamic_data,
                        timestamp,
                        now,
                        transport_participant,
                    ))
                }
                WriterServiceMail::LookupInstance {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                    dynamic_data,
                } => DcpsReply::OptionInstanceHandle(self.lookup_instance(
                    &publisher_handle,
                    &data_writer_handle,
                    &dynamic_data,
                )),
                WriterServiceMail::WriteWTimestamp {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                    dynamic_data,
                    timestamp,
                } => {
                    let timestamp = timestamp.unwrap_or(now);
                    match self.write_w_timestamp(
                        &publisher_handle,
                        &data_writer_handle,
                        &dynamic_data,
                        timestamp,
                        now,
                        transport_participant,
                    ) {
                        Ok(None) => DcpsReply::Ok(Ok(())),
                        Ok(Some(receiver)) => DcpsReply::PendingWrite(receiver),
                        Err(e) => DcpsReply::Ok(Err(e)),
                    }
                }
                WriterServiceMail::DisposeWTimestamp {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                    dynamic_data,
                    timestamp,
                } => {
                    let timestamp = timestamp.unwrap_or(now);
                    DcpsReply::Ok(self.dispose_w_timestamp(
                        &publisher_handle,
                        &data_writer_handle,
                        &dynamic_data,
                        timestamp,
                        now,
                        transport_participant,
                    ))
                }
                WriterServiceMail::GetOfferedDeadlineMissedStatus {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                } => DcpsReply::OfferedDeadlineMissedStatus(
                    self.get_offered_deadline_missed_status(&publisher_handle, &data_writer_handle),
                ),
                WriterServiceMail::EnableDataWriter {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                } => DcpsReply::Ok(self.enable_data_writer(
                    &publisher_handle,
                    &data_writer_handle,
                    now,
                    self.enable_type_information,
                    transport_participant,
                )),
                WriterServiceMail::SetDataWriterQos {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                    qos,
                } => DcpsReply::Ok(self.set_data_writer_qos(
                    &publisher_handle,
                    &data_writer_handle,
                    *qos,
                    now,
                    self.enable_type_information,
                    transport_participant,
                )),
            },
            DomainParticipantMail::Subscriber(mail) => match mail {
                SubscriberServiceMail::CreateDataReader(p) => {
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
                SubscriberServiceMail::DeleteDataReader {
                    participant_handle: _,
                    subscriber_handle,
                    datareader_handle,
                } => DcpsReply::Ok(self.delete_data_reader(
                    &subscriber_handle,
                    &datareader_handle,
                    now,
                    transport_participant,
                )),
                SubscriberServiceMail::LookupDataReader {
                    participant_handle: _,
                    subscriber_handle,
                    topic_name,
                } => DcpsReply::OptionInstanceHandle(
                    self.lookup_data_reader(&subscriber_handle, topic_name),
                ),
                SubscriberServiceMail::SetDefaultDataReaderQos {
                    participant_handle: _,
                    subscriber_handle,
                    qos,
                } => DcpsReply::Ok(self.set_default_data_reader_qos(&subscriber_handle, *qos)),
                SubscriberServiceMail::GetDefaultDataReaderQos {
                    participant_handle: _,
                    subscriber_handle,
                } => DcpsReply::DataReaderQos(
                    self.get_default_data_reader_qos(&subscriber_handle)
                        .map(Box::new),
                ),
                SubscriberServiceMail::SetQos {
                    participant_handle: _,
                    subscriber_handle,
                    qos,
                } => DcpsReply::Ok(self.set_subscriber_qos(&subscriber_handle, *qos)),
                SubscriberServiceMail::GetSubscriberQos {
                    participant_handle: _,
                    subscriber_handle,
                } => DcpsReply::SubscriberQos(self.get_subscriber_qos(&subscriber_handle)),
                SubscriberServiceMail::SetListener {
                    participant_handle: _,
                    subscriber_handle,
                    dcps_listener,
                    listener_mask,
                } => DcpsReply::Ok(self.set_subscriber_listener(
                    &subscriber_handle,
                    dcps_listener,
                    listener_mask,
                    spawner,
                )),
            },
            DomainParticipantMail::Reader(mail) => match mail {
                ReaderServiceMail::Enable {
                    participant_handle: _,
                    subscriber_handle,
                    data_reader_handle,
                } => DcpsReply::Ok(self.enable_data_reader(
                    &subscriber_handle,
                    &data_reader_handle,
                    now,
                    self.enable_type_information,
                    transport_participant,
                )),
                ReaderServiceMail::Read(p) => DcpsReply::Samples(self.read(
                    &p.subscriber_handle,
                    &p.data_reader_handle,
                    p.max_samples,
                    p.sample_state_mask,
                    p.view_state_mask,
                    p.instance_state_mask,
                    &p.specific_instance_handle,
                )),
                ReaderServiceMail::Take(p) => DcpsReply::Samples(self.take(
                    &p.subscriber_handle,
                    &p.data_reader_handle,
                    p.max_samples,
                    p.sample_state_mask,
                    p.view_state_mask,
                    p.instance_state_mask,
                    &p.specific_instance_handle,
                )),
                ReaderServiceMail::ReadNextInstance(p) => {
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
                ReaderServiceMail::TakeNextInstance(p) => {
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
                ReaderServiceMail::GetSubscriptionMatchedStatus {
                    participant_handle: _,
                    subscriber_handle,
                    data_reader_handle,
                } => DcpsReply::SubscriptionMatchedStatus(
                    self.get_subscription_matched_status(&subscriber_handle, &data_reader_handle),
                ),
                ReaderServiceMail::GetMatchedPublicationData {
                    participant_handle: _,
                    subscriber_handle,
                    data_reader_handle,
                    publication_handle,
                } => DcpsReply::PublicationBuiltinTopicData(
                    self.get_matched_publication_data(
                        &subscriber_handle,
                        &data_reader_handle,
                        &publication_handle,
                    )
                    .map(Box::new),
                ),
                ReaderServiceMail::GetMatchedPublications {
                    participant_handle: _,
                    subscriber_handle,
                    data_reader_handle,
                } => DcpsReply::InstanceHandleList(
                    self.get_matched_publications(&subscriber_handle, &data_reader_handle),
                ),
                ReaderServiceMail::SetQos {
                    participant_handle: _,
                    subscriber_handle,
                    data_reader_handle,
                    qos,
                } => DcpsReply::Ok(self.set_data_reader_qos(
                    &subscriber_handle,
                    &data_reader_handle,
                    *qos,
                    now,
                    self.enable_type_information,
                    transport_participant,
                )),
                ReaderServiceMail::GetQos {
                    participant_handle: _,
                    subscriber_handle,
                    data_reader_handle,
                } => DcpsReply::DataReaderQos(
                    self.get_data_reader_qos(&subscriber_handle, &data_reader_handle)
                        .map(Box::new),
                ),
                ReaderServiceMail::SetListener {
                    participant_handle: _,
                    subscriber_handle,
                    data_reader_handle,
                    dcps_listener,
                    listener_mask,
                } => DcpsReply::Ok(self.set_data_reader_listener(
                    &subscriber_handle,
                    &data_reader_handle,
                    dcps_listener,
                    listener_mask,
                    spawner,
                )),
            },
            DomainParticipantMail::StatusCondition(mail) => match mail {
                StatusConditionMail::GetStatusConditionEnabledStatuses { entity } => {
                    DcpsReply::StatusMask(self.get_status_condition_enabled_statuses(entity))
                }
                StatusConditionMail::GetStatusConditionTriggerValue { entity } => {
                    DcpsReply::TriggerValue(self.get_status_condition_trigger_value(entity))
                }
                StatusConditionMail::RegisterNotification {
                    entity,
                    notification_sender,
                } => DcpsReply::Ok(self.register_notification(entity, notification_sender)),
                StatusConditionMail::SetStatusConditionEnabledStatuses {
                    entity,
                    status_mask,
                } => DcpsReply::Ok(self.set_status_condition_enabled_statuses(entity, status_mask)),
            },
            DomainParticipantMail::Message(mail) => match mail {
                MessageServiceMail::NotifyAcknowledgments {
                    participant_handle: _,
                    publisher_handle,
                    data_writer_handle,
                    reply_sender,
                } => {
                    self.notify_acknowledgments(
                        &publisher_handle,
                        &data_writer_handle,
                        reply_sender,
                    );
                    DcpsReply::Ok(Ok(()))
                }
                MessageServiceMail::NotifyHistoricalData {
                    participant_handle: _,
                    subscriber_handle,
                    data_reader_handle,
                    reply_sender,
                } => {
                    self.notify_historical_data(
                        &subscriber_handle,
                        &data_reader_handle,
                        reply_sender,
                    );
                    DcpsReply::Ok(Ok(()))
                }
            },
        }
    }
}
