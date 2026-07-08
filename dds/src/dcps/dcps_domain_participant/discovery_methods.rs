use crate::{
    builtin_topics::{
        BuiltInTopicKey, DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC,
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    dcps::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
            spdp_discovered_participant_data::{
                BuiltinEndpointQos, BuiltinEndpointSet, ParticipantProxy,
                SpdpDiscoveredParticipantData,
            },
            type_lookup::{
                RequestHeader, SampleIdentity, TypeLookupCall, TypeLookupGetTypesIn,
                TypeLookupRequest,
            },
        },
        dcps_domain_participant::{
            BuiltInKeyHolder, DataReaderEntity, DataWriterEntity, DcpsDomainParticipant,
            DiscoveredParticipantInfo, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
            ENTITYID_TL_SVC_REPLY_READER, ENTITYID_TL_SVC_REPLY_WRITER, ENTITYID_TL_SVC_REQ_READER,
            ENTITYID_TL_SVC_REQ_WRITER, RtpsReaderKind, RtpsWriterKind,
            TYPE_LOOKUP_REQUEST_TOPIC_NAME, TopicDescriptionKind,
        },
        listeners::domain_participant_listener::ListenerMail,
        xtypes_glue::key_and_instance_handle::get_instance_handle_from_dynamic_data,
    },
    infrastructure::{
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, SubscriberQos, TopicQos},
        qos_policy::{
            DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, DurabilityQosPolicyKind,
            HistoryQosPolicy, LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID,
            LifespanQosPolicy, OWNERSHIP_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID, QosPolicyId,
            RELIABILITY_QOS_POLICY_ID, ReliabilityQosPolicyKind, ResourceLimitsQosPolicy,
            TransportPriorityQosPolicy, XCDR_DATA_REPRESENTATION,
        },
        sample_info::{ANY_INSTANCE_STATE, ANY_VIEW_STATE, SampleStateKind},
        status::StatusKind,
        time::{Duration, Time},
    },
    rtps::types::{PROTOCOLVERSION, VENDOR_ID_S2E},
    runtime::{Clock, DdsRuntime},
    transport::{
        self,
        types::{
            CacheChange, ChangeKind, DurabilityKind, ENTITYID_UNKNOWN, Guid, GuidPrefix,
            ReliabilityKind,
        },
    },
    xtypes::{
        deserializer::deserialize_top_level_type,
        dynamic_type::DynamicDataFactory,
        serializer::serialize_cdr2_le,
        type_object::TypeIdentifier,
        type_support::{_String, Type, TypeSupport},
    },
};
use alloc::{string::String, vec, vec::Vec};
use regex::Regex;

