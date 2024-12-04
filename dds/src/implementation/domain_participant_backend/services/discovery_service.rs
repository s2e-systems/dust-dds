use fnmatch_regex::glob_to_regex;

use crate::{
    builtin_topics::{
        BuiltInTopicKey, ParticipantBuiltinTopicData, PublicationBuiltinTopicData,
        SubscriptionBuiltinTopicData, TopicBuiltinTopicData, DCPS_PARTICIPANT, DCPS_PUBLICATION,
        DCPS_SUBSCRIPTION, DCPS_TOPIC,
    },
    implementation::{
        data_representation_builtin_endpoints::discovered_writer_data::DiscoveredWriterData,
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            entities::{
                data_reader::{DataReaderEntity, TransportReaderKind},
                data_writer::DataWriterEntity,
            },
        },
        listeners::{
            data_reader_listener, data_writer_listener, domain_participant_listener,
            publisher_listener, subscriber_listener,
        },
        status_condition::status_condition_actor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, SubscriberQos, TopicQos},
        qos_policy::{
            QosPolicyId, DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
            RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
        status::StatusKind,
    },
    runtime::actor::{ActorAddress, Mail, MailHandler},
    topic_definition::type_support::DdsSerialize,
    transport::reader::WriterProxy,
};

pub struct AnnounceParticipant;
impl Mail for AnnounceParticipant {
    type Result = DdsResult<()>;
}
impl MailHandler<AnnounceParticipant> for DomainParticipantActor {
    fn handle(&mut self, _: AnnounceParticipant) -> <AnnounceParticipant as Mail>::Result {
        if self.domain_participant.enabled() {
            let participant_builtin_topic_data = ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.transport.guid().into(),
                },
                user_data: self.domain_participant.qos().user_data.clone(),
            };
            let timestamp = self.domain_participant.get_current_time();

            if let Some(dw) = self
                .domain_participant
                .builtin_publisher_mut()
                .lookup_datawriter_mut(DCPS_PARTICIPANT)
            {
                dw.write_w_timestamp(participant_builtin_topic_data.serialize_data()?, timestamp)?;
            }
        }

        Ok(())
    }
}

pub struct AnnounceDeletedParticipant;
impl Mail for AnnounceDeletedParticipant {
    type Result = DdsResult<()>;
}
impl MailHandler<AnnounceDeletedParticipant> for DomainParticipantActor {
    fn handle(
        &mut self,
        _: AnnounceDeletedParticipant,
    ) -> <AnnounceDeletedParticipant as Mail>::Result {
        if self.domain_participant.enabled() {
            let timestamp = self.domain_participant.get_current_time();
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher_mut()
                .lookup_datawriter_mut(DCPS_PARTICIPANT)
            {
                let key = InstanceHandle::new(self.transport.guid().into());
                dw.dispose_w_timestamp(key.serialize_data()?, timestamp)?;
            }
        }

        Ok(())
    }
}

pub struct AnnounceDataWriter {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for AnnounceDataWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<AnnounceDataWriter> for DomainParticipantActor {
    fn handle(&mut self, message: AnnounceDataWriter) -> <AnnounceDataWriter as Mail>::Result {
        let publisher = self
            .domain_participant
            .get_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .get_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let topic_data = self
            .domain_participant
            .get_topic(data_writer.topic_name())
            .ok_or(DdsError::Error(
                "Internal error. Data writer exists without associated topic".to_owned(),
            ))?
            .qos()
            .topic_data
            .clone();

        let publication_builtin_topic_data = PublicationBuiltinTopicData {
            key: BuiltInTopicKey {
                value: data_writer.transport_writer().guid().into(),
            },
            participant_key: BuiltInTopicKey { value: [0; 16] },
            topic_name: data_writer.topic_name().to_owned(),
            type_name: data_writer.type_name().to_owned(),
            durability: data_writer.qos().durability.clone(),
            deadline: data_writer.qos().deadline.clone(),
            latency_budget: data_writer.qos().latency_budget.clone(),
            liveliness: data_writer.qos().liveliness.clone(),
            reliability: data_writer.qos().reliability.clone(),
            lifespan: data_writer.qos().lifespan.clone(),
            user_data: data_writer.qos().user_data.clone(),
            ownership: data_writer.qos().ownership.clone(),
            ownership_strength: data_writer.qos().ownership_strength.clone(),
            destination_order: data_writer.qos().destination_order.clone(),
            presentation: publisher.qos().presentation.clone(),
            partition: publisher.qos().partition.clone(),
            topic_data,
            group_data: publisher.qos().group_data.clone(),
            representation: data_writer.qos().representation.clone(),
        };
        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_PUBLICATION)
        {
            dw.write_w_timestamp(publication_builtin_topic_data.serialize_data()?, timestamp)?;
        }
        Ok(())
    }
}

