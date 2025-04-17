use fnmatch_regex::glob_to_regex;

use crate::{
    builtin_topics::{
        BuiltInTopicKey, ParticipantBuiltinTopicData, PublicationBuiltinTopicData,
        SubscriptionBuiltinTopicData, TopicBuiltinTopicData, DCPS_PARTICIPANT, DCPS_PUBLICATION,
        DCPS_SUBSCRIPTION, DCPS_TOPIC,
    },
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
            spdp_discovered_participant_data::{
                BuiltinEndpointQos, BuiltinEndpointSet, ParticipantProxy,
                SpdpDiscoveredParticipantData,
            },
        },
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            entities::{
                data_reader::{DataReaderEntity, TransportReaderKind},
                data_writer::{DataWriterEntity, TransportWriterKind},
            },
        },
        domain_participant_factory::domain_participant_factory_actor::{
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
        },
        listeners::{
            data_reader_listener, data_writer_listener, domain_participant_listener,
            publisher_listener, subscriber_listener,
        },
        status_condition::status_condition_actor,
    },
    infrastructure::{
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, SubscriberQos, TopicQos},
        qos_policy::{
            DurabilityQosPolicyKind, QosPolicyId, ReliabilityQosPolicyKind,
            DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
            RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
        status::StatusKind,
        time::Duration,
    },
    runtime::actor::{ActorAddress, MailHandler},
    topic_definition::type_support::DdsSerialize,
    transport::{
        self,
        types::{DurabilityKind, Guid, ReliabilityKind, ENTITYID_UNKNOWN},
    },
};

pub struct AnnounceParticipant;
impl MailHandler<AnnounceParticipant> for DomainParticipantActor {
    fn handle(&mut self, _: AnnounceParticipant) {
        if self.domain_participant.enabled() {
            let participant_builtin_topic_data = ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.transport.guid().into(),
                },
                user_data: self.domain_participant.qos().user_data.clone(),
            };
            let participant_proxy = ParticipantProxy {
                domain_id: Some(self.domain_participant.domain_id()),
                domain_tag: self.domain_participant.domain_tag().to_owned(),
                protocol_version: self.transport.protocol_version(),
                guid_prefix: self.transport.guid().prefix(),
                vendor_id: self.transport.vendor_id(),
                expects_inline_qos: false,
                metatraffic_unicast_locator_list: self
                    .transport
                    .metatraffic_unicast_locator_list()
                    .to_vec(),
                metatraffic_multicast_locator_list: self
                    .transport
                    .metatraffic_multicast_locator_list()
                    .to_vec(),
                default_unicast_locator_list: self
                    .transport
                    .default_unicast_locator_list()
                    .to_vec(),
                default_multicast_locator_list: self
                    .transport
                    .default_multicast_locator_list()
                    .to_vec(),
                available_builtin_endpoints: BuiltinEndpointSet::default(),
                manual_liveliness_count: 0,
                builtin_endpoint_qos: BuiltinEndpointQos::default(),
            };
            let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
                dds_participant_data: participant_builtin_topic_data,
                participant_proxy,
                lease_duration: Duration::new(100, 0),
                discovered_participant_list: self.domain_participant.get_discovered_participants(),
            };
            let timestamp = self.domain_participant.get_current_time();

            if let Some(dw) = self
                .domain_participant
                .builtin_publisher_mut()
                .lookup_datawriter_mut(DCPS_PARTICIPANT)
            {
                if let Ok(serialized_data) = spdp_discovered_participant_data.serialize_data() {
                    dw.write_w_timestamp(serialized_data, timestamp).ok();
                }
            }
        }
    }
}

pub struct AnnounceDeletedParticipant;
impl MailHandler<AnnounceDeletedParticipant> for DomainParticipantActor {
    fn handle(&mut self, _: AnnounceDeletedParticipant) {
        if self.domain_participant.enabled() {
            let timestamp = self.domain_participant.get_current_time();
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher_mut()
                .lookup_datawriter_mut(DCPS_PARTICIPANT)
            {
                let key = InstanceHandle::new(self.transport.guid().into());
                if let Ok(serialized_data) = key.serialize_data() {
                    dw.dispose_w_timestamp(serialized_data, timestamp).ok();
                }
            }
        }
    }
}