impl DcpsDomainParticipant {
    #[tracing::instrument(skip(self, runtime))]
    pub fn announce_participant(&mut self, runtime: &impl DdsRuntime) {
        if self.domain_participant.enabled {
            let builtin_topic_key = *self.domain_participant.instance_handle.as_ref();
            let guid = Guid::from(builtin_topic_key);
            let participant_builtin_topic_data = ParticipantBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: builtin_topic_key,
                },
                user_data: self.domain_participant.qos.user_data.clone(),
            };
            let participant_proxy = ParticipantProxy {
                domain_id: Some(self.domain_participant.domain_id),
                domain_tag: self.domain_participant.domain_tag.clone(),
                protocol_version: PROTOCOLVERSION,
                guid_prefix: guid.prefix(),
                vendor_id: VENDOR_ID_S2E,
                expects_inline_qos: false,
                metatraffic_unicast_locator_list: self
                    .transport
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                metatraffic_multicast_locator_list: self
                    .transport
                    .metatraffic_multicast_locator_list
                    .to_vec(),
                default_unicast_locator_list: self.transport.default_unicast_locator_list.to_vec(),
                default_multicast_locator_list: self
                    .transport
                    .default_multicast_locator_list
                    .to_vec(),
                available_builtin_endpoints: BuiltinEndpointSet::default(),
                manual_liveliness_count: 0,
                builtin_endpoint_qos: BuiltinEndpointQos::default(),
            };
            let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
                dds_participant_data: participant_builtin_topic_data,
                participant_proxy,
                lease_duration: Duration::new(100, 0),
                discovered_participant_list: self
                    .domain_participant
                    .discovered_participant_list
                    .iter()
                    .map(|p| InstanceHandle::new(p.dds_participant_data.key().value))
                    .collect(),
            };

            if let Some(w) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|x| x.topic_name == DCPS_PARTICIPANT)
            {
                let timestamp = runtime.clock().now();
                let sample_instance_handle = self.domain_participant.instance_handle;
                let serialized_data = spdp_discovered_participant_data.into_bytes();
                let sample_timestamp = timestamp;
                let now = timestamp;
                w.write_w_timestamp(
                    sample_instance_handle,
                    serialized_data,
                    sample_timestamp,
                    now,
                    self.transport.message_writer.as_ref(),
                    runtime,
                )
                .ok();
            }
        }
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn announce_deleted_participant(&mut self, runtime: &impl DdsRuntime) {
        if self.domain_participant.enabled {
            let timestamp = runtime.clock().now();
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|x| x.topic_name == DCPS_PARTICIPANT)
            {
                let builtin_topic_key = *self.domain_participant.instance_handle.as_ref();
                let mut dynamic_data = DynamicDataFactory::create_data(BuiltInKeyHolder::TYPE);
                let topic_key_data = BuiltInTopicKey {
                    value: builtin_topic_key,
                }
                .create_dynamic_sample();
                dynamic_data.set_complex_value(0, topic_key_data).unwrap();

                dw.unregister_w_timestamp(
                    &dynamic_data,
                    timestamp,
                    self.transport.message_writer.as_ref(),
                    runtime,
                )
                .ok();
            }
        }
    }

    pub fn remove_stale_participants(&mut self, now: Time) {
        while let Some(handle) = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .find_map(|x| {
                if now - x.reception_timestamp > x.lease_duration {
                    Some(InstanceHandle::new(x.dds_participant_data.key.value))
                } else {
                    None
                }
            })
        {
            self.remove_discovered_participant(&handle);
        }
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn announce_data_writer(
        &mut self,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
        runtime: &impl DdsRuntime,
    ) {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter()
            .find(|x| &x.instance_handle == data_writer_handle)
        else {
            return;
        };
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == data_writer.topic_name)
        else {
            return;
        };

        let topic_data = topic.qos.topic_data.clone();

        let dds_publication_data = PublicationBuiltinTopicData {
            key: BuiltInTopicKey {
                value: data_writer.transport_writer.guid().into(),
            },
            participant_key: BuiltInTopicKey {
                value: self.domain_participant.instance_handle.into(),
            },
            topic_name: data_writer.topic_name.clone().into(),
            type_name: data_writer.type_name.clone().into(),
            type_information: Some(topic.type_support.into()),
            durability: data_writer.qos.durability.clone(),
            deadline: data_writer.qos.deadline.clone(),
            latency_budget: data_writer.qos.latency_budget.clone(),
            liveliness: data_writer.qos.liveliness.clone(),
            reliability: data_writer.qos.reliability.clone(),
            lifespan: data_writer.qos.lifespan.clone(),
            user_data: data_writer.qos.user_data.clone(),
            ownership: data_writer.qos.ownership.clone(),
            ownership_strength: data_writer.qos.ownership_strength.clone(),
            destination_order: data_writer.qos.destination_order.clone(),
            presentation: publisher.qos.presentation.clone(),
            partition: publisher.qos.partition.clone(),
            topic_data,
            group_data: publisher.qos.group_data.clone(),
            representation: data_writer.qos.representation.clone(),
        };
        let writer_proxy = WriterProxy {
            remote_writer_guid: data_writer.transport_writer.guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
        };
        let discovered_writer_data = DiscoveredWriterData {
            dds_publication_data,
            writer_proxy,
        };

        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.topic_name == DCPS_PUBLICATION)
        {
            let now = runtime.clock().now();
            let sample_instance_handle = data_writer.transport_writer.guid().into();
            let serialized_data = discovered_writer_data.into_bytes();
            let sample_timestamp = now;
            let message_writer = self.transport.message_writer.as_ref();
            dw.write_w_timestamp(
                sample_instance_handle,
                serialized_data,
                sample_timestamp,
                now,
                message_writer,
                runtime,
            )
            .ok();
        }
    }

    #[tracing::instrument(skip(self, data_writer, runtime))]
    pub(super) fn announce_deleted_data_writer(
        &mut self,
        data_writer: DataWriterEntity,
        runtime: &impl DdsRuntime,
    ) {
        let timestamp = runtime.clock().now();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.topic_name == DCPS_PUBLICATION)
        {
            let mut dynamic_data = DynamicDataFactory::create_data(BuiltInKeyHolder::TYPE);
            let topic_key_data = BuiltInTopicKey {
                value: data_writer.transport_writer.guid().into(),
            }
            .create_dynamic_sample();
            dynamic_data.set_complex_value(0, topic_key_data).unwrap();

            dw.unregister_w_timestamp(
                &dynamic_data,
                timestamp,
                self.transport.message_writer.as_ref(),
                runtime,
            )
            .ok();
        }
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn announce_data_reader(
        &mut self,
        subscriber_handle: &InstanceHandle,
        data_reader_handle: &InstanceHandle,
        runtime: &impl DdsRuntime,
    ) {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter()
            .find(|x| &x.instance_handle == subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter()
            .find(|x| &x.instance_handle == data_reader_handle)
        else {
            return;
        };
        let Some(topic) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == data_reader.topic_name)
        else {
            return;
        };

        let topic = match topic {
            TopicDescriptionKind::Topic(t) => t,
            TopicDescriptionKind::ContentFilteredTopic(t) => {
                if let Some(TopicDescriptionKind::Topic(topic)) = self
                    .domain_participant
                    .topic_description_list
                    .iter()
                    .find(|x| x.topic_name() == t.related_topic_name)
                {
                    topic
                } else {
                    return;
                }
            }
        };
        let guid = data_reader.transport_reader.guid();
        let dds_subscription_data = SubscriptionBuiltinTopicData {
            key: BuiltInTopicKey { value: guid.into() },
            participant_key: BuiltInTopicKey {
                value: self.domain_participant.instance_handle.into(),
            },
            topic_name: _String {
                value: topic.topic_name.clone(),
            },
            type_name: _String {
                value: topic.type_name.clone(),
            },
            type_information: Some(topic.type_support.into()),
            durability: data_reader.qos.durability.clone(),
            deadline: data_reader.qos.deadline.clone(),
            latency_budget: data_reader.qos.latency_budget.clone(),
            liveliness: data_reader.qos.liveliness.clone(),
            reliability: data_reader.qos.reliability.clone(),
            ownership: data_reader.qos.ownership.clone(),
            destination_order: data_reader.qos.destination_order.clone(),
            user_data: data_reader.qos.user_data.clone(),
            time_based_filter: data_reader.qos.time_based_filter.clone(),
            presentation: subscriber.qos.presentation.clone(),
            partition: subscriber.qos.partition.clone(),
            topic_data: topic.qos.topic_data.clone(),
            group_data: subscriber.qos.group_data.clone(),
            representation: data_reader.qos.representation.clone(),
            type_consistency: data_reader.qos.type_consistency.clone(),
        };
        let reader_proxy = ReaderProxy {
            remote_reader_guid: data_reader.transport_reader.guid(),
            remote_group_entity_id: ENTITYID_UNKNOWN,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            expects_inline_qos: false,
        };
        let discovered_reader_data = DiscoveredReaderData {
            dds_subscription_data,
            reader_proxy,
        };

        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.topic_name == DCPS_SUBSCRIPTION)
        {
            let now = runtime.clock().now();
            let sample_instance_handle = data_reader.transport_reader.guid().into();
            let serialized_data = discovered_reader_data.into_bytes();
            let sample_timestamp = now;
            let message_writer = self.transport.message_writer.as_ref();
            dw.write_w_timestamp(
                sample_instance_handle,
                serialized_data,
                sample_timestamp,
                now,
                message_writer,
                runtime,
            )
            .ok();
        }
    }

    #[tracing::instrument(skip(self, data_reader, runtime))]
    pub(super) fn announce_deleted_data_reader(
        &mut self,
        data_reader: DataReaderEntity,
        runtime: &impl DdsRuntime,
    ) {
        let timestamp = runtime.clock().now();
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.topic_name == DCPS_SUBSCRIPTION)
        {
            let mut dynamic_data = DynamicDataFactory::create_data(BuiltInKeyHolder::TYPE);
            let topic_key_data = BuiltInTopicKey {
                value: data_reader.transport_reader.guid().into(),
            }
            .create_dynamic_sample();
            dynamic_data.set_complex_value(0, topic_key_data).unwrap();
            dw.unregister_w_timestamp(
                &dynamic_data,
                timestamp,
                self.transport.message_writer.as_ref(),
                runtime,
            )
            .ok();
        }
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn announce_topic(&mut self, topic_name: String, runtime: &impl DdsRuntime) {
        let Some(TopicDescriptionKind::Topic(topic)) = self
            .domain_participant
            .topic_description_list
            .iter()
            .find(|x| x.topic_name() == topic_name)
        else {
            return;
        };

        let discovered_topic_data = DiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: topic.instance_handle.into(),
                },
                name: topic.topic_name.clone().into(),
                type_information: Some(topic.type_support.into()),
                type_name: topic.type_name.clone().into(),
                durability: topic.qos.durability.clone(),
                deadline: topic.qos.deadline.clone(),
                latency_budget: topic.qos.latency_budget.clone(),
                liveliness: topic.qos.liveliness.clone(),
                reliability: topic.qos.reliability.clone(),
                transport_priority: topic.qos.transport_priority.clone(),
                lifespan: topic.qos.lifespan.clone(),
                destination_order: topic.qos.destination_order.clone(),
                history: topic.qos.history.clone(),
                resource_limits: topic.qos.resource_limits.clone(),
                ownership: topic.qos.ownership.clone(),
                topic_data: topic.qos.topic_data.clone(),
                representation: topic.qos.representation.clone(),
            },
        };

        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.topic_name == DCPS_TOPIC)
        {
            let sample_instance_handle = topic.instance_handle;
            let serialized_data = discovered_topic_data.into_bytes();
            let now = runtime.clock().now();
            let sample_timestamp = now;
            let message_writer = self.transport.message_writer.as_ref();
            dw.write_w_timestamp(
                sample_instance_handle,
                serialized_data,
                sample_timestamp,
                now,
                message_writer,
                runtime,
            )
            .ok();
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn add_discovered_reader(
        &mut self,
        discovered_reader_data: &DiscoveredReaderData,
        publisher_handle: &InstanceHandle,
        data_writer_handle: &InstanceHandle,
    ) {
        let default_unicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .find(|p| {
                p.guid_prefix
                    == discovered_reader_data
                        .reader_proxy
                        .remote_reader_guid
                        .prefix()
            }) {
            p.default_unicast_locator_list.clone()
        } else {
            vec![]
        };
        let default_multicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .find(|p| {
                p.guid_prefix
                    == discovered_reader_data
                        .reader_proxy
                        .remote_reader_guid
                        .prefix()
            }) {
            p.default_multicast_locator_list.clone()
        } else {
            vec![]
        };
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| &x.instance_handle == publisher_handle)
        else {
            return;
        };

        let is_any_name_matched = discovered_reader_data
            .dds_subscription_data
            .partition
            .name
            .iter()
            .any(|n| publisher.qos.partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = discovered_reader_data
            .dds_subscription_data
            .partition
            .name
            .iter()
            .filter_map(|n| Regex::new(&fnmatch_to_regex(n)).ok())
            .any(|regex| {
                publisher
                    .qos
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_any_local_regex_matched_with_received_partition_qos = publisher
            .qos
            .partition
            .name
            .iter()
            .filter_map(|n| Regex::new(&fnmatch_to_regex(n)).ok())
            .any(|regex| {
                discovered_reader_data
                    .dds_subscription_data
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_partition_matched = discovered_reader_data.dds_subscription_data.partition
            == publisher.qos.partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos;
        if is_partition_matched {
            let publisher_qos = publisher.qos.clone();
            let Some(data_writer) = publisher
                .data_writer_list
                .iter_mut()
                .find(|x| &x.instance_handle == data_writer_handle)
            else {
                return;
            };

            let is_matched_topic_name = discovered_reader_data
                .dds_subscription_data
                .topic_name
                .value
                == data_writer.topic_name;
            let is_matched_type_name = discovered_reader_data.dds_subscription_data.get_type_name()
                == data_writer.type_name;

            if is_matched_topic_name && is_matched_type_name {
                let incompatible_qos_policy_list =
                    get_discovered_reader_incompatible_qos_policy_list(
                        &data_writer.qos,
                        &discovered_reader_data.dds_subscription_data,
                        &publisher_qos,
                    );
                if incompatible_qos_policy_list.is_empty() {
                    data_writer.add_matched_subscription(
                        discovered_reader_data.dds_subscription_data.clone(),
                    );

                    let unicast_locator_list = if discovered_reader_data
                        .reader_proxy
                        .unicast_locator_list
                        .is_empty()
                    {
                        default_unicast_locator_list
                    } else {
                        discovered_reader_data
                            .reader_proxy
                            .unicast_locator_list
                            .clone()
                    };
                    let multicast_locator_list = if discovered_reader_data
                        .reader_proxy
                        .multicast_locator_list
                        .is_empty()
                    {
                        default_multicast_locator_list
                    } else {
                        discovered_reader_data
                            .reader_proxy
                            .multicast_locator_list
                            .clone()
                    };
                    let reliability_kind = match discovered_reader_data
                        .dds_subscription_data
                        .reliability
                        .kind
                    {
                        ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                        ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                    };
                    let durability_kind =
                        match discovered_reader_data.dds_subscription_data.durability.kind {
                            DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                            DurabilityQosPolicyKind::TransientLocal => {
                                DurabilityKind::TransientLocal
                            }
                            DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
                            DurabilityQosPolicyKind::Persistent => DurabilityKind::Persistent,
                        };

                    let reader_proxy = transport::types::ReaderProxy {
                        remote_reader_guid: discovered_reader_data.reader_proxy.remote_reader_guid,
                        remote_group_entity_id: discovered_reader_data
                            .reader_proxy
                            .remote_group_entity_id,
                        reliability_kind,
                        durability_kind,
                        unicast_locator_list,
                        multicast_locator_list,
                        expects_inline_qos: false,
                    };
                    if let RtpsWriterKind::Stateful(w) = &mut data_writer.transport_writer {
                        w.add_matched_reader(reader_proxy);
                    }

                    if data_writer
                        .listener_mask
                        .is_enabled(&StatusKind::PublicationMatched)
                    {
                        let status = data_writer.get_publication_matched_status();
                        let Ok(the_writer) =
                            self.get_data_writer_async(publisher_handle, data_writer_handle)
                        else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher
                            .data_writer_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == data_writer_handle)
                        else {
                            return;
                        };
                        if let Some(l) = &data_writer.listener_sender {
                            l.send(ListenerMail::PublicationMatched { the_writer, status })
                                .ok();
                        }
                    } else if publisher
                        .listener_mask
                        .is_enabled(&StatusKind::PublicationMatched)
                    {
                        let Ok(the_writer) =
                            self.get_data_writer_async(publisher_handle, data_writer_handle)
                        else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher
                            .data_writer_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_publication_matched_status();
                        if let Some(l) = &publisher.listener_sender {
                            l.send(ListenerMail::PublicationMatched { the_writer, status })
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
                        .is_enabled(&StatusKind::PublicationMatched)
                    {
                        let Ok(the_writer) =
                            self.get_data_writer_async(publisher_handle, data_writer_handle)
                        else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher
                            .data_writer_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_publication_matched_status();
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::PublicationMatched { the_writer, status })
                                .ok();
                        }
                    }

                    let Some(publisher) = self
                        .domain_participant
                        .user_defined_publisher_list
                        .iter_mut()
                        .find(|x| &x.instance_handle == publisher_handle)
                    else {
                        return;
                    };
                    let Some(data_writer) = publisher
                        .data_writer_list
                        .iter_mut()
                        .find(|x| &x.instance_handle == data_writer_handle)
                    else {
                        return;
                    };
                    data_writer
                        .status_condition
                        .add_communication_state(StatusKind::PublicationMatched);
                } else {
                    data_writer.add_incompatible_subscription(
                        InstanceHandle::new(
                            discovered_reader_data.dds_subscription_data.key().value,
                        ),
                        incompatible_qos_policy_list,
                    );

                    if data_writer
                        .listener_mask
                        .is_enabled(&StatusKind::OfferedIncompatibleQos)
                    {
                        let status = data_writer.get_offered_incompatible_qos_status();
                        let Ok(the_writer) =
                            self.get_data_writer_async(publisher_handle, data_writer_handle)
                        else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher
                            .data_writer_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == data_writer_handle)
                        else {
                            return;
                        };
                        if let Some(l) = &data_writer.listener_sender {
                            l.send(ListenerMail::OfferedIncompatibleQos { the_writer, status })
                                .ok();
                        }
                    } else if publisher
                        .listener_mask
                        .is_enabled(&StatusKind::OfferedIncompatibleQos)
                    {
                        let Ok(the_writer) =
                            self.get_data_writer_async(publisher_handle, data_writer_handle)
                        else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher
                            .data_writer_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_offered_incompatible_qos_status();
                        if let Some(l) = &publisher.listener_sender {
                            l.send(ListenerMail::OfferedIncompatibleQos { the_writer, status })
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
                        .is_enabled(&StatusKind::OfferedIncompatibleQos)
                    {
                        let Ok(the_writer) =
                            self.get_data_writer_async(publisher_handle, data_writer_handle)
                        else {
                            return;
                        };
                        let Some(publisher) = self
                            .domain_participant
                            .user_defined_publisher_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == publisher_handle)
                        else {
                            return;
                        };
                        let Some(data_writer) = publisher
                            .data_writer_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == data_writer_handle)
                        else {
                            return;
                        };
                        let status = data_writer.get_offered_incompatible_qos_status();
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::OfferedIncompatibleQos { the_writer, status })
                                .ok();
                        }
                    }

                    let Some(publisher) = self
                        .domain_participant
                        .user_defined_publisher_list
                        .iter_mut()
                        .find(|x| &x.instance_handle == publisher_handle)
                    else {
                        return;
                    };
                    let Some(data_writer) = publisher
                        .data_writer_list
                        .iter_mut()
                        .find(|x| &x.instance_handle == data_writer_handle)
                    else {
                        return;
                    };
                    data_writer
                        .status_condition
                        .add_communication_state(StatusKind::OfferedIncompatibleQos);
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_discovered_reader(
        &mut self,
        subscription_handle: InstanceHandle,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) {
        let Some(publisher) = self
            .domain_participant
            .user_defined_publisher_list
            .iter_mut()
            .find(|x| x.instance_handle == publisher_handle)
        else {
            return;
        };
        let Some(data_writer) = publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.instance_handle == data_writer_handle)
        else {
            return;
        };
        if data_writer
            .matched_subscription_list
            .iter()
            .any(|x| subscription_handle.as_ref() == &x.key().value)
        {
            data_writer.remove_matched_subscription(&subscription_handle);

            data_writer
                .status_condition
                .add_communication_state(StatusKind::PublicationMatched);
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn add_discovered_writer(
        &mut self,
        discovered_writer_data: &DiscoveredWriterData,
        subscriber_handle: &InstanceHandle,
        data_reader_handle: &InstanceHandle,
    ) {
        let default_unicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .find(|p| {
                p.guid_prefix
                    == discovered_writer_data
                        .writer_proxy
                        .remote_writer_guid
                        .prefix()
            }) {
            p.default_unicast_locator_list.clone()
        } else {
            vec![]
        };
        let default_multicast_locator_list = if let Some(p) = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .find(|p| {
                p.guid_prefix
                    == discovered_writer_data
                        .writer_proxy
                        .remote_writer_guid
                        .prefix()
            }) {
            p.default_multicast_locator_list.clone()
        } else {
            vec![]
        };
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| &x.instance_handle == subscriber_handle)
        else {
            return;
        };
        let is_any_name_matched = discovered_writer_data
            .dds_publication_data
            .partition
            .name
            .iter()
            .any(|n| subscriber.qos.partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = discovered_writer_data
            .dds_publication_data
            .partition
            .name
            .iter()
            .filter_map(|n| Regex::new(&fnmatch_to_regex(n)).ok())
            .any(|regex| {
                subscriber
                    .qos
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_any_local_regex_matched_with_received_partition_qos = subscriber
            .qos
            .partition
            .name
            .iter()
            .filter_map(|n| Regex::new(&fnmatch_to_regex(n)).ok())
            .any(|regex| {
                discovered_writer_data
                    .dds_publication_data
                    .partition
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        let is_partition_matched = discovered_writer_data.dds_publication_data.partition
            == subscriber.qos.partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos;
        if is_partition_matched {
            let subscriber_qos = subscriber.qos.clone();
            let Some(data_reader) = subscriber
                .data_reader_list
                .iter_mut()
                .find(|x| &x.instance_handle == data_reader_handle)
            else {
                return;
            };
            let Some(matched_topic) = self
                .domain_participant
                .topic_description_list
                .iter()
                .find(|t| t.topic_name() == data_reader.topic_name)
            else {
                return;
            };
            let (reader_topic_name, reader_type_name) = match matched_topic {
                TopicDescriptionKind::Topic(t) => (&t.topic_name, &t.type_name),
                TopicDescriptionKind::ContentFilteredTopic(content_filtered_topic) => {
                    if let Some(TopicDescriptionKind::Topic(matched_topic)) = self
                        .domain_participant
                        .topic_description_list
                        .iter()
                        .find(|t| t.topic_name() == content_filtered_topic.related_topic_name)
                    {
                        (&matched_topic.topic_name, &matched_topic.type_name)
                    } else {
                        return;
                    }
                }
            };
            let is_matched_topic_name =
                discovered_writer_data.dds_publication_data.topic_name() == reader_topic_name;
            let is_matched_type_name =
                discovered_writer_data.dds_publication_data.get_type_name() == reader_type_name;

            if is_matched_topic_name && is_matched_type_name {
                let incompatible_qos_policy_list =
                    get_discovered_writer_incompatible_qos_policy_list(
                        data_reader,
                        &discovered_writer_data.dds_publication_data,
                        &subscriber_qos,
                    );
                if incompatible_qos_policy_list.is_empty() {
                    data_reader.add_matched_publication(
                        discovered_writer_data.dds_publication_data.clone(),
                    );
                    let unicast_locator_list = if discovered_writer_data
                        .writer_proxy
                        .unicast_locator_list
                        .is_empty()
                    {
                        default_unicast_locator_list
                    } else {
                        discovered_writer_data
                            .writer_proxy
                            .unicast_locator_list
                            .clone()
                    };
                    let multicast_locator_list = if discovered_writer_data
                        .writer_proxy
                        .multicast_locator_list
                        .is_empty()
                    {
                        default_multicast_locator_list
                    } else {
                        discovered_writer_data
                            .writer_proxy
                            .multicast_locator_list
                            .clone()
                    };
                    let reliability_kind = match data_reader.qos.reliability.kind {
                        ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                        ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                    };
                    let durability_kind = match data_reader.qos.durability.kind {
                        DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                        DurabilityQosPolicyKind::TransientLocal => DurabilityKind::TransientLocal,
                        DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
                        DurabilityQosPolicyKind::Persistent => DurabilityKind::Persistent,
                    };
                    let writer_proxy = transport::types::WriterProxy {
                        remote_writer_guid: discovered_writer_data.writer_proxy.remote_writer_guid,
                        remote_group_entity_id: discovered_writer_data
                            .writer_proxy
                            .remote_group_entity_id,
                        unicast_locator_list,
                        multicast_locator_list,
                        reliability_kind,
                        durability_kind,
                    };
                    if let RtpsReaderKind::Stateful(r) = &mut data_reader.transport_reader {
                        r.add_matched_writer(&writer_proxy);
                    }

                    if data_reader
                        .listener_mask
                        .is_enabled(&StatusKind::SubscriptionMatched)
                    {
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = &data_reader.listener_sender {
                            l.send(ListenerMail::SubscriptionMatched { the_reader, status })
                                .ok();
                        }
                    } else if subscriber
                        .listener_mask
                        .is_enabled(&StatusKind::SubscriptionMatched)
                    {
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = &subscriber.listener_sender {
                            l.send(ListenerMail::SubscriptionMatched { the_reader, status })
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
                        .is_enabled(&StatusKind::SubscriptionMatched)
                    {
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_subscription_matched_status();
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::SubscriptionMatched { the_reader, status })
                                .ok();
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .user_defined_subscriber_list
                        .iter_mut()
                        .find(|x| &x.instance_handle == subscriber_handle)
                    else {
                        return;
                    };
                    let Some(data_reader) = subscriber
                        .data_reader_list
                        .iter_mut()
                        .find(|x| &x.instance_handle == data_reader_handle)
                    else {
                        return;
                    };
                    data_reader
                        .status_condition
                        .add_communication_state(StatusKind::SubscriptionMatched);
                } else {
                    data_reader.add_requested_incompatible_qos(
                        InstanceHandle::new(
                            discovered_writer_data.dds_publication_data.key().value,
                        ),
                        incompatible_qos_policy_list,
                    );

                    if data_reader
                        .listener_mask
                        .is_enabled(&StatusKind::RequestedIncompatibleQos)
                    {
                        let status = data_reader.get_requested_incompatible_qos_status();
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        if let Some(l) = &data_reader.listener_sender {
                            l.send(ListenerMail::RequestedIncompatibleQos { the_reader, status })
                                .ok();
                        }
                    } else if subscriber
                        .listener_mask
                        .is_enabled(&StatusKind::RequestedIncompatibleQos)
                    {
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_requested_incompatible_qos_status();
                        if let Some(l) = &subscriber.listener_sender {
                            l.send(ListenerMail::RequestedIncompatibleQos { the_reader, status })
                                .ok();
                        }
                    } else if self
                        .domain_participant
                        .listener_mask
                        .is_enabled(&StatusKind::RequestedIncompatibleQos)
                    {
                        let Ok(the_reader) =
                            self.get_data_reader_async(subscriber_handle, data_reader_handle)
                        else {
                            return;
                        };
                        let Some(subscriber) = self
                            .domain_participant
                            .user_defined_subscriber_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == subscriber_handle)
                        else {
                            return;
                        };
                        let Some(data_reader) = subscriber
                            .data_reader_list
                            .iter_mut()
                            .find(|x| &x.instance_handle == data_reader_handle)
                        else {
                            return;
                        };
                        let status = data_reader.get_requested_incompatible_qos_status();
                        if let Some(l) = &self.domain_participant.listener_sender {
                            l.send(ListenerMail::RequestedIncompatibleQos { the_reader, status })
                                .ok();
                        }
                    }

                    let Some(subscriber) = self
                        .domain_participant
                        .user_defined_subscriber_list
                        .iter_mut()
                        .find(|x| &x.instance_handle == subscriber_handle)
                    else {
                        return;
                    };
                    let Some(data_reader) = subscriber
                        .data_reader_list
                        .iter_mut()
                        .find(|x| &x.instance_handle == data_reader_handle)
                    else {
                        return;
                    };
                    data_reader
                        .status_condition
                        .add_communication_state(StatusKind::RequestedIncompatibleQos);
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_discovered_writer(
        &mut self,
        publication_handle: InstanceHandle,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) {
        let Some(subscriber) = self
            .domain_participant
            .user_defined_subscriber_list
            .iter_mut()
            .find(|x| x.instance_handle == subscriber_handle)
        else {
            return;
        };
        let Some(data_reader) = subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.instance_handle == data_reader_handle)
        else {
            return;
        };
        if data_reader
            .matched_publication_list
            .iter()
            .any(|x| &x.key().value == publication_handle.as_ref())
        {
            data_reader.remove_matched_publication(&publication_handle);
        }
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn process_discovered_participants_detector_cache_change(
        &mut self,
        runtime: &impl DdsRuntime,
    ) {
        if let Some(dcps_reader) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| &x.topic_name == DCPS_PARTICIPANT)
        {
            tracing::trace!("Found SPDP Discovered Participant detector");

            if let Ok(samples) = dcps_reader.read(
                i32::MAX,
                &[SampleStateKind::NotRead],
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                &None,
            ) {
                tracing::trace!("There are {} not read samples", samples.len());
                for (sample, sample_info) in samples {
                    let span = tracing::trace_span!("process_discovered_participant_sample");
                    let _enter_ = span.enter();
                    if sample_info.valid_data {
                        if let Ok(discovered_participant_data) =
                            SpdpDiscoveredParticipantData::from_bytes(sample.as_ref())
                        {
                            self.add_discovered_participant(&discovered_participant_data, runtime);
                        }
                    } else {
                        self.remove_discovered_participant(&sample_info.instance_handle);
                    }
                }
            } else {
                tracing::warn!("Failed to read samples from SPDP discovered participant detector");
            }
        }
    }

    pub fn process_builtin_publications_detector_cache_change(&mut self) {
        if let Some(sedp_publication) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| &x.topic_name == DCPS_PUBLICATION)
        {
            if let Ok(samples) = sedp_publication.read(
                i32::MAX,
                &[SampleStateKind::NotRead],
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                &None,
            ) {
                for (sample, sample_info) in samples {
                    if sample_info.valid_data {
                        if let Ok(discovered_writer_data) =
                            DiscoveredWriterData::from_bytes(sample.as_ref())
                        {
                            let publication_builtin_topic_data =
                                &discovered_writer_data.dds_publication_data;
                            if self
                                .domain_participant
                                .find_topic(publication_builtin_topic_data.topic_name())
                                .is_none()
                            {
                                let writer_topic = TopicBuiltinTopicData {
                                    key: BuiltInTopicKey::default(),
                                    name: publication_builtin_topic_data.topic_name.clone(),
                                    type_name: publication_builtin_topic_data.type_name.clone(),
                                    type_information: None,
                                    durability: publication_builtin_topic_data.durability().clone(),
                                    deadline: publication_builtin_topic_data.deadline().clone(),
                                    latency_budget: publication_builtin_topic_data
                                        .latency_budget()
                                        .clone(),
                                    liveliness: publication_builtin_topic_data.liveliness().clone(),
                                    reliability: publication_builtin_topic_data
                                        .reliability()
                                        .clone(),
                                    transport_priority: TransportPriorityQosPolicy::default(),
                                    lifespan: publication_builtin_topic_data.lifespan().clone(),
                                    destination_order: publication_builtin_topic_data
                                        .destination_order()
                                        .clone(),
                                    history: HistoryQosPolicy::default(),
                                    resource_limits: ResourceLimitsQosPolicy::default(),
                                    ownership: publication_builtin_topic_data.ownership().clone(),
                                    topic_data: publication_builtin_topic_data.topic_data().clone(),
                                    representation: publication_builtin_topic_data
                                        .representation()
                                        .clone(),
                                };
                                self.domain_participant.add_discovered_topic(writer_topic);
                            }

                            self.domain_participant
                                .add_discovered_writer(discovered_writer_data.clone());
                            let mut handle_list = Vec::new();
                            for subscriber in &self.domain_participant.user_defined_subscriber_list
                            {
                                for data_reader in subscriber.data_reader_list.iter() {
                                    handle_list.push((
                                        subscriber.instance_handle,
                                        data_reader.instance_handle,
                                    ));
                                }
                            }
                            for (subscriber_handle, data_reader_handle) in handle_list {
                                self.add_discovered_writer(
                                    &discovered_writer_data,
                                    &subscriber_handle,
                                    &data_reader_handle,
                                );
                            }
                        }
                    } else {
                        self.domain_participant
                            .remove_discovered_writer(&sample_info.instance_handle);

                        let mut handle_list = Vec::new();
                        for subscriber in &self.domain_participant.user_defined_subscriber_list {
                            for data_reader in subscriber.data_reader_list.iter() {
                                handle_list.push((
                                    subscriber.instance_handle,
                                    data_reader.instance_handle,
                                ));
                            }
                        }
                        for (subscriber_handle, data_reader_handle) in handle_list {
                            self.remove_discovered_writer(
                                sample_info.instance_handle,
                                subscriber_handle,
                                data_reader_handle,
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn process_builtin_subscriptions_detector_cache_change(&mut self) {
        if let Some(sedp_subscriptions) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| &x.topic_name == DCPS_SUBSCRIPTION)
        {
            if let Ok(samples) = sedp_subscriptions.read(
                i32::MAX,
                &[SampleStateKind::NotRead],
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                &None,
            ) {
                for (sample, sample_info) in samples {
                    if sample_info.valid_data {
                        if let Ok(discovered_reader_data) =
                            DiscoveredReaderData::from_bytes(sample.as_ref())
                        {
                            if self
                                .domain_participant
                                .find_topic(
                                    discovered_reader_data.dds_subscription_data.topic_name(),
                                )
                                .is_none()
                            {
                                let reader_topic = TopicBuiltinTopicData {
                                    key: BuiltInTopicKey::default(),
                                    name: discovered_reader_data
                                        .dds_subscription_data
                                        .topic_name
                                        .clone(),
                                    type_name: discovered_reader_data
                                        .dds_subscription_data
                                        .type_name
                                        .clone(),
                                    type_information: discovered_reader_data
                                        .dds_subscription_data
                                        .type_information
                                        .clone(),
                                    topic_data: discovered_reader_data
                                        .dds_subscription_data
                                        .topic_data()
                                        .clone(),
                                    durability: discovered_reader_data
                                        .dds_subscription_data
                                        .durability()
                                        .clone(),
                                    deadline: discovered_reader_data
                                        .dds_subscription_data
                                        .deadline()
                                        .clone(),
                                    latency_budget: discovered_reader_data
                                        .dds_subscription_data
                                        .latency_budget()
                                        .clone(),
                                    liveliness: discovered_reader_data
                                        .dds_subscription_data
                                        .liveliness()
                                        .clone(),
                                    reliability: discovered_reader_data
                                        .dds_subscription_data
                                        .reliability()
                                        .clone(),
                                    destination_order: discovered_reader_data
                                        .dds_subscription_data
                                        .destination_order()
                                        .clone(),
                                    history: HistoryQosPolicy::default(),
                                    resource_limits: ResourceLimitsQosPolicy::default(),
                                    transport_priority: TransportPriorityQosPolicy::default(),
                                    lifespan: LifespanQosPolicy::default(),
                                    ownership: discovered_reader_data
                                        .dds_subscription_data
                                        .ownership()
                                        .clone(),
                                    representation: discovered_reader_data
                                        .dds_subscription_data
                                        .representation()
                                        .clone(),
                                };
                                self.domain_participant.add_discovered_topic(reader_topic);
                            }

                            self.domain_participant
                                .add_discovered_reader(discovered_reader_data.clone());
                            let mut handle_list = Vec::new();
                            for publisher in &self.domain_participant.user_defined_publisher_list {
                                for data_writer in publisher.data_writer_list.iter() {
                                    handle_list.push((
                                        publisher.instance_handle,
                                        data_writer.instance_handle,
                                    ));
                                }
                            }
                            for (publisher_handle, data_writer_handle) in handle_list {
                                self.add_discovered_reader(
                                    &discovered_reader_data,
                                    &publisher_handle,
                                    &data_writer_handle,
                                );
                            }
                        }
                    } else {
                        self.domain_participant
                            .remove_discovered_reader(&sample_info.instance_handle);

                        let mut handle_list = Vec::new();
                        for publisher in &self.domain_participant.user_defined_publisher_list {
                            for data_writer in publisher.data_writer_list.iter() {
                                handle_list
                                    .push((publisher.instance_handle, data_writer.instance_handle));
                            }
                        }

                        for (publisher_handle, data_writer_handle) in handle_list {
                            self.remove_discovered_reader(
                                sample_info.instance_handle,
                                publisher_handle,
                                data_writer_handle,
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn add_builtin_topics_detector_cache_change(
        &mut self,
        cache_change: &CacheChange,
        runtime: &impl DdsRuntime,
    ) {
        match cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(discovered_topic_data) =
                    DiscoveredTopicData::from_bytes(cache_change.data_value.as_ref())
                {
                    let topic_builtin_topic_data = discovered_topic_data.topic_builtin_topic_data;
                    self.domain_participant
                        .add_discovered_topic(topic_builtin_topic_data.clone());
                    for topic in self.domain_participant.topic_description_list.iter_mut() {
                        if let TopicDescriptionKind::Topic(topic) = topic {
                            if topic.topic_name == topic_builtin_topic_data.name()
                                && topic.type_name == topic_builtin_topic_data.get_type_name()
                                && !is_discovered_topic_consistent(
                                    &topic.qos,
                                    &topic_builtin_topic_data,
                                )
                            {
                                topic.inconsistent_topic_status.total_count += 1;
                                topic.inconsistent_topic_status.total_count_change += 1;
                                topic
                                    .status_condition
                                    .add_communication_state(StatusKind::InconsistentTopic);
                            }
                        }
                    }
                }
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::AliveFiltered
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => (),
        }
    }

    #[tracing::instrument(skip(self, runtime))]
    fn add_discovered_participant(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
        runtime: &impl DdsRuntime,
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
        let is_domain_id_matching = match discovered_participant_data.participant_proxy.domain_id {
            Some(id) => id == self.domain_participant.domain_id,
            None => true,
        };
        let is_domain_tag_matching = discovered_participant_data.participant_proxy.domain_tag
            == self.domain_participant.domain_tag;
        let is_participant_discovered = self
            .domain_participant
            .discovered_participant_list
            .iter()
            .any(|p| {
                p.dds_participant_data.key().value
                    == discovered_participant_data.dds_participant_data.key.value
            });
        let is_participant_ignored = self
            .domain_participant
            .ignored_participants
            .iter()
            .any(|handle| handle == &discovered_participant_data.dds_participant_data.key.value);

        if is_domain_id_matching
            && is_domain_tag_matching
            && !is_participant_discovered
            && !is_participant_ignored
        {
            self.add_matched_publications_detector(discovered_participant_data);
            self.add_matched_publications_announcer(discovered_participant_data);
            self.add_matched_subscriptions_detector(discovered_participant_data);
            self.add_matched_subscriptions_announcer(discovered_participant_data);
            self.add_matched_topics_detector(discovered_participant_data);
            self.add_matched_topics_announcer(discovered_participant_data);

            self.add_matched_service_request_data_reader(discovered_participant_data);
            self.add_matched_service_request_data_writer(discovered_participant_data);
            self.add_matched_service_reply_data_reader(discovered_participant_data);
            self.add_matched_service_reply_data_writer(discovered_participant_data);

            self.announce_participant(runtime);

            let discovered_participant_info = DiscoveredParticipantInfo {
                dds_participant_data: discovered_participant_data.dds_participant_data.clone(),
                guid_prefix: discovered_participant_data.participant_proxy.guid_prefix,
                default_unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .default_unicast_locator_list
                    .clone(),
                default_multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .default_multicast_locator_list
                    .clone(),
                lease_duration: discovered_participant_data.lease_duration,
                reception_timestamp: runtime.clock().now(),
            };
            match self
                .domain_participant
                .discovered_participant_list
                .iter_mut()
                .find(|p| {
                    p.dds_participant_data.key()
                        == discovered_participant_info.dds_participant_data.key()
                }) {
                Some(x) => *x = discovered_participant_info,
                None => self
                    .domain_participant
                    .discovered_participant_list
                    .push(discovered_participant_info),
            }
        }
    }

    /// Remove discovered [domain participant](SpdpDiscoveredParticipantData) with the speficied [handle](InstanceHandle).
    pub fn remove_discovered_participant(&mut self, handle: &InstanceHandle) {
        self.domain_participant
            .discovered_participant_list
            .retain(|domain_participant| {
                &domain_participant.dds_participant_data.key.value != handle
            });

        let prefix = Guid::from(<[u8; 16]>::from(*handle)).prefix();

        for subscriber in &mut self.domain_participant.user_defined_subscriber_list {
            for data_reader in &mut subscriber.data_reader_list {
                // Remove samples
                data_reader
                    .sample_list
                    .retain(|sample| sample.writer_guid[..12] != prefix);

                for matched_publication in &data_reader.matched_publication_list {
                    if matched_publication.key.value[0..12] == prefix {
                        // Remove matched writers
                        if let RtpsReaderKind::Stateful(stateful_reader) =
                            &mut data_reader.transport_reader
                        {
                            stateful_reader
                                .delete_matched_writer(matched_publication.key.value.into());
                        }
                    }
                }
            }
        }

        for publisher in &mut self.domain_participant.user_defined_publisher_list {
            for data_writer in &mut publisher.data_writer_list {
                for matched_subscription in &data_writer.matched_subscription_list {
                    if matched_subscription.key.value[..12] == prefix {
                        // Remove readers
                        if let RtpsWriterKind::Stateful(stateful_writer) =
                            &mut data_writer.transport_writer
                        {
                            stateful_writer
                                .delete_matched_reader(matched_subscription.key.value.into());
                        }
                    }
                }
                data_writer
                    .matched_subscription_list
                    .retain(|subscription| subscription.key.value[..12] != prefix);
            }
        }

        self.remove_matched_publications_detector(prefix);
        self.remove_matched_publications_announcer(prefix);

        self.remove_matched_subscriptions_detector(prefix);
        self.remove_matched_subscriptions_announcer(prefix);

        self.remove_matched_topics_detector(prefix);
        self.remove_matched_topics_announcer(prefix);

        self.remove_matched_service_request_data_reader(prefix);
        self.remove_matched_service_request_data_writer(prefix);
        self.remove_matched_service_reply_data_reader(prefix);
        self.remove_matched_service_reply_data_writer(prefix);
    }

    #[tracing::instrument(skip(self))]
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
            let reader_proxy = transport::types::ReaderProxy {
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
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| {
                    dw.transport_writer.guid().entity_id()
                        == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER
                })
            {
                match &mut dw.transport_writer {
                    RtpsWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                    RtpsWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_publications_detector(&mut self, prefix: GuidPrefix) {
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|dw| {
                dw.transport_writer.guid().entity_id()
                    == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER
            })
        {
            if let RtpsWriterKind::Stateful(w) = &mut dw.transport_writer {
                let guid = Guid::new(prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR);
                w.delete_matched_reader(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
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

            let writer_proxy = transport::types::WriterProxy {
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
            if let Some(dr) = self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| {
                    dr.transport_reader.guid().entity_id()
                        == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR
                })
            {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => r.add_matched_writer(&writer_proxy),
                    RtpsReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_publications_announcer(&mut self, prefix: GuidPrefix) {
        if let Some(dr) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| {
                dr.transport_reader.guid().entity_id()
                    == ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR
            })
        {
            if let RtpsReaderKind::Stateful(r) = &mut dr.transport_reader {
                let guid = Guid::new(prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER);
                r.delete_matched_writer(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
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
            let reader_proxy = transport::types::ReaderProxy {
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
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| {
                    dw.transport_writer.guid().entity_id()
                        == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER
                })
            {
                match &mut dw.transport_writer {
                    RtpsWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                    RtpsWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_subscriptions_detector(&mut self, prefix: GuidPrefix) {
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|dw| {
                dw.transport_writer.guid().entity_id()
                    == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER
            })
        {
            if let RtpsWriterKind::Stateful(w) = &mut dw.transport_writer {
                let guid = Guid::new(prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
                w.delete_matched_reader(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
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

            let writer_proxy = transport::types::WriterProxy {
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
            if let Some(dr) = self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| {
                    dr.transport_reader.guid().entity_id()
                        == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR
                })
            {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => r.add_matched_writer(&writer_proxy),
                    RtpsReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_subscriptions_announcer(&mut self, prefix: GuidPrefix) {
        if let Some(dr) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| {
                dr.transport_reader.guid().entity_id()
                    == ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR
            })
        {
            if let RtpsReaderKind::Stateful(r) = &mut dr.transport_reader {
                let guid = Guid::new(prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER);
                r.delete_matched_writer(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
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
            let reader_proxy = transport::types::ReaderProxy {
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
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| {
                    dw.transport_writer.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER
                })
            {
                match &mut dw.transport_writer {
                    RtpsWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                    RtpsWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_topics_detector(&mut self, prefix: GuidPrefix) {
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|dw| {
                dw.transport_writer.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER
            })
        {
            if let RtpsWriterKind::Stateful(w) = &mut dw.transport_writer {
                let guid = Guid::new(prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR);
                w.delete_matched_reader(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
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

            let writer_proxy = transport::types::WriterProxy {
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
            if let Some(dr) = self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| {
                    dr.transport_reader.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR
                })
            {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => r.add_matched_writer(&writer_proxy),
                    RtpsReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_topics_announcer(&mut self, prefix: GuidPrefix) {
        if let Some(dr) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| {
                dr.transport_reader.guid().entity_id() == ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR
            })
        {
            if let RtpsReaderKind::Stateful(r) = &mut dr.transport_reader {
                let guid = Guid::new(prefix, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER);
                r.delete_matched_writer(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn add_matched_service_request_data_reader(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TYPE_LOOKUP_SERVICE_REPLY_DATA_READER)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_TL_SVC_REQ_READER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = transport::types::ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::Volatile,
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
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| dw.transport_writer.guid().entity_id() == ENTITYID_TL_SVC_REQ_WRITER)
            {
                match &mut dw.transport_writer {
                    RtpsWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                    RtpsWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_service_request_data_reader(&mut self, prefix: GuidPrefix) {
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|dw| dw.transport_writer.guid().entity_id() == ENTITYID_TL_SVC_REQ_WRITER)
        {
            if let RtpsWriterKind::Stateful(w) = &mut dw.transport_writer {
                let guid = Guid::new(prefix, ENTITYID_TL_SVC_REQ_READER);
                w.delete_matched_reader(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn add_matched_service_request_data_writer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TYPE_LOOKUP_SERVICE_REQUEST_DATA_WRITER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_TL_SVC_REQ_WRITER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;

            let writer_proxy = transport::types::WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::Volatile,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
            };
            if let Some(dr) = self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| dr.transport_reader.guid().entity_id() == ENTITYID_TL_SVC_REQ_READER)
            {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => r.add_matched_writer(&writer_proxy),
                    RtpsReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_service_request_data_writer(&mut self, prefix: GuidPrefix) {
        if let Some(dr) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| dr.transport_reader.guid().entity_id() == ENTITYID_TL_SVC_REQ_READER)
        {
            if let RtpsReaderKind::Stateful(r) = &mut dr.transport_reader {
                let guid = Guid::new(prefix, ENTITYID_TL_SVC_REQ_WRITER);
                r.delete_matched_writer(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn add_matched_service_reply_data_reader(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TYPE_LOOKUP_SERVICE_REPLY_DATA_READER)
        {
            let remote_reader_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_TL_SVC_REPLY_READER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;
            let expects_inline_qos = false;
            let reader_proxy = transport::types::ReaderProxy {
                remote_reader_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::Volatile,
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
            if let Some(dw) = self
                .domain_participant
                .builtin_publisher
                .data_writer_list
                .iter_mut()
                .find(|dw| dw.transport_writer.guid().entity_id() == ENTITYID_TL_SVC_REPLY_WRITER)
            {
                match &mut dw.transport_writer {
                    RtpsWriterKind::Stateful(w) => w.add_matched_reader(reader_proxy),
                    RtpsWriterKind::Stateless(_) => panic!("Invalid built-in writer type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_service_reply_data_reader(&mut self, prefix: GuidPrefix) {
        if let Some(dw) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|dw| dw.transport_writer.guid().entity_id() == ENTITYID_TL_SVC_REPLY_WRITER)
        {
            if let RtpsWriterKind::Stateful(w) = &mut dw.transport_writer {
                let guid = Guid::new(prefix, ENTITYID_TL_SVC_REPLY_READER);
                w.delete_matched_reader(guid);
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn add_matched_service_reply_data_writer(
        &mut self,
        discovered_participant_data: &SpdpDiscoveredParticipantData,
    ) {
        if discovered_participant_data
            .participant_proxy
            .available_builtin_endpoints
            .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TYPE_LOOKUP_SERVICE_REPLY_DATA_WRITER)
        {
            let remote_writer_guid = Guid::new(
                discovered_participant_data.participant_proxy.guid_prefix,
                ENTITYID_TL_SVC_REPLY_WRITER,
            );
            let remote_group_entity_id = ENTITYID_UNKNOWN;

            let writer_proxy = transport::types::WriterProxy {
                remote_writer_guid,
                remote_group_entity_id,
                reliability_kind: ReliabilityKind::Reliable,
                durability_kind: DurabilityKind::Volatile,
                unicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_unicast_locator_list
                    .to_vec(),
                multicast_locator_list: discovered_participant_data
                    .participant_proxy
                    .metatraffic_multicast_locator_list
                    .to_vec(),
            };
            if let Some(dr) = self
                .domain_participant
                .builtin_subscriber
                .data_reader_list
                .iter_mut()
                .find(|dr| dr.transport_reader.guid().entity_id() == ENTITYID_TL_SVC_REPLY_READER)
            {
                match &mut dr.transport_reader {
                    RtpsReaderKind::Stateful(r) => r.add_matched_writer(&writer_proxy),
                    RtpsReaderKind::Stateless(_) => panic!("Invalid built-in reader type"),
                }
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn remove_matched_service_reply_data_writer(&mut self, prefix: GuidPrefix) {
        if let Some(dr) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|dr| dr.transport_reader.guid().entity_id() == ENTITYID_TL_SVC_REPLY_READER)
        {
            if let RtpsReaderKind::Stateful(r) = &mut dr.transport_reader {
                let guid = Guid::new(prefix, ENTITYID_TL_SVC_REPLY_WRITER);
                r.delete_matched_writer(guid);
            }
        }
    }

    #[tracing::instrument(skip(self, runtime))]
    pub fn _request_type_lookup(
        &mut self,
        type_ids: Vec<TypeIdentifier>,
        runtime: &impl DdsRuntime,
    ) {
        if let Some(w) = self
            .domain_participant
            .builtin_publisher
            .data_writer_list
            .iter_mut()
            .find(|x| x.topic_name == TYPE_LOOKUP_REQUEST_TOPIC_NAME)
        {
            let dynamic_data = TypeLookupRequest {
                header: RequestHeader {
                    request_id: SampleIdentity {
                        writer_guid: w.transport_writer.guid(),
                        sequence_number: (w.last_change_sequence_number + 1).into(),
                    },
                    instance_name: String::from(""),
                },
                call: TypeLookupCall::TypeLookupGetTypesHashId {
                    get_types: TypeLookupGetTypesIn { type_ids },
                },
            }
            .create_dynamic_sample();

            let timestamp = runtime.clock().now();
            let sample_instance_handle = self.domain_participant.instance_handle;
            let serialized_data = serialize_cdr2_le(&dynamic_data).unwrap();
            let sample_timestamp = timestamp;
            let now = timestamp;
            w.write_w_timestamp(
                sample_instance_handle,
                serialized_data,
                sample_timestamp,
                now,
                self.transport.message_writer.as_ref(),
                runtime,
            )
            .ok();
        }
    }
}

#[tracing::instrument]
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
    if &writer_qos.latency_budget > discovered_reader_data.latency_budget() {
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

#[tracing::instrument(skip(data_reader))]
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
    if &data_reader.qos.durability > publication_builtin_topic_data.durability() {
        incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
    }
    if &data_reader.qos.deadline < publication_builtin_topic_data.deadline() {
        incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
    }
    if &data_reader.qos.latency_budget < publication_builtin_topic_data.latency_budget() {
        incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
    }
    if &data_reader.qos.liveliness > publication_builtin_topic_data.liveliness() {
        incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
    }
    if data_reader.qos.reliability.kind > publication_builtin_topic_data.reliability().kind {
        incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
    }
    if &data_reader.qos.destination_order > publication_builtin_topic_data.destination_order() {
        incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
    }
    if data_reader.qos.ownership.kind != publication_builtin_topic_data.ownership().kind {
        incompatible_qos_policy_list.push(OWNERSHIP_QOS_POLICY_ID);
    }

    let writer_offered_representation = publication_builtin_topic_data
        .representation()
        .value
        .first()
        .unwrap_or(&XCDR_DATA_REPRESENTATION);
    if !data_reader
        .qos
        .representation
        .value
        .contains(writer_offered_representation)
    {
        // Empty list is interpreted as containing XCDR_DATA_REPRESENTATION
        if !(writer_offered_representation == &XCDR_DATA_REPRESENTATION
            && data_reader.qos.representation.value.is_empty())
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

fn fnmatch_to_regex(pattern: &str) -> String {
    fn flush_literal(out: &mut String, lit: &mut String) {
        if !lit.is_empty() {
            out.push_str(&regex::escape(lit));
            lit.clear();
        }
    }

    let mut out = String::from("^");
    let mut literal = String::new();
    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // backslash escapes next char literally
            '\\' => {
                if let Some(next) = chars.next() {
                    literal.push(next);
                } else {
                    literal.push('\\');
                }
            }

            // glob wildcards
            '*' => {
                flush_literal(&mut out, &mut literal);
                out.push_str(".*");
            }
            '?' => {
                flush_literal(&mut out, &mut literal);
                out.push('.');
            }

            // character class
            '[' => {
                flush_literal(&mut out, &mut literal);

                let mut class = String::from("[");
                // handle fnmatch negation [!...] -> regex [^...]
                if let Some(&next) = chars.peek() {
                    if next == '!' {
                        chars.next();
                        class.push('^');
                    } else if next == '^' {
                        // treat ^ the same if user used it
                        chars.next();
                        class.push('^');
                    }
                }

                let mut closed = false;
                while let Some(ch) = chars.next() {
                    class.push(ch);
                    if ch == ']' {
                        closed = true;
                        break;
                    }
                    // preserve escaped chars inside class
                    if ch == '\\' {
                        if let Some(esc) = chars.next() {
                            class.push(esc);
                        }
                    }
                }

                if closed {
                    out.push_str(&class);
                } else {
                    // unclosed '[' — treat as literal
                    literal.push('[');
                    literal.push_str(&class[1..]); // append rest as literal
                }
            }

            '+' => {
                flush_literal(&mut out, &mut literal);
                out.push('+'); // regex plus (quantifier)
            }

            // default: accumulate literal characters (will be escaped when flushed)
            other => literal.push(other),
        }
    }

    flush_literal(&mut out, &mut literal);
    out.push('$');
    out
}