pub struct AnnounceDeletedDataWriter {
    pub data_writer: DataWriterEntity,
}
impl Mail for AnnounceDeletedDataWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<AnnounceDeletedDataWriter> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AnnounceDeletedDataWriter,
    ) -> <AnnounceDeletedDataWriter as Mail>::Result {
        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_PUBLICATION)
        {
            let key = InstanceHandle::new(message.data_writer.transport_writer().guid().into());
            dw.dispose_w_timestamp(key.serialize_data()?, timestamp)?;
        }
        Ok(())
    }
}

pub struct AnnounceDataReader {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for AnnounceDataReader {
    type Result = DdsResult<()>;
}
impl MailHandler<AnnounceDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: AnnounceDataReader) -> <AnnounceDataReader as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let topic = self
            .domain_participant
            .get_topic(data_reader.topic_name())
            .ok_or(DdsError::Error(
                "Internal error. Data reader exists without associated topic".to_owned(),
            ))?;

        let guid = match data_reader.transport_reader() {
            TransportReaderKind::Stateful(r) => r.guid(),
            TransportReaderKind::Stateless(r) => r.guid(),
        };
        let subscription_builtin_topic_data = SubscriptionBuiltinTopicData {
            key: BuiltInTopicKey { value: guid.into() },
            participant_key: BuiltInTopicKey { value: [0; 16] },
            topic_name: data_reader.topic_name().to_owned(),
            type_name: data_reader.type_name().to_owned(),
            durability: data_reader.qos().durability.clone(),
            deadline: data_reader.qos().deadline.clone(),
            latency_budget: data_reader.qos().latency_budget.clone(),
            liveliness: data_reader.qos().liveliness.clone(),
            reliability: data_reader.qos().reliability.clone(),
            ownership: data_reader.qos().ownership.clone(),
            destination_order: data_reader.qos().destination_order.clone(),
            user_data: data_reader.qos().user_data.clone(),
            time_based_filter: data_reader.qos().time_based_filter.clone(),
            presentation: subscriber.qos().presentation.clone(),
            partition: subscriber.qos().partition.clone(),
            topic_data: topic.qos().topic_data.clone(),
            group_data: subscriber.qos().group_data.clone(),
            representation: data_reader.qos().representation.clone(),
        };
        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_SUBSCRIPTION)
        {
            dw.write_w_timestamp(subscription_builtin_topic_data.serialize_data()?, timestamp)?;
        }
        Ok(())
    }
}

pub struct AnnounceTopic {
    pub topic_name: String,
}
impl Mail for AnnounceTopic {
    type Result = DdsResult<()>;
}
impl MailHandler<AnnounceTopic> for DomainParticipantActor {
    fn handle(&mut self, message: AnnounceTopic) -> <AnnounceTopic as Mail>::Result {
        let topic = self
            .domain_participant
            .get_topic(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;

        let topic_builtin_topic_data = TopicBuiltinTopicData {
            key: BuiltInTopicKey {
                value: topic.instance_handle().into(),
            },
            name: topic.topic_name().to_owned(),
            type_name: topic.type_name().to_owned(),
            durability: topic.qos().durability.clone(),
            deadline: topic.qos().deadline.clone(),
            latency_budget: topic.qos().latency_budget.clone(),
            liveliness: topic.qos().liveliness.clone(),
            reliability: topic.qos().reliability.clone(),
            transport_priority: topic.qos().transport_priority.clone(),
            lifespan: topic.qos().lifespan.clone(),
            destination_order: topic.qos().destination_order.clone(),
            history: topic.qos().history.clone(),
            resource_limits: topic.qos().resource_limits.clone(),
            ownership: topic.qos().ownership.clone(),
            topic_data: topic.qos().topic_data.clone(),
            representation: topic.qos().representation.clone(),
        };

        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_TOPIC)
        {
            dw.write_w_timestamp(topic_builtin_topic_data.serialize_data()?, timestamp)?;
        }
        Ok(())
    }
}