pub struct AnnounceDataWriter {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl MailHandler<AnnounceDataWriter> for DomainParticipantActor {
    fn handle(&mut self, message: AnnounceDataWriter) {
        let Some(publisher) = self
            .domain_participant
            .get_publisher(message.publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher.get_data_writer(message.data_writer_handle) else {
            return;
        };
        let Some(topic) = self.domain_participant.get_topic(data_writer.topic_name()) else {
            return;
        };

        let topic_data = topic.qos().topic_data.clone();

        let dds_publication_data = PublicationBuiltinTopicData {
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
        let writer_proxy = WriterProxy {
            remote_writer_guid: data_writer.transport_writer().guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
        };
        let discovered_writer_data = DiscoveredWriterData {
            dds_publication_data,
            writer_proxy,
        };
        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_PUBLICATION)
        {
            if let Ok(serialized_data) = discovered_writer_data.serialize_data() {
                dw.write_w_timestamp(serialized_data, timestamp).ok();
            }
        }
    }
}

pub struct AnnounceDeletedDataWriter {
    pub data_writer: DataWriterEntity,
}
impl MailHandler<AnnounceDeletedDataWriter> for DomainParticipantActor {
    fn handle(&mut self, message: AnnounceDeletedDataWriter) {
        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_PUBLICATION)
        {
            let key = InstanceHandle::new(message.data_writer.transport_writer().guid().into());
            if let Ok(serialized_data) = key.serialize_data() {
                dw.dispose_w_timestamp(serialized_data, timestamp).ok();
            }
        }
    }
}

pub struct AnnounceDataReader {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl MailHandler<AnnounceDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: AnnounceDataReader) {
        let Some(subscriber) = self
            .domain_participant
            .get_subscriber(message.subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber.get_data_reader(message.data_reader_handle) else {
            return;
        };
        let Some(topic) = self.domain_participant.get_topic(data_reader.topic_name()) else {
            return;
        };

        let guid = data_reader.transport_reader().guid();
        let dds_subscription_data = SubscriptionBuiltinTopicData {
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
        let reader_proxy = ReaderProxy {
            remote_reader_guid: data_reader.transport_reader().guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        };
        let discovered_reader_data = DiscoveredReaderData {
            dds_subscription_data,
            reader_proxy,
        };
        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_SUBSCRIPTION)
        {
            if let Ok(serialized_data) = discovered_reader_data.serialize_data() {
                dw.write_w_timestamp(serialized_data, timestamp).ok();
            }
        }
    }
}

pub struct AnnounceTopic {
    pub topic_name: String,
}
impl MailHandler<AnnounceTopic> for DomainParticipantActor {
    fn handle(&mut self, message: AnnounceTopic) {
        let Some(topic) = self.domain_participant.get_topic(&message.topic_name) else {
            return;
        };

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
            if let Ok(serialized_data) = topic_builtin_topic_data.serialize_data() {
                dw.write_w_timestamp(serialized_data, timestamp).ok();
            }
        }
    }
}

pub struct AnnounceDeletedDataReader {
    pub data_reader: DataReaderEntity,
}
impl MailHandler<AnnounceDeletedDataReader> for DomainParticipantActor {
    fn handle(&mut self, message: AnnounceDeletedDataReader) {
        let timestamp = self.domain_participant.get_current_time();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher_mut()
            .lookup_datawriter_mut(DCPS_SUBSCRIPTION)
        {
            let guid = message.data_reader.transport_reader().guid();
            let key = InstanceHandle::new(guid.into());
            if let Ok(serialized_data) = key.serialize_data() {
                dw.dispose_w_timestamp(serialized_data, timestamp).ok();
            }
        }
    }
}

pub struct AddDiscoveredTopic {
    pub topic_builtin_topic_data: TopicBuiltinTopicData,
    pub topic_name: String,
}
impl MailHandler<AddDiscoveredTopic> for DomainParticipantActor {
    fn handle(&mut self, message: AddDiscoveredTopic) {
        let Some(topic) = self.domain_participant.get_mut_topic(&message.topic_name) else {
            return;
        };
        if topic.topic_name() == message.topic_builtin_topic_data.name()
            && topic.type_name() == message.topic_builtin_topic_data.get_type_name()
            && !is_discovered_topic_consistent(topic.qos(), &message.topic_builtin_topic_data)
        {
            topic.increment_inconsistent_topic_status();
        }
    }
}

