use crate::{
    dcps::{
        dcps_mail::{
            DcpsMail, MessageServiceMail, ParticipantFactoryMail, ParticipantServiceMail,
            PublisherServiceMail, ReaderServiceMail, StatusConditionMail, SubscriberServiceMail,
            TopicServiceMail, WriterServiceMail,
        },
        dcps_participant_factory::DcpsParticipantFactory,
        dcps_reply::DcpsReply,
    },
    infrastructure::error::DdsError,
    runtime::{Clock, DdsRuntime},
};

impl<R: DdsRuntime> DcpsParticipantFactory<R> {
    pub fn handle(&mut self, message: DcpsMail) -> DcpsReply {
        match message {
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::CreateParticipant {
                guid_prefix,
                domain_id,
                qos,
                dcps_listener,
                listener_mask,
                transport_participant,
                domain_tag,
                participant_announcement_interval,
            }) => DcpsReply::InstanceHandle(self.create_participant(
                guid_prefix,
                domain_id,
                qos,
                dcps_listener,
                listener_mask,
                transport_participant,
                domain_tag,
                participant_announcement_interval,
            )),
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::DeleteParticipant {
                participant_handle,
            }) => DcpsReply::Unit(self.delete_participant(&participant_handle)),
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::SetDefaultParticipantQos {
                qos,
            }) => DcpsReply::Unit(self.set_default_participant_qos(qos)),
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::GetDefaultParticipantQos {}) => {
                DcpsReply::DomainParticipantQos(self.get_default_participant_qos())
            }
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::SetQos { qos }) => {
                DcpsReply::Unit(self.set_qos(qos))
            }
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::GetQos {}) => {
                DcpsReply::DomainParticipantFactoryQos(self.get_qos())
            }
            DcpsMail::ParticipantFactory(ParticipantFactoryMail::LookupParticipant {
                domain_id,
            }) => DcpsReply::InstanceHandleOpt(
                self.domain_participant_list
                    .iter()
                    .find(|x| x.domain_id() == domain_id)
                    .map(|x| *x.get_instance_handle()),
            ),
            DcpsMail::Participant(ParticipantServiceMail::CreateUserDefinedPublisher {
                qos,
                participant_handle,
                dcps_listener,
                listener_mask,
            }) => DcpsReply::InstanceHandle(
                self.domain_participant_list
                    .iter_mut()
                    .find(|x| x.get_instance_handle() == &participant_handle)
                    .ok_or(DdsError::AlreadyDeleted)
                    .and_then(|p| {
                        p.create_user_defined_publisher(
                            qos,
                            dcps_listener,
                            listener_mask,
                            &self.runtime,
                        )
                    }),
            ),
            DcpsMail::Participant(ParticipantServiceMail::DeleteUserDefinedPublisher {
                participant_handle,
                parent_participant_handle,
                publisher_handle,
            }) => DcpsReply::Unit(self.find_participant(&participant_handle).and_then(|p| {
                p.delete_user_defined_publisher(&parent_participant_handle, &publisher_handle)
            })),
            DcpsMail::Participant(ParticipantServiceMail::CreateUserDefinedSubscriber {
                participant_handle,
                qos,
                dcps_listener,
                listener_mask,
            }) => DcpsReply::InstanceHandle(
                self.domain_participant_list
                    .iter_mut()
                    .find(|x| x.get_instance_handle() == &participant_handle)
                    .ok_or(DdsError::AlreadyDeleted)
                    .and_then(|p| {
                        p.create_user_defined_subscriber(
                            qos,
                            dcps_listener,
                            listener_mask,
                            &self.runtime,
                        )
                    }),
            ),
            DcpsMail::Participant(ParticipantServiceMail::DeleteUserDefinedSubscriber {
                participant_handle,
                parent_participant_handle,
                subscriber_handle,
            }) => DcpsReply::Unit(self.find_participant(&participant_handle).and_then(|p| {
                p.delete_user_defined_subscriber(&parent_participant_handle, &subscriber_handle)
            })),
            DcpsMail::Participant(ParticipantServiceMail::CreateTopic {
                participant_handle,
                topic_name,
                type_name,
                qos,
                dcps_listener,
                listener_mask,
                type_support,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => DcpsReply::InstanceHandle(p.create_topic(
                    topic_name,
                    type_name,
                    qos,
                    dcps_listener,
                    listener_mask,
                    type_support,
                    &self.runtime,
                )),
                Err(e) => DcpsReply::InstanceHandle(Err(e)),
            },
            DcpsMail::Participant(ParticipantServiceMail::DeleteUserDefinedTopic {
                participant_handle,
                parent_participant_handle,
                topic_name,
            }) => {
                DcpsReply::Unit(self.find_participant(&participant_handle).and_then(|p| {
                    p.delete_user_defined_topic(&parent_participant_handle, topic_name)
                }))
            }
            DcpsMail::Participant(ParticipantServiceMail::CreateContentFilteredTopic {
                participant_handle,
                name,
                related_topic_name,
                filter_expression,
                expression_parameters,
            }) => match self.find_participant(&participant_handle) {
                Ok(p) => DcpsReply::InstanceHandle(p.create_content_filtered_topic(
                    &participant_handle,
                    name,
                    related_topic_name,
                    filter_expression,
                    expression_parameters,
                )),
                Err(e) => DcpsReply::InstanceHandle(Err(e)),
            },
            DcpsMail::Participant(ParticipantServiceMail::DeleteContentFilteredTopic {
                participant_handle,
                name,
            }) => DcpsReply::Unit(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.delete_content_filtered_topic(&participant_handle, name)),
            ),
            DcpsMail::Participant(ParticipantServiceMail::FindTopic {
                participant_handle,
                topic_name,
                type_support,
                timeout,
            }) => DcpsReply::InstanceHandleAndString(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.find_topic(topic_name, type_support, timeout)),
            ),

            DcpsMail::Participant(ParticipantServiceMail::LookupTopicdescription {
                participant_handle,
                topic_name,
            }) => DcpsReply::StringOpt(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.lookup_topicdescription(topic_name)),
            ),
            DcpsMail::Participant(ParticipantServiceMail::IgnoreParticipant {
                participant_handle,
                handle,
            }) => match self.find_participant(&participant_handle) {
                Ok(p) => DcpsReply::Unit(p.ignore_participant(&handle)),
                Err(e) => DcpsReply::Unit(Err(e)),
            },
            DcpsMail::Participant(ParticipantServiceMail::IgnoreSubscription {
                participant_handle,
                handle,
            }) => DcpsReply::Unit(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.ignore_subscription(&handle)),
            ),
            DcpsMail::Participant(ParticipantServiceMail::IgnorePublication {
                participant_handle,
                handle,
            }) => DcpsReply::Unit(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.ignore_publication(&handle)),
            ),
            DcpsMail::Participant(ParticipantServiceMail::DeleteContainedEntities {
                participant_handle,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => DcpsReply::Unit(p.delete_participant_contained_entities(&self.runtime)),
                Err(e) => DcpsReply::Unit(Err(e)),
            },
            DcpsMail::Participant(ParticipantServiceMail::SetDefaultPublisherQos {
                participant_handle,
                qos,
            }) => DcpsReply::Unit(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.set_default_publisher_qos(qos)),
            ),
            DcpsMail::Participant(ParticipantServiceMail::GetDefaultPublisherQos {
                participant_handle,
            }) => DcpsReply::PublisherQos(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_default_publisher_qos()),
            ),
            DcpsMail::Participant(ParticipantServiceMail::SetDefaultSubscriberQos {
                participant_handle,
                qos,
            }) => DcpsReply::Unit(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.set_default_subscriber_qos(qos)),
            ),
            DcpsMail::Participant(ParticipantServiceMail::GetDefaultSubscriberQos {
                participant_handle,
            }) => DcpsReply::SubscriberQos(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_default_subscriber_qos()),
            ),
            DcpsMail::Participant(ParticipantServiceMail::SetDefaultTopicQos {
                participant_handle,
                qos,
            }) => DcpsReply::Unit(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.set_default_topic_qos(qos)),
            ),
            DcpsMail::Participant(ParticipantServiceMail::GetDefaultTopicQos {
                participant_handle,
            }) => DcpsReply::TopicQos(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_default_topic_qos()),
            ),
            DcpsMail::Participant(ParticipantServiceMail::GetCurrentTime {
                participant_handle,
            }) => DcpsReply::Time(
                self.domain_participant_list
                    .iter_mut()
                    .find(|x| x.get_instance_handle() == &participant_handle)
                    .ok_or(DdsError::AlreadyDeleted)
                    .map(|_| self.runtime.clock().now()),
            ),
            DcpsMail::Participant(ParticipantServiceMail::GetDiscoveredParticipants {
                participant_handle,
            }) => DcpsReply::InstanceHandleList(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_discovered_participants()),
            ),
            DcpsMail::Participant(ParticipantServiceMail::GetDiscoveredParticipantData {
                participant_handle,
                discovered_participant_handle,
            }) => DcpsReply::ParticipantBuiltinTopicData(
                self.find_participant(&participant_handle).and_then(|p| {
                    p.get_discovered_participant_data(&discovered_participant_handle)
                }),
            ),
            DcpsMail::Participant(ParticipantServiceMail::GetDiscoveredTopics {
                participant_handle,
            }) => DcpsReply::InstanceHandleList(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_discovered_topics()),
            ),
            DcpsMail::Participant(ParticipantServiceMail::GetDiscoveredTopicData {
                participant_handle,
                topic_handle,
            }) => DcpsReply::TopicBuiltinTopicData(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_discovered_topic_data(&topic_handle)),
            ),
            DcpsMail::Participant(ParticipantServiceMail::SetQos {
                participant_handle,
                qos,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => DcpsReply::Unit(p.set_domain_participant_qos(qos, &self.runtime)),
                Err(e) => DcpsReply::Unit(Err(e)),
            },
            DcpsMail::Participant(ParticipantServiceMail::GetQos { participant_handle }) => {
                DcpsReply::DomainParticipantQosResult(
                    self.find_participant(&participant_handle)
                        .and_then(|p| p.get_domain_participant_qos()),
                )
            }
            DcpsMail::Participant(ParticipantServiceMail::SetListener {
                participant_handle,
                dcps_listener,
                listener_mask,
            }) => DcpsReply::Unit(
                self.domain_participant_list
                    .iter_mut()
                    .find(|x| x.get_instance_handle() == &participant_handle)
                    .ok_or(DdsError::AlreadyDeleted)
                    .and_then(|p| {
                        p.set_domain_participant_listener(
                            dcps_listener,
                            listener_mask,
                            &self.runtime,
                        )
                    }),
            ),
            DcpsMail::Participant(ParticipantServiceMail::Enable { participant_handle }) => {
                match self
                    .domain_participant_list
                    .iter_mut()
                    .find(|x| x.get_instance_handle() == &participant_handle)
                    .ok_or(DdsError::AlreadyDeleted)
                {
                    Ok(p) => DcpsReply::Unit(p.enable_domain_participant(&self.runtime)),
                    Err(e) => DcpsReply::Unit(Err(e)),
                }
            }
            DcpsMail::Topic(TopicServiceMail::GetInconsistentTopicStatus {
                participant_handle,
                topic_name,
            }) => match self.find_participant(&participant_handle) {
                Ok(p) => {
                    DcpsReply::InconsistentTopicStatus(p.get_inconsistent_topic_status(topic_name))
                }
                Err(e) => DcpsReply::InconsistentTopicStatus(Err(e)),
            },
            DcpsMail::Topic(TopicServiceMail::SetQos {
                participant_handle,
                topic_name,
                topic_qos,
            }) => DcpsReply::Unit(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.set_topic_qos(topic_name, topic_qos)),
            ),
            DcpsMail::Topic(TopicServiceMail::GetQos {
                participant_handle,
                topic_name,
            }) => DcpsReply::TopicQos(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_topic_qos(topic_name)),
            ),
            DcpsMail::Topic(TopicServiceMail::Enable {
                participant_handle,
                topic_name,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => DcpsReply::Unit(p.enable_topic(topic_name, &self.runtime)),
                Err(e) => DcpsReply::Unit(Err(e)),
            },
            DcpsMail::Topic(TopicServiceMail::GetTypeSupport {
                participant_handle,
                topic_name,
            }) => DcpsReply::DynamicType(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_type_support(topic_name)),
            ),
            DcpsMail::Publisher(PublisherServiceMail::CreateDataWriter {
                participant_handle,
                publisher_handle,
                topic_name,
                qos,
                dcps_listener,
                listener_mask,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => DcpsReply::InstanceHandle(p.create_data_writer(
                    &publisher_handle,
                    topic_name,
                    qos,
                    dcps_listener,
                    listener_mask,
                    &self.runtime,
                )),
                Err(e) => DcpsReply::InstanceHandle(Err(e)),
            },
            DcpsMail::Publisher(PublisherServiceMail::DeleteDataWriter {
                participant_handle,
                publisher_handle,
                datawriter_handle,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => DcpsReply::Unit(p.delete_data_writer(
                    &publisher_handle,
                    &datawriter_handle,
                    &self.runtime,
                )),
                Err(e) => DcpsReply::Unit(Err(e)),
            },
            DcpsMail::Publisher(PublisherServiceMail::GetDefaultDataWriterQos {
                participant_handle,
                publisher_handle,
            }) => DcpsReply::DataWriterQos(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_default_datawriter_qos(&publisher_handle)),
            ),
            DcpsMail::Publisher(PublisherServiceMail::SetDefaultDataWriterQos {
                participant_handle,
                publisher_handle,
                qos,
            }) => DcpsReply::Unit(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.set_default_datawriter_qos(&publisher_handle, qos)),
            ),
            DcpsMail::Publisher(PublisherServiceMail::GetPublisherQos {
                participant_handle,
                publisher_handle,
            }) => DcpsReply::PublisherQos(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_publisher_qos(&publisher_handle)),
            ),
            DcpsMail::Publisher(PublisherServiceMail::SetPublisherQos {
                participant_handle,
                publisher_handle,
                qos,
            }) => DcpsReply::Unit(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.set_publisher_qos(&publisher_handle, qos)),
            ),
            DcpsMail::Publisher(PublisherServiceMail::SetPublisherListener {
                participant_handle,
                publisher_handle,
                dcps_listener,
                listener_mask,
            }) => DcpsReply::Unit(
                self.domain_participant_list
                    .iter_mut()
                    .find(|x| x.get_instance_handle() == &participant_handle)
                    .ok_or(DdsError::AlreadyDeleted)
                    .and_then(|p| {
                        p.set_publisher_listener(
                            &publisher_handle,
                            dcps_listener,
                            listener_mask,
                            &self.runtime,
                        )
                    }),
            ),
            DcpsMail::Writer(WriterServiceMail::SetListener {
                participant_handle,
                publisher_handle,
                data_writer_handle,
                dcps_listener,
                listener_mask,
            }) => DcpsReply::Unit(
                self.domain_participant_list
                    .iter_mut()
                    .find(|x| x.get_instance_handle() == &participant_handle)
                    .ok_or(DdsError::AlreadyDeleted)
                    .and_then(|p| {
                        p.set_listener_data_writer(
                            &publisher_handle,
                            &data_writer_handle,
                            dcps_listener,
                            listener_mask,
                            &self.runtime,
                        )
                    }),
            ),
            DcpsMail::Writer(WriterServiceMail::GetDataWriterQos {
                participant_handle,
                publisher_handle,
                data_writer_handle,
            }) => DcpsReply::DataWriterQos(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_data_writer_qos(&publisher_handle, &data_writer_handle)),
            ),
            DcpsMail::Writer(WriterServiceMail::GetMatchedSubscriptions {
                participant_handle,
                publisher_handle,
                data_writer_handle,
            }) => {
                DcpsReply::InstanceHandleList(self.find_participant(&participant_handle).and_then(
                    |p| p.get_matched_subscriptions(&publisher_handle, &data_writer_handle),
                ))
            }
            DcpsMail::Writer(WriterServiceMail::GetMatchedSubscriptionData {
                participant_handle,
                publisher_handle,
                data_writer_handle,
                subscription_handle,
            }) => DcpsReply::SubscriptionBuiltinTopicData(
                self.find_participant(&participant_handle).and_then(|p| {
                    p.get_matched_subscription_data(
                        &publisher_handle,
                        &data_writer_handle,
                        &subscription_handle,
                    )
                }),
            ),
            DcpsMail::Writer(WriterServiceMail::GetPublicationMatchedStatus {
                participant_handle,
                publisher_handle,
                data_writer_handle,
            }) => match self.find_participant(&participant_handle) {
                Ok(p) => DcpsReply::PublicationMatchedStatus(
                    p.get_publication_matched_status(&publisher_handle, &data_writer_handle),
                ),
                Err(e) => DcpsReply::PublicationMatchedStatus(Err(e)),
            },
            DcpsMail::Writer(WriterServiceMail::RegisterInstance {
                participant_handle,
                publisher_handle,
                data_writer_handle,
                dynamic_data,
                timestamp,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => {
                    let timestamp = timestamp.unwrap_or_else(|| p.get_current_time(&self.runtime));
                    DcpsReply::InstanceHandleDdsOpt(p.register_instance(
                        &publisher_handle,
                        &data_writer_handle,
                        &dynamic_data,
                        timestamp,
                    ))
                }
                Err(e) => DcpsReply::InstanceHandleDdsOpt(Err(e)),
            },
            DcpsMail::Writer(WriterServiceMail::UnregisterInstance {
                participant_handle,
                publisher_handle,
                data_writer_handle,
                dynamic_data,
                timestamp,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => {
                    let timestamp = timestamp.unwrap_or_else(|| p.get_current_time(&self.runtime));
                    DcpsReply::Unit(p.unregister_instance(
                        &publisher_handle,
                        &data_writer_handle,
                        &dynamic_data,
                        timestamp,
                        &self.runtime,
                    ))
                }
                Err(e) => DcpsReply::Unit(Err(e)),
            },
            DcpsMail::Writer(WriterServiceMail::LookupInstance {
                participant_handle,
                publisher_handle,
                data_writer_handle,
                dynamic_data,
            }) => DcpsReply::InstanceHandleDdsOpt(
                self.find_participant(&participant_handle).and_then(|p| {
                    p.lookup_instance(&publisher_handle, &data_writer_handle, &dynamic_data)
                }),
            ),
            DcpsMail::Writer(WriterServiceMail::WriteWTimestamp {
                participant_handle,
                publisher_handle,
                data_writer_handle,
                dynamic_data,
                timestamp,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => {
                    let timestamp = timestamp.unwrap_or_else(|| p.get_current_time(&self.runtime));
                    DcpsReply::WriteResult(p.write_w_timestamp(
                        &publisher_handle,
                        &data_writer_handle,
                        &dynamic_data,
                        timestamp,
                        &self.runtime,
                    ))
                }
                Err(e) => DcpsReply::WriteResult(Err(e)),
            },

            DcpsMail::Writer(WriterServiceMail::DisposeWTimestamp {
                participant_handle,
                publisher_handle,
                data_writer_handle,
                dynamic_data,
                timestamp,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => {
                    let timestamp = timestamp.unwrap_or_else(|| p.get_current_time(&self.runtime));
                    DcpsReply::Unit(p.dispose_w_timestamp(
                        &publisher_handle,
                        &data_writer_handle,
                        &dynamic_data,
                        timestamp,
                        &self.runtime,
                    ))
                }
                Err(e) => DcpsReply::Unit(Err(e)),
            },
            DcpsMail::Writer(WriterServiceMail::GetOfferedDeadlineMissedStatus {
                participant_handle,
                publisher_handle,
                data_writer_handle,
            }) => match self.find_participant(&participant_handle) {
                Ok(p) => DcpsReply::OfferedDeadlineMissedStatus(
                    p.get_offered_deadline_missed_status(&publisher_handle, &data_writer_handle),
                ),
                Err(e) => DcpsReply::OfferedDeadlineMissedStatus(Err(e)),
            },
            DcpsMail::Writer(WriterServiceMail::EnableDataWriter {
                participant_handle,
                publisher_handle,
                data_writer_handle,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => DcpsReply::Unit(p.enable_data_writer(
                    &publisher_handle,
                    &data_writer_handle,
                    &self.runtime,
                )),
                Err(e) => DcpsReply::Unit(Err(e)),
            },
            DcpsMail::Writer(WriterServiceMail::SetDataWriterQos {
                participant_handle,
                publisher_handle,
                data_writer_handle,
                qos,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => DcpsReply::Unit(p.set_data_writer_qos(
                    &publisher_handle,
                    &data_writer_handle,
                    qos,
                    &self.runtime,
                )),
                Err(e) => DcpsReply::Unit(Err(e)),
            },
            DcpsMail::Subscriber(SubscriberServiceMail::CreateDataReader {
                participant_handle,
                subscriber_handle,
                topic_name,
                qos,
                dcps_listener,
                listener_mask,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => DcpsReply::InstanceHandle(p.create_data_reader(
                    &subscriber_handle,
                    topic_name,
                    qos,
                    dcps_listener,
                    listener_mask,
                    &self.runtime,
                )),
                Err(e) => DcpsReply::InstanceHandle(Err(e)),
            },
            DcpsMail::Subscriber(SubscriberServiceMail::DeleteDataReader {
                participant_handle,
                subscriber_handle,
                datareader_handle,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => DcpsReply::Unit(p.delete_data_reader(
                    &subscriber_handle,
                    &datareader_handle,
                    &self.runtime,
                )),
                Err(e) => DcpsReply::Unit(Err(e)),
            },
            DcpsMail::Subscriber(SubscriberServiceMail::LookupDataReader {
                participant_handle,
                subscriber_handle,
                topic_name,
            }) => DcpsReply::InstanceHandleDdsOpt(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.lookup_data_reader(&subscriber_handle, topic_name)),
            ),
            DcpsMail::Subscriber(SubscriberServiceMail::SetDefaultDataReaderQos {
                participant_handle,
                subscriber_handle,
                qos,
            }) => DcpsReply::Unit(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.set_default_data_reader_qos(&subscriber_handle, qos)),
            ),
            DcpsMail::Subscriber(SubscriberServiceMail::GetDefaultDataReaderQos {
                participant_handle,
                subscriber_handle,
            }) => DcpsReply::DataReaderQos(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_default_data_reader_qos(&subscriber_handle)),
            ),
            DcpsMail::Subscriber(SubscriberServiceMail::SetQos {
                participant_handle,
                subscriber_handle,
                qos,
            }) => DcpsReply::Unit(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.set_subscriber_qos(&subscriber_handle, qos)),
            ),
            DcpsMail::Subscriber(SubscriberServiceMail::GetSubscriberQos {
                participant_handle,
                subscriber_handle,
            }) => DcpsReply::SubscriberQos(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_subscriber_qos(&subscriber_handle)),
            ),
            DcpsMail::Subscriber(SubscriberServiceMail::SetListener {
                participant_handle,
                subscriber_handle,
                dcps_listener,
                listener_mask,
            }) => DcpsReply::Unit(
                self.domain_participant_list
                    .iter_mut()
                    .find(|x| x.get_instance_handle() == &participant_handle)
                    .ok_or(DdsError::AlreadyDeleted)
                    .and_then(|p| {
                        p.set_subscriber_listener(
                            &subscriber_handle,
                            dcps_listener,
                            listener_mask,
                            &self.runtime,
                        )
                    }),
            ),
            DcpsMail::Reader(ReaderServiceMail::Read {
                participant_handle,
                subscriber_handle,
                data_reader_handle,
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
            }) => match self.find_participant(&participant_handle) {
                Ok(p) => DcpsReply::DynamicDataSampleList(p.read(
                    &subscriber_handle,
                    &data_reader_handle,
                    max_samples,
                    &sample_states,
                    &view_states,
                    &instance_states,
                    &specific_instance_handle,
                )),
                Err(e) => DcpsReply::DynamicDataSampleList(Err(e)),
            },
            DcpsMail::Reader(ReaderServiceMail::Take {
                participant_handle,
                subscriber_handle,
                data_reader_handle,
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
            }) => match self.find_participant(&participant_handle) {
                Ok(p) => DcpsReply::DynamicDataSampleList(p.take(
                    &subscriber_handle,
                    &data_reader_handle,
                    max_samples,
                    &sample_states,
                    &view_states,
                    &instance_states,
                    &specific_instance_handle,
                )),
                Err(e) => DcpsReply::DynamicDataSampleList(Err(e)),
            },
            DcpsMail::Reader(ReaderServiceMail::ReadNextInstance {
                participant_handle,
                subscriber_handle,
                data_reader_handle,
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            }) => match self.find_participant(&participant_handle) {
                Ok(p) => DcpsReply::DynamicDataSampleList(p.read_next_instance(
                    &subscriber_handle,
                    &data_reader_handle,
                    max_samples,
                    &previous_handle,
                    &sample_states,
                    &view_states,
                    &instance_states,
                )),
                Err(e) => DcpsReply::DynamicDataSampleList(Err(e)),
            },
            DcpsMail::Reader(ReaderServiceMail::TakeNextInstance {
                participant_handle,
                subscriber_handle,
                data_reader_handle,
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            }) => match self.find_participant(&participant_handle) {
                Ok(p) => DcpsReply::DynamicDataSampleList(p.take_next_instance(
                    &subscriber_handle,
                    &data_reader_handle,
                    max_samples,
                    &previous_handle,
                    &sample_states,
                    &view_states,
                    &instance_states,
                )),
                Err(e) => DcpsReply::DynamicDataSampleList(Err(e)),
            },
            DcpsMail::Reader(ReaderServiceMail::Enable {
                participant_handle,
                subscriber_handle,
                data_reader_handle,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => DcpsReply::Unit(p.enable_data_reader(
                    &subscriber_handle,
                    &data_reader_handle,
                    &self.runtime,
                )),
                Err(e) => DcpsReply::Unit(Err(e)),
            },
            DcpsMail::Reader(ReaderServiceMail::GetSubscriptionMatchedStatus {
                participant_handle,
                subscriber_handle,
                data_reader_handle,
            }) => match self.find_participant(&participant_handle) {
                Ok(p) => DcpsReply::SubscriptionMatchedStatus(
                    p.get_subscription_matched_status(&subscriber_handle, &data_reader_handle),
                ),
                Err(e) => DcpsReply::SubscriptionMatchedStatus(Err(e)),
            },
            DcpsMail::Reader(ReaderServiceMail::GetMatchedPublicationData {
                participant_handle,
                subscriber_handle,
                data_reader_handle,
                publication_handle,
            }) => DcpsReply::PublicationBuiltinTopicData(
                self.find_participant(&participant_handle).and_then(|p| {
                    p.get_matched_publication_data(
                        &subscriber_handle,
                        &data_reader_handle,
                        &publication_handle,
                    )
                }),
            ),
            DcpsMail::Reader(ReaderServiceMail::GetMatchedPublications {
                participant_handle,
                subscriber_handle,
                data_reader_handle,
            }) => {
                DcpsReply::InstanceHandleList(self.find_participant(&participant_handle).and_then(
                    |p| p.get_matched_publications(&subscriber_handle, &data_reader_handle),
                ))
            }
            DcpsMail::Reader(ReaderServiceMail::GetQos {
                participant_handle,
                subscriber_handle,
                data_reader_handle,
            }) => DcpsReply::DataReaderQos(
                self.find_participant(&participant_handle)
                    .and_then(|p| p.get_data_reader_qos(&subscriber_handle, &data_reader_handle)),
            ),
            DcpsMail::Reader(ReaderServiceMail::SetQos {
                participant_handle,
                subscriber_handle,
                data_reader_handle,
                qos,
            }) => match self
                .domain_participant_list
                .iter_mut()
                .find(|x| x.get_instance_handle() == &participant_handle)
                .ok_or(DdsError::AlreadyDeleted)
            {
                Ok(p) => DcpsReply::Unit(p.set_data_reader_qos(
                    &subscriber_handle,
                    &data_reader_handle,
                    qos,
                    &self.runtime,
                )),
                Err(e) => DcpsReply::Unit(Err(e)),
            },
            DcpsMail::Reader(ReaderServiceMail::SetListener {
                participant_handle,
                subscriber_handle,
                data_reader_handle,
                dcps_listener,
                listener_mask,
            }) => DcpsReply::Unit(
                self.domain_participant_list
                    .iter_mut()
                    .find(|x| x.get_instance_handle() == &participant_handle)
                    .ok_or(DdsError::AlreadyDeleted)
                    .and_then(|p| {
                        p.set_data_reader_listener(
                            &subscriber_handle,
                            &data_reader_handle,
                            dcps_listener,
                            listener_mask,
                            &self.runtime,
                        )
                    }),
            ),
            DcpsMail::StatusCondition(StatusConditionMail::GetStatusConditionEnabledStatuses {
                entity,
            }) => DcpsReply::StatusMask(self.get_status_condition_enabled_statuses(entity)),
            DcpsMail::StatusCondition(StatusConditionMail::GetStatusConditionTriggerValue {
                entity,
            }) => DcpsReply::Bool(self.get_status_condition_trigger_value(entity)),
            DcpsMail::StatusCondition(StatusConditionMail::RegisterNotification {
                entity,
                notification_sender,
            }) => DcpsReply::Unit(self.register_notification(entity, notification_sender)),
            DcpsMail::StatusCondition(StatusConditionMail::SetStatusConditionEnabledStatuses {
                entity,
                status_mask,
            }) => DcpsReply::Unit(self.set_status_condition_enabled_statuses(entity, status_mask)),
            DcpsMail::Message(MessageServiceMail::NotifyAcknowledgments {
                participant_handle,
                publisher_handle,
                data_writer_handle,
                notification_sender,
            }) => DcpsReply::Unit(self.find_participant(&participant_handle).and_then(|p| {
                p.notify_acknowledgments(
                    &publisher_handle,
                    &data_writer_handle,
                    notification_sender,
                )
            })),
            DcpsMail::Message(MessageServiceMail::NotifyHistoricalData {
                participant_handle,
                subscriber_handle,
                data_reader_handle,
                notification_sender,
            }) => DcpsReply::Unit(self.find_participant(&participant_handle).and_then(|p| {
                p.notify_historical_data(
                    &subscriber_handle,
                    &data_reader_handle,
                    notification_sender,
                )
            })),
            DcpsMail::Message(MessageServiceMail::HandleData {
                participant_handle,
                data_message,
            }) => {
                if let Ok(p) = self
                    .domain_participant_list
                    .iter_mut()
                    .find(|x| x.get_instance_handle() == &participant_handle)
                    .ok_or(DdsError::AlreadyDeleted)
                {
                    p.handle_data(data_message.as_slice(), &self.runtime);
                }
                DcpsReply::None
            }
        }
    }
}