pub struct AddDiscoveredTopic {
    pub topic_builtin_topic_data: TopicBuiltinTopicData,
    pub topic_name: String,
}
impl Mail for AddDiscoveredTopic {
    type Result = DdsResult<()>;
}
impl MailHandler<AddDiscoveredTopic> for DomainParticipantActor {
    fn handle(&mut self, message: AddDiscoveredTopic) -> <AddDiscoveredTopic as Mail>::Result {
        let topic = self
            .domain_participant
            .get_mut_topic(&message.topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;
        if topic.topic_name() == message.topic_builtin_topic_data.name()
            && topic.type_name() == message.topic_builtin_topic_data.get_type_name()
            && !is_discovered_topic_consistent(topic.qos(), &message.topic_builtin_topic_data)
        {
            topic.increment_inconsistent_topic_status();
        }
        Ok(())
    }
}
pub struct AnnounceDeletedDataReader {
    pub data_reader: DataReaderEntity,
}
impl Mail for AnnounceDeletedDataReader {
    type Result = DdsResult<()>;
}
impl MailHandler<AnnounceDeletedDataReader> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AnnounceDeletedDataReader,
    ) -> <AnnounceDeletedDataReader as Mail>::Result {
        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_SUBSCRIPTION)
        {
            let guid = match message.data_reader.transport_reader() {
                TransportReaderKind::Stateful(r) => r.guid(),
                TransportReaderKind::Stateless(r) => r.guid(),
            };
            let key = InstanceHandle::new(guid.into());
            dw.dispose_w_timestamp(key.serialize_data()?, timestamp)?;
        }
        Ok(())
    }
}

