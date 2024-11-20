use fnmatch_regex::glob_to_regex;

use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::domain_participant_backend::domain_participant_actor::DomainParticipantActor,
    infrastructure::{
        qos::{DataWriterQos, PublisherQos},
        qos_policy::{
            QosPolicyId, DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
            RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
    },
    runtime::actor::{Mail, MailHandler},
};

pub struct AddDiscoveredReader {
    pub subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
}
impl Mail for AddDiscoveredReader {
    type Result = ();
}
impl MailHandler<AddDiscoveredReader> for DomainParticipantActor {
    fn handle(&mut self, message: AddDiscoveredReader) -> <AddDiscoveredReader as Mail>::Result {
        fn get_discovered_reader_incompatible_qos_policy_list(
            writer_qos: &DataWriterQos,
            discovered_reader_data: &SubscriptionBuiltinTopicData,
            publisher_qos: &PublisherQos,
        ) -> Vec<QosPolicyId> {
            let mut incompatible_qos_policy_list = Vec::new();
            if &writer_qos.durability < discovered_reader_data.durability() {
                incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
            }
            if publisher_qos.presentation.access_scope
                < discovered_reader_data.presentation().access_scope
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

        for publisher in self.domain_participant.publisher_list_mut() {
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
                for data_writer in publisher.data_writer_list_mut() {
                    let is_matched_topic_name =
                        message.subscription_builtin_topic_data.topic_name()
                            == data_writer.topic_name();
                    let is_matched_type_name =
                        message.subscription_builtin_topic_data.get_type_name()
                            == data_writer.type_name();

                    if is_matched_topic_name && is_matched_type_name {
                        let incompatible_qos_policy_list =
                            get_discovered_reader_incompatible_qos_policy_list(
                                &data_writer.qos(),
                                &message.subscription_builtin_topic_data,
                                &publisher_qos,
                            );
                        if incompatible_qos_policy_list.is_empty() {
                            data_writer.add_matched_subscription(
                                message.subscription_builtin_topic_data.clone(),
                            );
                        } else {
                            todo!("Incompatible reader found")
                        }
                    }
                }
            }
        }
    }
}
