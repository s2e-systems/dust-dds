use fnmatch_regex::glob_to_regex;

use crate::{
    builtin_topics::{
        BuiltInTopicKey, ParticipantBuiltinTopicData, PublicationBuiltinTopicData,
        SubscriptionBuiltinTopicData, DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION,
    },
    implementation::domain_participant_backend::{
        domain_participant_actor::DomainParticipantActor, entities::data_reader::DataReaderEntity,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos},
        qos_policy::{
            QosPolicyId, DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
            RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
    },
    runtime::actor::{Mail, MailHandler},
    topic_definition::type_support::DdsSerialize,
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
                    value: self.transport.guid(),
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
                value: data_writer.transport_writer().guid(),
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

        let subscription_builtin_topic_data = SubscriptionBuiltinTopicData {
            key: BuiltInTopicKey {
                value: data_reader.transport_reader().guid(),
            },
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
            let key = InstanceHandle::new(message.data_reader.transport_reader().guid());
            dw.dispose_w_timestamp(key.serialize_data()?, timestamp)?;
        }
        Ok(())
    }
}

pub struct AddDiscoveredReader {
    pub subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
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
                        &data_writer.qos(),
                        &message.subscription_builtin_topic_data,
                        &publisher_qos,
                    );
                if incompatible_qos_policy_list.is_empty() {
                    data_writer
                        .add_matched_subscription(message.subscription_builtin_topic_data.clone());
                } else {
                    todo!("Incompatible reader found")
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