pub struct AddDiscoveredParticipant {
    pub discovered_participant_data: SpdpDiscoveredParticipantData,
}
impl MailHandler<AddDiscoveredParticipant> for DomainParticipantActor {
    fn handle(&mut self, message: AddDiscoveredParticipant) {
        // pub fn add_discovered_participant(
        //     &mut self,
        //     discovered_participant_data: &SpdpDiscoveredParticipantData,
        // ) {
        // Check that the domainId of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // AND
        // Check that the domainTag of the discovered participant equals the local one.
        // If it is not equal then there the local endpoints are not configured to
        // communicate with the discovered participant.
        // IN CASE no domain id was transmitted the a local domain id is assumed
        // (as specified in Table 9.19 - ParameterId mapping and default values)
        let is_domain_id_matching = match message
            .discovered_participant_data
            .participant_proxy
            .domain_id
        {
            Some(id) => id == self.domain_participant.domain_id(),
            None => true,
        };
        let is_domain_tag_matching = message
            .discovered_participant_data
            .participant_proxy
            .domain_tag
            == self.domain_participant.domain_tag();

        let is_participant_discovered = self
            .domain_participant
            .get_discovered_participant_data(&InstanceHandle::new(
                message
                    .discovered_participant_data
                    .dds_participant_data
                    .key
                    .value,
            ))
            .is_some();

        if is_domain_id_matching && is_domain_tag_matching && !is_participant_discovered {
            add_matched_publications_detector(self, &message.discovered_participant_data);
            add_matched_publications_announcer(self, &message.discovered_participant_data);
            add_matched_subscriptions_detector(self, &message.discovered_participant_data);
            add_matched_subscriptions_announcer(self, &message.discovered_participant_data);
            add_matched_topics_detector(self, &message.discovered_participant_data);
            add_matched_topics_announcer(self, &message.discovered_participant_data);
        }

        self.domain_participant
            .add_discovered_participant(message.discovered_participant_data);
    }
}

pub struct RemoveDiscoveredParticipant {
    pub discovered_participant: InstanceHandle,
}
impl MailHandler<RemoveDiscoveredParticipant> for DomainParticipantActor {
    fn handle(&mut self, message: RemoveDiscoveredParticipant) {
        self.domain_participant
            .remove_discovered_participant(&message.discovered_participant);
    }
}