pub struct AddDiscoveredReader {
    pub subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for AddDiscoveredReader {
    type Result = DdsResult<()>;
}
impl MailHandler<AddDiscoveredReader> for DomainParticipantActor {
    fn handle(&mut self, message: AddDiscoveredReader) -> <AddDiscoveredReader as Mail>::Result {
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let is_any_name_matched = message
            .subscription_builtin_topic_data
            .partition
            .name
            .iter()
            .any(|n| publisher.qos().partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = message
            .subscription_builtin_topic_data
            .partition
            .name
            .iter()
            .filter_map(|n| glob_to_regex(n).ok())
            .any(|regex| {
                publisher
                    .qos()
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_any_local_regex_matched_with_received_partition_qos = publisher
            .qos()
            .partition
            .name
            .iter()
            .filter_map(|n| glob_to_regex(n).ok())
            .any(|regex| {
                message
                    .subscription_builtin_topic_data
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_partition_matched = message.subscription_builtin_topic_data.partition
            == publisher.qos().partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos;
        if is_partition_matched {
            let publisher_qos = publisher.qos().clone();
            let data_writer = publisher
                .get_mut_data_writer(message.data_writer_handle)
                .ok_or(DdsError::AlreadyDeleted)?;

            let is_matched_topic_name =
                message.subscription_builtin_topic_data.topic_name() == data_writer.topic_name();
            let is_matched_type_name =
                message.subscription_builtin_topic_data.get_type_name() == data_writer.type_name();

            if is_matched_topic_name && is_matched_type_name {
                let incompatible_qos_policy_list =
                    get_discovered_reader_incompatible_qos_policy_list(
                        data_writer.qos(),
                        &message.subscription_builtin_topic_data,
                        &publisher_qos,
                    );
                if incompatible_qos_policy_list.is_empty() {
                    data_writer
                        .add_matched_subscription(message.subscription_builtin_topic_data.clone());

                    if data_writer
                        .listener_mask()
                        .contains(&StatusKind::PublicationMatched)
                    {
                        let status = data_writer.get_publication_matched_status();
                        let the_writer = self.get_data_writer_async(
                            message.participant_address,
                            message.publisher_handle,
                            message.data_writer_handle,
                        )?;
                        if let Some(l) = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_mut_data_writer(message.data_writer_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .listener()
                        {
                            l.send_actor_mail(data_writer_listener::TriggerPublicationMatched {
                                the_writer,
                                status,
                            });
                        }
                    } else if self
                        .domain_participant
                        .get_mut_publisher(message.publisher_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .listener_mask()
                        .contains(&StatusKind::PublicationMatched)
                    {
                        let the_writer = self.get_data_writer_async(
                            message.participant_address,
                            message.publisher_handle,
                            message.data_writer_handle,
                        )?;
                        let status = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_mut_data_writer(message.data_writer_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_publication_matched_status();
                        if let Some(l) = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .listener()
                        {
                            l.send_actor_mail(publisher_listener::TriggerOnPublicationMatched {
                                the_writer,
                                status,
                            });
                        }
                    } else if self
                        .domain_participant
                        .listener_mask()
                        .contains(&StatusKind::PublicationMatched)
                    {
                        let the_writer = self.get_data_writer_async(
                            message.participant_address,
                            message.publisher_handle,
                            message.data_writer_handle,
                        )?;
                        let status = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_mut_data_writer(message.data_writer_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_publication_matched_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(
                                domain_participant_listener::TriggerPublicationMatched {
                                    the_writer,
                                    status,
                                },
                            );
                        }
                    }

                    self.domain_participant
                        .get_mut_publisher(message.publisher_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .get_mut_data_writer(message.data_writer_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .status_condition()
                        .send_actor_mail(status_condition_actor::AddCommunicationState {
                            state: StatusKind::PublicationMatched,
                        });
                } else {
                    data_writer.add_incompatible_subscription(
                        InstanceHandle::new(message.subscription_builtin_topic_data.key().value),
                        incompatible_qos_policy_list,
                    );

                    if data_writer
                        .listener_mask()
                        .contains(&StatusKind::OfferedIncompatibleQos)
                    {
                        let status = data_writer.get_offered_incompatible_qos_status();
                        let the_writer = self.get_data_writer_async(
                            message.participant_address,
                            message.publisher_handle,
                            message.data_writer_handle,
                        )?;
                        if let Some(l) = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_mut_data_writer(message.data_writer_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .listener()
                        {
                            l.send_actor_mail(
                                data_writer_listener::TriggerOfferedIncompatibleQos {
                                    the_writer,
                                    status,
                                },
                            );
                        }
                    } else if self
                        .domain_participant
                        .get_mut_publisher(message.publisher_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .listener_mask()
                        .contains(&StatusKind::OfferedIncompatibleQos)
                    {
                        let the_writer = self.get_data_writer_async(
                            message.participant_address,
                            message.publisher_handle,
                            message.data_writer_handle,
                        )?;
                        let status = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_mut_data_writer(message.data_writer_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_offered_incompatible_qos_status();
                        if let Some(l) = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .listener()
                        {
                            l.send_actor_mail(publisher_listener::TriggerOfferedIncompatibleQos {
                                the_writer,
                                status,
                            });
                        }
                    } else if self
                        .domain_participant
                        .listener_mask()
                        .contains(&StatusKind::OfferedIncompatibleQos)
                    {
                        let the_writer = self.get_data_writer_async(
                            message.participant_address,
                            message.publisher_handle,
                            message.data_writer_handle,
                        )?;
                        let status = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_mut_data_writer(message.data_writer_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_offered_incompatible_qos_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(
                                domain_participant_listener::TriggerOfferedIncompatibleQos {
                                    the_writer,
                                    status,
                                },
                            );
                        }
                    }

                    self.domain_participant
                        .get_mut_publisher(message.publisher_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .get_mut_data_writer(message.data_writer_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .status_condition()
                        .send_actor_mail(status_condition_actor::AddCommunicationState {
                            state: StatusKind::OfferedIncompatibleQos,
                        });
                }
            }
        }
        Ok(())
    }
}

pub struct RemoveDiscoveredReader {
    pub subscription_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for RemoveDiscoveredReader {
    type Result = DdsResult<()>;
}
impl MailHandler<RemoveDiscoveredReader> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: RemoveDiscoveredReader,
    ) -> <RemoveDiscoveredReader as Mail>::Result {
        let publisher = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_writer = publisher
            .get_mut_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        if data_writer
            .get_matched_subscription_data(&message.subscription_handle)
            .is_some()
        {
            data_writer.remove_matched_subscription(&message.subscription_handle);

            data_writer.status_condition().send_actor_mail(
                status_condition_actor::AddCommunicationState {
                    state: StatusKind::PublicationMatched,
                },
            );
        }
        Ok(())
    }
}

pub struct AddDiscoveredWriter {
    pub discovered_writer_data: DiscoveredWriterData,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for AddDiscoveredWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<AddDiscoveredWriter> for DomainParticipantActor {
    fn handle(&mut self, message: AddDiscoveredWriter) -> <AddDiscoveredWriter as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let is_any_name_matched = message
            .discovered_writer_data
            .dds_publication_data
            .partition
            .name
            .iter()
            .any(|n| subscriber.qos().partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = message
            .discovered_writer_data
            .dds_publication_data
            .partition
            .name
            .iter()
            .filter_map(|n| glob_to_regex(n).ok())
            .any(|regex| {
                subscriber
                    .qos()
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_any_local_regex_matched_with_received_partition_qos = subscriber
            .qos()
            .partition
            .name
            .iter()
            .filter_map(|n| glob_to_regex(n).ok())
            .any(|regex| {
                message
                    .discovered_writer_data
                    .dds_publication_data
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_partition_matched = message
            .discovered_writer_data
            .dds_publication_data
            .partition
            == subscriber.qos().partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos;
        if is_partition_matched {
            let subscriber_qos = subscriber.qos().clone();
            let data_reader = subscriber
                .get_mut_data_reader(message.data_reader_handle)
                .ok_or(DdsError::AlreadyDeleted)?;
            let is_matched_topic_name = message
                .discovered_writer_data
                .dds_publication_data
                .topic_name()
                == data_reader.topic_name();
            let is_matched_type_name = message
                .discovered_writer_data
                .dds_publication_data
                .get_type_name()
                == data_reader.type_name();

            if is_matched_topic_name && is_matched_type_name {
                let incompatible_qos_policy_list =
                    get_discovered_writer_incompatible_qos_policy_list(
                        data_reader,
                        &message.discovered_writer_data.dds_publication_data,
                        &subscriber_qos,
                    );
                if incompatible_qos_policy_list.is_empty() {
                    data_reader.add_matched_publication(
                        message.discovered_writer_data.dds_publication_data.clone(),
                    );
                    let writer_proxy = WriterProxy {
                        remote_group_entity_id: todo!(),
                        unicast_locator_list: todo!(),
                        multicast_locator_list: todo!(),
                        data_max_size_serialized: todo!(),
                        remote_writer_guid: todo!(),
                        reliability_kind: todo!(),
                        durability_kind: todo!(),
                    };
                    if let TransportReaderKind::Stateful(r) = data_reader.transport_reader_mut() {
                        r.add_matched_writer(writer_proxy);
                    }

                    if data_reader
                        .listener_mask()
                        .contains(&StatusKind::SubscriptionMatched)
                    {
                        let status = data_reader.get_subscription_matched_status();
                        let the_reader = self.get_data_reader_async(
                            message.participant_address,
                            message.subscriber_handle,
                            message.data_reader_handle,
                        )?;
                        if let Some(l) = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_mut_data_reader(message.data_reader_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .listener()
                        {
                            l.send_actor_mail(data_reader_listener::TriggerSubscriptionMatched {
                                the_reader,
                                status,
                            });
                        }
                    } else if self
                        .domain_participant
                        .get_mut_subscriber(message.subscriber_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .listener_mask()
                        .contains(&StatusKind::SubscriptionMatched)
                    {
                        let the_reader = self.get_data_reader_async(
                            message.participant_address,
                            message.subscriber_handle,
                            message.data_reader_handle,
                        )?;
                        let status = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_mut_data_reader(message.data_reader_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_subscription_matched_status();
                        if let Some(l) = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .listener()
                        {
                            l.send_actor_mail(subscriber_listener::TriggerSubscriptionMatched {
                                the_reader,
                                status,
                            });
                        }
                    } else if self
                        .domain_participant
                        .listener_mask()
                        .contains(&StatusKind::SubscriptionMatched)
                    {
                        let the_reader = self.get_data_reader_async(
                            message.participant_address,
                            message.subscriber_handle,
                            message.data_reader_handle,
                        )?;
                        let status = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_mut_data_reader(message.data_reader_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_subscription_matched_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(
                                domain_participant_listener::TriggerSubscriptionMatched {
                                    the_reader,
                                    status,
                                },
                            );
                        }
                    }

                    self.domain_participant
                        .get_mut_subscriber(message.subscriber_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .get_mut_data_reader(message.data_reader_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .status_condition()
                        .send_actor_mail(status_condition_actor::AddCommunicationState {
                            state: StatusKind::SubscriptionMatched,
                        });
                } else {
                    data_reader.add_requested_incompatible_qos(
                        InstanceHandle::new(
                            message
                                .discovered_writer_data
                                .dds_publication_data
                                .key()
                                .value,
                        ),
                        incompatible_qos_policy_list,
                    );

                    if data_reader
                        .listener_mask()
                        .contains(&StatusKind::RequestedIncompatibleQos)
                    {
                        let status = data_reader.get_requested_incompatible_qos_status();
                        let the_reader = self.get_data_reader_async(
                            message.participant_address,
                            message.subscriber_handle,
                            message.data_reader_handle,
                        )?;
                        if let Some(l) = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_mut_data_reader(message.data_reader_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .listener()
                        {
                            l.send_actor_mail(
                                data_reader_listener::TriggerRequestedIncompatibleQos {
                                    the_reader,
                                    status,
                                },
                            );
                        }
                    } else if self
                        .domain_participant
                        .get_mut_subscriber(message.subscriber_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .listener_mask()
                        .contains(&StatusKind::RequestedIncompatibleQos)
                    {
                        let the_reader = self.get_data_reader_async(
                            message.participant_address,
                            message.subscriber_handle,
                            message.data_reader_handle,
                        )?;
                        let status = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_mut_data_reader(message.data_reader_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_requested_incompatible_qos_status();
                        if let Some(l) = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .listener()
                        {
                            l.send_actor_mail(
                                subscriber_listener::TriggerRequestedIncompatibleQos {
                                    the_reader,
                                    status,
                                },
                            );
                        }
                    } else if self
                        .domain_participant
                        .listener_mask()
                        .contains(&StatusKind::RequestedIncompatibleQos)
                    {
                        let the_reader = self.get_data_reader_async(
                            message.participant_address,
                            message.subscriber_handle,
                            message.data_reader_handle,
                        )?;
                        let status = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_mut_data_reader(message.data_reader_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .get_requested_incompatible_qos_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(
                                domain_participant_listener::TriggerRequestedIncompatibleQos {
                                    the_reader,
                                    status,
                                },
                            );
                        }
                    }

                    self.domain_participant
                        .get_mut_subscriber(message.subscriber_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .get_mut_data_reader(message.data_reader_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .status_condition()
                        .send_actor_mail(status_condition_actor::AddCommunicationState {
                            state: StatusKind::RequestedIncompatibleQos,
                        });
                }
            }
        }
        Ok(())
    }
}

pub struct RemoveDiscoveredWriter {
    pub publication_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for RemoveDiscoveredWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<RemoveDiscoveredWriter> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: RemoveDiscoveredWriter,
    ) -> <RemoveDiscoveredWriter as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_mut_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        if data_reader
            .get_matched_publication_data(&message.publication_handle)
            .is_some()
        {
            data_reader.remove_matched_publication(&message.publication_handle);
        }
        Ok(())
    }
}

fn get_discovered_reader_incompatible_qos_policy_list(
    writer_qos: &DataWriterQos,
    discovered_reader_data: &SubscriptionBuiltinTopicData,
    publisher_qos: &PublisherQos,
) -> Vec<QosPolicyId> {
    let mut incompatible_qos_policy_list = Vec::new();
    if &writer_qos.durability < discovered_reader_data.durability() {
        incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
    }
    if publisher_qos.presentation.access_scope < discovered_reader_data.presentation().access_scope
        || publisher_qos.presentation.coherent_access
            != discovered_reader_data.presentation().coherent_access
        || publisher_qos.presentation.ordered_access
            != discovered_reader_data.presentation().ordered_access
    {
        incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
    }
    if &writer_qos.deadline > discovered_reader_data.deadline() {
        incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
    }
    if &writer_qos.latency_budget < discovered_reader_data.latency_budget() {
        incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
    }
    if &writer_qos.liveliness < discovered_reader_data.liveliness() {
        incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
    }
    if writer_qos.reliability.kind < discovered_reader_data.reliability().kind {
        incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
    }
    if &writer_qos.destination_order < discovered_reader_data.destination_order() {
        incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
    }
    if writer_qos.ownership.kind != discovered_reader_data.ownership().kind {
        incompatible_qos_policy_list.push(OWNERSHIP_QOS_POLICY_ID);
    }

    let writer_offered_representation = writer_qos
        .representation
        .value
        .first()
        .unwrap_or(&XCDR_DATA_REPRESENTATION);
    if !(discovered_reader_data
        .representation()
        .value
        .contains(writer_offered_representation)
        || (writer_offered_representation == &XCDR_DATA_REPRESENTATION
            && discovered_reader_data.representation().value.is_empty()))
    {
        incompatible_qos_policy_list.push(DATA_REPRESENTATION_QOS_POLICY_ID);
    }

    incompatible_qos_policy_list
}

fn get_discovered_writer_incompatible_qos_policy_list(
    data_reader: &DataReaderEntity,
    publication_builtin_topic_data: &PublicationBuiltinTopicData,
    subscriber_qos: &SubscriberQos,
) -> Vec<QosPolicyId> {
    let mut incompatible_qos_policy_list = Vec::new();

    if subscriber_qos.presentation.access_scope
        > publication_builtin_topic_data.presentation().access_scope
        || subscriber_qos.presentation.coherent_access
            != publication_builtin_topic_data
                .presentation()
                .coherent_access
        || subscriber_qos.presentation.ordered_access
            != publication_builtin_topic_data.presentation().ordered_access
    {
        incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
    }
    if &data_reader.qos().durability > publication_builtin_topic_data.durability() {
        incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
    }
    if &data_reader.qos().deadline < publication_builtin_topic_data.deadline() {
        incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
    }
    if &data_reader.qos().latency_budget > publication_builtin_topic_data.latency_budget() {
        incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
    }
    if &data_reader.qos().liveliness > publication_builtin_topic_data.liveliness() {
        incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
    }
    if data_reader.qos().reliability.kind > publication_builtin_topic_data.reliability().kind {
        incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
    }
    if &data_reader.qos().destination_order > publication_builtin_topic_data.destination_order() {
        incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
    }
    if data_reader.qos().ownership.kind != publication_builtin_topic_data.ownership().kind {
        incompatible_qos_policy_list.push(OWNERSHIP_QOS_POLICY_ID);
    }

    let writer_offered_representation = publication_builtin_topic_data
        .representation()
        .value
        .first()
        .unwrap_or(&XCDR_DATA_REPRESENTATION);
    if !data_reader
        .qos()
        .representation
        .value
        .contains(writer_offered_representation)
    {
        // Empty list is interpreted as containing XCDR_DATA_REPRESENTATION
        if !(writer_offered_representation == &XCDR_DATA_REPRESENTATION
            && data_reader.qos().representation.value.is_empty())
        {
            incompatible_qos_policy_list.push(DATA_REPRESENTATION_QOS_POLICY_ID)
        }
    }

    incompatible_qos_policy_list
}

fn is_discovered_topic_consistent(
    topic_qos: &TopicQos,
    topic_builtin_topic_data: &TopicBuiltinTopicData,
) -> bool {
    &topic_qos.topic_data == topic_builtin_topic_data.topic_data()
        && &topic_qos.durability == topic_builtin_topic_data.durability()
        && &topic_qos.deadline == topic_builtin_topic_data.deadline()
        && &topic_qos.latency_budget == topic_builtin_topic_data.latency_budget()
        && &topic_qos.liveliness == topic_builtin_topic_data.liveliness()
        && &topic_qos.reliability == topic_builtin_topic_data.reliability()
        && &topic_qos.destination_order == topic_builtin_topic_data.destination_order()
        && &topic_qos.history == topic_builtin_topic_data.history()
        && &topic_qos.resource_limits == topic_builtin_topic_data.resource_limits()
        && &topic_qos.transport_priority == topic_builtin_topic_data.transport_priority()
        && &topic_qos.lifespan == topic_builtin_topic_data.lifespan()
        && &topic_qos.ownership == topic_builtin_topic_data.ownership()
}