pub struct AddDiscoveredReader {
    pub discovered_reader_data: DiscoveredReaderData,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl MailHandler<AddDiscoveredReader> for DomainParticipantActor {
    fn handle(&mut self, message: AddDiscoveredReader) {
        let default_unicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list()
            .find(|p| {
                p.participant_proxy.guid_prefix
                    == message
                        .discovered_reader_data
                        .reader_proxy
                        .remote_reader_guid
                        .prefix()
            }) {
            p.participant_proxy.default_unicast_locator_list.clone()
        } else {
            vec![]
        };
        let default_multicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list()
            .find(|p| {
                p.participant_proxy.guid_prefix
                    == message
                        .discovered_reader_data
                        .reader_proxy
                        .remote_reader_guid
                        .prefix()
            }) {
            p.participant_proxy.default_multicast_locator_list.clone()
        } else {
            vec![]
        };
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            return;
        };

        let is_any_name_matched = message
            .discovered_reader_data
            .dds_subscription_data
            .partition
            .name
            .iter()
            .any(|n| publisher.qos().partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = message
            .discovered_reader_data
            .dds_subscription_data
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
                    .discovered_reader_data
                    .dds_subscription_data
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_partition_matched = message
            .discovered_reader_data
            .dds_subscription_data
            .partition
            == publisher.qos().partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos;
        if is_partition_matched {
            let publisher_qos = publisher.qos().clone();
            let Some(data_writer) = publisher.get_mut_data_writer(message.data_writer_handle)
            else {
                return;
            };

            let is_matched_topic_name = message
                .discovered_reader_data
                .dds_subscription_data
                .topic_name()
                == data_writer.topic_name();
            let is_matched_type_name = message
                .discovered_reader_data
                .dds_subscription_data
                .get_type_name()
                == data_writer.type_name();

            if is_matched_topic_name && is_matched_type_name {
                let incompatible_qos_policy_list =
                    get_discovered_reader_incompatible_qos_policy_list(
                        data_writer.qos(),
                        &message.discovered_reader_data.dds_subscription_data,
                        &publisher_qos,
                    );
                if incompatible_qos_policy_list.is_empty() {
                    data_writer.add_matched_subscription(
                        message.discovered_reader_data.dds_subscription_data.clone(),
                    );

                    let unicast_locator_list = if message
                        .discovered_reader_data
                        .reader_proxy
                        .unicast_locator_list
                        .is_empty()
                    {
                        default_unicast_locator_list
                    } else {
                        message
                            .discovered_reader_data
                            .reader_proxy
                            .unicast_locator_list
                    };
                    let multicast_locator_list = if message
                        .discovered_reader_data
                        .reader_proxy
                        .multicast_locator_list
                        .is_empty()
                    {
                        default_multicast_locator_list
                    } else {
                        message
                            .discovered_reader_data
                            .reader_proxy
                            .multicast_locator_list
                    };
                    let reliability_kind = match message
                        .discovered_reader_data
                        .dds_subscription_data
                        .reliability
                        .kind
                    {
                        ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                        ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                    };
                    let durability_kind = match message
                        .discovered_reader_data
                        .dds_subscription_data
                        .durability
                        .kind
                    {
                        DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                        DurabilityQosPolicyKind::TransientLocal => DurabilityKind::TransientLocal,
                        DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
                        DurabilityQosPolicyKind::Persistent => DurabilityKind::Persistent,
                    };

                    let reader_proxy = transport::writer::ReaderProxy {
                        remote_reader_guid: message
                            .discovered_reader_data
                            .reader_proxy
                            .remote_reader_guid,
                        remote_group_entity_id: message
                            .discovered_reader_data
                            .reader_proxy
                            .remote_group_entity_id,
                        reliability_kind,
                        durability_kind,
                        unicast_locator_list,
                        multicast_locator_list,
                        expects_inline_qos: false,
                    };
                    if let TransportWriterKind::Stateful(w) = data_writer.transport_writer_mut() {
                        w.add_matched_reader(reader_proxy);
                    }

                    if data_writer
                        .listener_mask()
                        .contains(&StatusKind::PublicationMatched)
                    {
                        let status = data_writer.get_publication_matched_status();
                        let Ok(the_writer) = self.get_data_writer_async(
                            message.participant_address,
                            message.publisher_handle,
                            message.data_writer_handle,
                        ) else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) =
                            publisher.get_mut_data_writer(message.data_writer_handle)
                        else {
                            return;
                        };
                        if let Some(l) = data_writer.listener() {
                            l.send_actor_mail(data_writer_listener::TriggerPublicationMatched {
                                the_writer,
                                status,
                            });
                        }
                    } else if publisher
                        .listener_mask()
                        .contains(&StatusKind::PublicationMatched)
                    {
                        let Ok(the_writer) = self.get_data_writer_async(
                            message.participant_address,
                            message.publisher_handle,
                            message.data_writer_handle,
                        ) else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) =
                            publisher.get_mut_data_writer(message.data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_publication_matched_status();
                        if let Some(l) = publisher.listener() {
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
                        let Ok(the_writer) = self.get_data_writer_async(
                            message.participant_address,
                            message.publisher_handle,
                            message.data_writer_handle,
                        ) else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) =
                            publisher.get_mut_data_writer(message.data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_publication_matched_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(
                                domain_participant_listener::TriggerPublicationMatched {
                                    the_writer,
                                    status,
                                },
                            );
                        }
                    }

                    let Some(publisher) = self
                        .domain_participant
                        .get_mut_publisher(message.publisher_handle)
                    else {
                        return;
                    };
                    let Some(data_writer) =
                        publisher.get_mut_data_writer(message.data_writer_handle)
                    else {
                        return;
                    };
                    data_writer.status_condition().send_actor_mail(
                        status_condition_actor::AddCommunicationState {
                            state: StatusKind::PublicationMatched,
                        },
                    );
                } else {
                    data_writer.add_incompatible_subscription(
                        InstanceHandle::new(
                            message
                                .discovered_reader_data
                                .dds_subscription_data
                                .key()
                                .value,
                        ),
                        incompatible_qos_policy_list,
                    );

                    if data_writer
                        .listener_mask()
                        .contains(&StatusKind::OfferedIncompatibleQos)
                    {
                        let status = data_writer.get_offered_incompatible_qos_status();
                        let Ok(the_writer) = self.get_data_writer_async(
                            message.participant_address,
                            message.publisher_handle,
                            message.data_writer_handle,
                        ) else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) =
                            publisher.get_mut_data_writer(message.data_writer_handle)
                        else {
                            return;
                        };
                        if let Some(l) = data_writer.listener() {
                            l.send_actor_mail(
                                data_writer_listener::TriggerOfferedIncompatibleQos {
                                    the_writer,
                                    status,
                                },
                            );
                        }
                    } else if publisher
                        .listener_mask()
                        .contains(&StatusKind::OfferedIncompatibleQos)
                    {
                        let Ok(the_writer) = self.get_data_writer_async(
                            message.participant_address,
                            message.publisher_handle,
                            message.data_writer_handle,
                        ) else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) =
                            publisher.get_mut_data_writer(message.data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_offered_incompatible_qos_status();
                        if let Some(l) = publisher.listener() {
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
                        let Ok(the_writer) = self.get_data_writer_async(
                            message.participant_address,
                            message.publisher_handle,
                            message.data_writer_handle,
                        ) else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .get_mut_publisher(message.publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) =
                            publisher.get_mut_data_writer(message.data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_offered_incompatible_qos_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(
                                domain_participant_listener::TriggerOfferedIncompatibleQos {
                                    the_writer,
                                    status,
                                },
                            );
                        }
                    }

                    let Some(publisher) = self
                        .domain_participant
                        .get_mut_publisher(message.publisher_handle)
                    else {
                        return;
                    };
                    let Some(data_writer) =
                        publisher.get_mut_data_writer(message.data_writer_handle)
                    else {
                        return;
                    };
                    data_writer.status_condition().send_actor_mail(
                        status_condition_actor::AddCommunicationState {
                            state: StatusKind::OfferedIncompatibleQos,
                        },
                    );
                }
            }
        }
    }
}

pub struct RemoveDiscoveredReader {
    pub subscription_handle: InstanceHandle,
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl MailHandler<RemoveDiscoveredReader> for DomainParticipantActor {
    fn handle(&mut self, message: RemoveDiscoveredReader) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher.get_mut_data_writer(message.data_writer_handle) else {
            return;
        };
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
    }
}

pub struct AddDiscoveredWriter {
    pub discovered_writer_data: DiscoveredWriterData,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl MailHandler<AddDiscoveredWriter> for DomainParticipantActor {
    fn handle(&mut self, message: AddDiscoveredWriter) {
        let default_unicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list()
            .find(|p| {
                p.participant_proxy.guid_prefix
                    == message
                        .discovered_writer_data
                        .writer_proxy
                        .remote_writer_guid
                        .prefix()
            }) {
            p.participant_proxy.default_unicast_locator_list.clone()
        } else {
            vec![]
        };
        let default_multicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list()
            .find(|p| {
                p.participant_proxy.guid_prefix
                    == message
                        .discovered_writer_data
                        .writer_proxy
                        .remote_writer_guid
                        .prefix()
            }) {
            p.participant_proxy.default_multicast_locator_list.clone()
        } else {
            vec![]
        };
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            return;
        };
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
            let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle)
            else {
                return;
            };
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
                    let unicast_locator_list = if message
                        .discovered_writer_data
                        .writer_proxy
                        .unicast_locator_list
                        .is_empty()
                    {
                        default_unicast_locator_list
                    } else {
                        message
                            .discovered_writer_data
                            .writer_proxy
                            .unicast_locator_list
                    };
                    let multicast_locator_list = if message
                        .discovered_writer_data
                        .writer_proxy
                        .multicast_locator_list
                        .is_empty()
                    {
                        default_multicast_locator_list
                    } else {
                        message
                            .discovered_writer_data
                            .writer_proxy
                            .multicast_locator_list
                    };
                    let reliability_kind = match data_reader.qos().reliability.kind {
                        ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                        ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                    };
                    let durability_kind = match data_reader.qos().durability.kind {
                        DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                        DurabilityQosPolicyKind::TransientLocal => DurabilityKind::TransientLocal,
                        DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
                        DurabilityQosPolicyKind::Persistent => DurabilityKind::Persistent,
                    };
                    let writer_proxy = transport::reader::WriterProxy {
                        remote_writer_guid: message
                            .discovered_writer_data
                            .writer_proxy
                            .remote_writer_guid,
                        remote_group_entity_id: message
                            .discovered_writer_data
                            .writer_proxy
                            .remote_group_entity_id,
                        unicast_locator_list,
                        multicast_locator_list,
                        reliability_kind,
                        durability_kind,
                    };
                    if let TransportReaderKind::Stateful(r) = data_reader.transport_reader_mut() {
                        r.add_matched_writer(writer_proxy);
                    }

                    if data_reader
                        .listener_mask()
                        .contains(&StatusKind::SubscriptionMatched)
                    {
                        let Ok(the_reader) = self.get_data_reader_async(
                            message.participant_address,
                            message.subscriber_handle,
                            message.data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) =
                            subscriber.get_mut_data_reader(message.data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = data_reader.listener() {
                            l.send_actor_mail(data_reader_listener::TriggerSubscriptionMatched {
                                the_reader,
                                status,
                            });
                        }
                    } else if subscriber
                        .listener_mask()
                        .contains(&StatusKind::SubscriptionMatched)
                    {
                        let Ok(the_reader) = self.get_data_reader_async(
                            message.participant_address,
                            message.subscriber_handle,
                            message.data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) =
                            subscriber.get_mut_data_reader(message.data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = subscriber.listener() {
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
                        let Ok(the_reader) = self.get_data_reader_async(
                            message.participant_address,
                            message.subscriber_handle,
                            message.data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) =
                            subscriber.get_mut_data_reader(message.data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(
                                domain_participant_listener::TriggerSubscriptionMatched {
                                    the_reader,
                                    status,
                                },
                            );
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .get_mut_subscriber(message.subscriber_handle)
                    else {
                        return;
                    };
                    let Some(data_reader) =
                        subscriber.get_mut_data_reader(message.data_reader_handle)
                    else {
                        return;
                    };
                    data_reader.status_condition().send_actor_mail(
                        status_condition_actor::AddCommunicationState {
                            state: StatusKind::SubscriptionMatched,
                        },
                    );
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
                        let Ok(the_reader) = self.get_data_reader_async(
                            message.participant_address,
                            message.subscriber_handle,
                            message.data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) =
                            subscriber.get_mut_data_reader(message.data_reader_handle)
                        else {
                            return;
                        };
                        if let Some(l) = data_reader.listener() {
                            l.send_actor_mail(
                                data_reader_listener::TriggerRequestedIncompatibleQos {
                                    the_reader,
                                    status,
                                },
                            );
                        }
                    } else if subscriber
                        .listener_mask()
                        .contains(&StatusKind::RequestedIncompatibleQos)
                    {
                        let Ok(the_reader) = self.get_data_reader_async(
                            message.participant_address,
                            message.subscriber_handle,
                            message.data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) =
                            subscriber.get_mut_data_reader(message.data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_requested_incompatible_qos_status();
                        if let Some(l) = subscriber.listener() {
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
                        let Ok(the_reader) = self.get_data_reader_async(
                            message.participant_address,
                            message.subscriber_handle,
                            message.data_reader_handle,
                        ) else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) =
                            subscriber.get_mut_data_reader(message.data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_requested_incompatible_qos_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(
                                domain_participant_listener::TriggerRequestedIncompatibleQos {
                                    the_reader,
                                    status,
                                },
                            );
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .get_mut_subscriber(message.subscriber_handle)
                    else {
                        return;
                    };
                    let Some(data_reader) =
                        subscriber.get_mut_data_reader(message.data_reader_handle)
                    else {
                        return;
                    };
                    data_reader.status_condition().send_actor_mail(
                        status_condition_actor::AddCommunicationState {
                            state: StatusKind::RequestedIncompatibleQos,
                        },
                    );
                }
            }
        }
    }
}

pub struct RemoveDiscoveredWriter {
    pub publication_handle: InstanceHandle,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl MailHandler<RemoveDiscoveredWriter> for DomainParticipantActor {
    fn handle(&mut self, message: RemoveDiscoveredWriter) {
        let Some(subscriber) = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber.get_mut_data_reader(message.data_reader_handle) else {
            return;
        };
        if data_reader
            .get_matched_publication_data(&message.publication_handle)
            .is_some()
        {
            data_reader.remove_matched_publication(&message.publication_handle);
        }
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

fn add_matched_publications_detector(
    domain_participant_actor: &mut DomainParticipantActor,
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
        let reader_proxy = transport::writer::ReaderProxy {
            remote_reader_guid,
            remote_group_entity_id,
            reliability_kind: ReliabilityKind::Reliable,
            durability_kind: DurabilityKind::TransientLocal,
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
        if let Some(dw) = domain_participant_actor
            .domain_participant
            .builtin_publisher_mut()
            .data_writer_list_mut()
            .find(|dw| {
                dw.transport_writer().guid().entity_id()
                    == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER
            })
        {
            match dw.transport_writer_mut() {
                TransportWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                TransportWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
            }
        }
    }
}

fn add_matched_publications_announcer(
    domain_participant_actor: &mut DomainParticipantActor,
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

        let writer_proxy = transport::reader::WriterProxy {
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
            reliability_kind: ReliabilityKind::Reliable,
            durability_kind: DurabilityKind::TransientLocal,
        };
        if let Some(dr) = domain_participant_actor
            .domain_participant
            .builtin_subscriber_mut()
            .data_reader_list_mut()
            .find(|dr| {
                dr.transport_reader().guid().entity_id()
                    == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR
            })
        {
            match dr.transport_reader_mut() {
                TransportReaderKind::Stateful(r) => r.add_matched_writer(writer_proxy),
                TransportReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
            }
        }
    }
}

fn add_matched_subscriptions_detector(
    domain_participant_actor: &mut DomainParticipantActor,
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
        let reader_proxy = transport::writer::ReaderProxy {
            remote_reader_guid,
            remote_group_entity_id,
            reliability_kind: ReliabilityKind::Reliable,
            durability_kind: DurabilityKind::TransientLocal,
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
        if let Some(dw) = domain_participant_actor
            .domain_participant
            .builtin_publisher_mut()
            .data_writer_list_mut()
            .find(|dw| {
                dw.transport_writer().guid().entity_id()
                    == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER
            })
        {
            match dw.transport_writer_mut() {
                TransportWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                TransportWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
            }
        }
    }
}

fn add_matched_subscriptions_announcer(
    domain_participant_actor: &mut DomainParticipantActor,
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

        let writer_proxy = transport::reader::WriterProxy {
            remote_writer_guid,
            remote_group_entity_id,
            reliability_kind: ReliabilityKind::Reliable,
            durability_kind: DurabilityKind::TransientLocal,
            unicast_locator_list: discovered_participant_data
                .participant_proxy
                .metatraffic_unicast_locator_list
                .to_vec(),
            multicast_locator_list: discovered_participant_data
                .participant_proxy
                .metatraffic_multicast_locator_list
                .to_vec(),
        };
        if let Some(dr) = domain_participant_actor
            .domain_participant
            .builtin_subscriber_mut()
            .data_reader_list_mut()
            .find(|dr| {
                dr.transport_reader().guid().entity_id()
                    == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR
            })
        {
            match dr.transport_reader_mut() {
                TransportReaderKind::Stateful(r) => r.add_matched_writer(writer_proxy),
                TransportReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
            }
        }
    }
}

fn add_matched_topics_detector(
    domain_participant_actor: &mut DomainParticipantActor,
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
        let reader_proxy = transport::writer::ReaderProxy {
            remote_reader_guid,
            remote_group_entity_id,
            reliability_kind: ReliabilityKind::Reliable,
            durability_kind: DurabilityKind::TransientLocal,
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
        if let Some(dw) = domain_participant_actor
            .domain_participant
            .builtin_publisher_mut()
            .data_writer_list_mut()
            .find(|dw| {
                dw.transport_writer().guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER
            })
        {
            match dw.transport_writer_mut() {
                TransportWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                TransportWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
            }
        }
    }
}

fn add_matched_topics_announcer(
    domain_participant_actor: &mut DomainParticipantActor,
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

        let writer_proxy = transport::reader::WriterProxy {
            remote_writer_guid,
            remote_group_entity_id,
            reliability_kind: ReliabilityKind::Reliable,
            durability_kind: DurabilityKind::TransientLocal,
            unicast_locator_list: discovered_participant_data
                .participant_proxy
                .metatraffic_unicast_locator_list
                .to_vec(),
            multicast_locator_list: discovered_participant_data
                .participant_proxy
                .metatraffic_multicast_locator_list
                .to_vec(),
        };
        if let Some(dr) = domain_participant_actor
            .domain_participant
            .builtin_subscriber_mut()
            .data_reader_list_mut()
            .find(|dr| {
                dr.transport_reader().guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR
            })
        {
            match dr.transport_reader_mut() {
                TransportReaderKind::Stateful(r) => r.add_matched_writer(writer_proxy),
                TransportReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
            }
        }
    }
}
