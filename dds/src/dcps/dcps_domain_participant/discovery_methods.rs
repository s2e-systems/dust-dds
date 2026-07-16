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
                RemoteExceptionCode, ReplyHeader, RequestHeader, SampleIdentity, TypeLookupCall,
                TypeLookupGetTypesIn, TypeLookupGetTypesOut, TypeLookupGetTypesResult,
                TypeLookupReply, TypeLookupRequest, TypeLookupReturn,
            },
        },
        dcps_domain_participant::{
            BUILT_IN_TOPIC_NAME_LIST, BuiltInKeyHolder, DataReaderEntity, DataWriterEntity,
            DcpsDomainParticipant, DiscoveredParticipantInfo, DiscoveredTypeRepresentationState,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR, ENTITYID_TL_SVC_REPLY_READER,
            ENTITYID_TL_SVC_REPLY_WRITER, ENTITYID_TL_SVC_REQ_READER, ENTITYID_TL_SVC_REQ_WRITER,
            IncompatibleSubscriptions, RtpsReaderKind, RtpsWriterKind,
            TYPE_LOOKUP_REPLY_TOPIC_NAME, TYPE_LOOKUP_REQUEST_TOPIC_NAME,
        },
        listeners::domain_participant_listener::ListenerMail,
    },
    dds_async::{
        data_reader::DataReaderAsync, data_writer::DataWriterAsync,
        domain_participant::DomainParticipantAsync, publisher::PublisherAsync,
        subscriber::SubscriberAsync, topic::TopicAsync,
    },
    infrastructure::{
        error::DdsError,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, SubscriberQos},
        qos_policy::{
            DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, DurabilityQosPolicyKind,
            HistoryQosPolicy, LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID,
            LifespanQosPolicy, OWNERSHIP_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID, QosPolicyId,
            RELIABILITY_QOS_POLICY_ID, ReliabilityQosPolicyKind, ResourceLimitsQosPolicy,
            TransportPriorityQosPolicy, XCDR_DATA_REPRESENTATION,
        },
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE, SampleStateKind},
        status::{
            OfferedIncompatibleQosStatus, PublicationMatchedStatus, QosPolicyCount, StatusKind,
        },
        time::{Duration, DurationKind, Time},
    },
    rtps::types::{PROTOCOLVERSION, VENDOR_ID_S2E},
    runtime::{Clock, DdsRuntime},
    transport::{
        self,
        types::{DurabilityKind, ENTITYID_UNKNOWN, Guid, GuidPrefix, ReliabilityKind},
    },
    xtypes::{
        deserializer::deserialize_top_level_type,
        dynamic_type::DynamicDataFactory,
        serializer::serialize_cdr2_le,
        type_object::{
            CompleteTypeObject, MinimalTypeObject, TypeIdentifier, TypeIdentifierTypeObjectPair,
            TypeInformation, TypeObject,
        },
        type_support::{_String, Type, TypeSupport},
    },
};
use alloc::{format, string::String, vec, vec::Vec};
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

    pub fn notify_find_topic_senders(&mut self, now: Time) {
        let found_topics = self
            .domain_participant
            .find_topic_sender_list
            .extract_if(.., |x| {
                now > x.deadline
                    || self
                        .domain_participant
                        .discovered_topic_list
                        .iter()
                        .any(|t| t.name.value == x.topic_name)
                    || self
                        .domain_participant
                        .locally_created_topic_list
                        .iter()
                        .any(|t| t.topic_name == x.topic_name)
            })
            .collect::<Vec<_>>();
        for t in found_topics {
            if let Some(value) = self
                .domain_participant
                .find_topic(&t.topic_name, t.type_support)
            {
                t.reply_sender.send(Ok(value))
            } else if now > t.deadline {
                t.reply_sender.send(Err(DdsError::Timeout));
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

    pub fn check_missed_reader_deadline(&mut self, now: Time) {
        for subscriber in &mut self.domain_participant.user_defined_subscriber_list {
            for data_reader in &mut subscriber.data_reader_list {
                if let DurationKind::Finite(deadline) = data_reader.qos.deadline.period {
                    for change_instance_handle in data_reader.instances.iter().filter_map(|x| {
                        if now - x.last_received_time_stamp() > deadline {
                            Some(x.handle)
                        } else {
                            None
                        }
                    }) {
                        data_reader
                            .instance_ownership
                            .retain(|x| x.instance_handle != change_instance_handle);

                        data_reader.requested_deadline_missed_status.total_count += 1;
                        data_reader
                            .requested_deadline_missed_status
                            .total_count_change += 1;
                        data_reader
                            .requested_deadline_missed_status
                            .last_instance_handle = change_instance_handle;

                        let the_participant = DomainParticipantAsync::new(
                            self.dcps_sender,
                            self.domain_participant.domain_id,
                            self.domain_participant.instance_handle,
                        );
                        let the_subscriber = SubscriberAsync::new(
                            subscriber.instance_handle,
                            the_participant.clone(),
                        );

                        let type_name = if let Some(content_filtered_topic) = self
                            .domain_participant
                            .content_filtered_topic_list
                            .iter()
                            .find(|t| t.topic_name == data_reader.topic_name)
                        {
                            let topic = self
                                .domain_participant
                                .locally_created_topic_list
                                .iter()
                                .find(|t| t.topic_name == content_filtered_topic.related_topic_name)
                                .expect("Topic is guaranteed to exist");
                            topic.type_name.clone()
                        } else if let Some(topic) = self
                            .domain_participant
                            .locally_created_topic_list
                            .iter()
                            .find(|t| t.topic_name == data_reader.topic_name)
                        {
                            topic.type_name.clone()
                        } else {
                            panic!("Reader is guaranteed to always have a related topic");
                        };

                        let the_reader = DataReaderAsync::new(
                            data_reader.instance_handle,
                            the_subscriber.clone(),
                            data_reader.topic_name.clone(),
                            type_name,
                        );
                        if data_reader
                            .listener_mask
                            .is_enabled(&StatusKind::RequestedDeadlineMissed)
                        {
                            let status = data_reader.requested_deadline_missed_status.clone();
                            data_reader
                                .requested_deadline_missed_status
                                .total_count_change = 0;
                            if let Some(l) = &data_reader.listener_sender {
                                l.send(ListenerMail::RequestedDeadlineMissed {
                                    the_reader,
                                    status,
                                })
                                .ok();
                            }
                        } else if subscriber
                            .listener_mask
                            .is_enabled(&StatusKind::RequestedDeadlineMissed)
                        {
                            let status = data_reader.requested_deadline_missed_status.clone();
                            data_reader
                                .requested_deadline_missed_status
                                .total_count_change = 0;
                            if let Some(l) = &subscriber.listener_sender {
                                l.send(ListenerMail::RequestedDeadlineMissed {
                                    the_reader,
                                    status,
                                })
                                .ok();
                            }
                        } else if self
                            .domain_participant
                            .listener_mask
                            .is_enabled(&StatusKind::RequestedDeadlineMissed)
                        {
                            let status = data_reader.requested_deadline_missed_status.clone();
                            data_reader
                                .requested_deadline_missed_status
                                .total_count_change = 0;
                            if let Some(l) = &self.domain_participant.listener_sender {
                                l.send(ListenerMail::RequestedDeadlineMissed {
                                    the_reader,
                                    status,
                                })
                                .ok();
                            }
                        }

                        data_reader
                            .status_condition
                            .add_communication_state(StatusKind::RequestedDeadlineMissed);
                    }
                } else {
                    continue;
                }
            }
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
        let Some(topic) = self
            .domain_participant
            .locally_created_topic_list
            .iter()
            .find(|x| x.topic_name == data_writer.topic_name)
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

        let topic = if let Some(content_filtered_topic) = self
            .domain_participant
            .content_filtered_topic_list
            .iter()
            .find(|x| x.topic_name == data_reader.topic_name)
        {
            let Some(t) = self
                .domain_participant
                .locally_created_topic_list
                .iter()
                .find(|x| x.topic_name == content_filtered_topic.related_topic_name)
            else {
                return;
            };
            t
        } else {
            let Some(t) = self
                .domain_participant
                .locally_created_topic_list
                .iter()
                .find(|x| x.topic_name == data_reader.topic_name)
            else {
                return;
            };
            t
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
        let Some(topic) = self
            .domain_participant
            .locally_created_topic_list
            .iter()
            .find(|x| x.topic_name == topic_name)
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
    pub fn process_discovered_readers(&mut self) {
        for publisher in &mut self.domain_participant.user_defined_publisher_list {
            for data_writer in &mut publisher.data_writer_list {
                for discovered_reader_data in self
                    .domain_participant
                    .discovered_reader_list
                    .iter()
                    .filter(|x| x.dds_subscription_data.topic_name.value == data_writer.topic_name)
                {
                    if data_writer
                        .matched_subscription_list
                        .contains(&discovered_reader_data.dds_subscription_data)
                    {
                        continue;
                    }

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

                    let is_partition_matched =
                        discovered_reader_data.dds_subscription_data.partition
                            == publisher.qos.partition
                            || is_any_name_matched
                            || is_any_received_regex_matched_with_partition_qos
                            || is_any_local_regex_matched_with_received_partition_qos;
                    if is_partition_matched {
                        let publisher_qos = publisher.qos.clone();

                        let is_matched_topic_name = discovered_reader_data
                            .dds_subscription_data
                            .topic_name
                            .value
                            == data_writer.topic_name;
                        let writer_associated_topic = self
                            .domain_participant
                            .locally_created_topic_list
                            .iter()
                            .find(|x| x.topic_name == data_writer.topic_name)
                            .expect("A matched topic to the writer must exist");

                        let is_matched_type = match &discovered_reader_data
                            .dds_subscription_data
                            .type_information
                        {
                            Some(discovered_type_information)
                            // This additional check is done for interoperability with implementations that 
                            // do not communicate the correct type information. 
                            // In that case we fallback to matching on type name 
                                if discovered_type_information
                                    .complete
                                    .typeid_with_size
                                    .typeobject_serialized_size
                                    > 0 =>
                            {
                                // If the minimal hash match it is guaranteed compatible
                                if writer_associated_topic
                                    .type_information
                                    .minimal
                                    .typeid_with_size
                                    == discovered_type_information.minimal.typeid_with_size
                                {
                                    true
                                } else if let Some(discovered_type_information) =
                                    writer_associated_topic
                                        .discovered_type_representation
                                        .iter()
                                        .find(|(x, _)| x == discovered_type_information)
                                {
                                    match &discovered_type_information.1 {
                                        DiscoveredTypeRepresentationState::Requested => {
                                            return;
                                        }
                                        DiscoveredTypeRepresentationState::Discovered(
                                            type_object,
                                        ) => match &type_object {
                                            TypeObject::EkComplete { complete } => {
                                                CompleteTypeObject::from(
                                                    writer_associated_topic.type_support,
                                                )
                                                .is_assignable_from(complete)
                                            }
                                            TypeObject::EkMinimal { minimal } => {
                                                &MinimalTypeObject::from(
                                                    writer_associated_topic.type_support,
                                                ) == minimal
                                            }
                                        },
                                    }
                                } else {
                                    todo!("Must send a request for this type")
                                }
                            }
                            _ => {
                                discovered_reader_data.dds_subscription_data.get_type_name()
                                    == writer_associated_topic.type_name
                            }
                        };

                        let the_participant = DomainParticipantAsync::new(
                            self.dcps_sender,
                            self.domain_participant.domain_id,
                            self.domain_participant.instance_handle,
                        );
                        let the_publisher =
                            PublisherAsync::new(publisher.instance_handle, the_participant.clone());
                        let topic = self
                            .domain_participant
                            .locally_created_topic_list
                            .iter()
                            .find(|x| x.topic_name == data_writer.topic_name)
                            .expect("Writer is guaranteed to have matching topic");
                        let the_topic = TopicAsync::new(
                            topic.instance_handle,
                            data_writer.type_name.clone(),
                            data_writer.topic_name.clone(),
                            the_participant,
                        );
                        let the_writer = DataWriterAsync::new(
                            data_writer.instance_handle,
                            the_publisher,
                            the_topic,
                        );

                        if is_matched_topic_name && is_matched_type {
                            let incompatible_qos_policy_list =
                                get_discovered_reader_incompatible_qos_policy_list(
                                    &data_writer.qos,
                                    &discovered_reader_data.dds_subscription_data,
                                    &publisher_qos,
                                );
                            if incompatible_qos_policy_list.is_empty() {
                                match data_writer.matched_subscription_list.iter_mut().find(|x| {
                                    x.key() == discovered_reader_data.dds_subscription_data.key()
                                }) {
                                    Some(x) => {
                                        *x = discovered_reader_data.dds_subscription_data.clone()
                                    }
                                    None => data_writer
                                        .matched_subscription_list
                                        .push(discovered_reader_data.dds_subscription_data.clone()),
                                };
                                data_writer.publication_matched_status.current_count =
                                    data_writer.matched_subscription_list.len() as i32;
                                data_writer.publication_matched_status.current_count_change += 1;
                                data_writer.publication_matched_status.total_count += 1;
                                data_writer.publication_matched_status.total_count_change += 1;

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
                                    ReliabilityQosPolicyKind::BestEffort => {
                                        ReliabilityKind::BestEffort
                                    }
                                    ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                                };
                                let durability_kind = match discovered_reader_data
                                    .dds_subscription_data
                                    .durability
                                    .kind
                                {
                                    DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                                    DurabilityQosPolicyKind::TransientLocal => {
                                        DurabilityKind::TransientLocal
                                    }
                                    DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
                                    DurabilityQosPolicyKind::Persistent => {
                                        DurabilityKind::Persistent
                                    }
                                };

                                let reader_proxy = transport::types::ReaderProxy {
                                    remote_reader_guid: discovered_reader_data
                                        .reader_proxy
                                        .remote_reader_guid,
                                    remote_group_entity_id: discovered_reader_data
                                        .reader_proxy
                                        .remote_group_entity_id,
                                    reliability_kind,
                                    durability_kind,
                                    unicast_locator_list,
                                    multicast_locator_list,
                                    expects_inline_qos: false,
                                };
                                if let RtpsWriterKind::Stateful(w) =
                                    &mut data_writer.transport_writer
                                {
                                    w.add_matched_reader(reader_proxy);
                                }

                                if data_writer
                                    .listener_mask
                                    .is_enabled(&StatusKind::PublicationMatched)
                                {
                                    let status = data_writer.publication_matched_status.get();
                                    if let Some(l) = &data_writer.listener_sender {
                                        l.send(ListenerMail::PublicationMatched {
                                            the_writer,
                                            status,
                                        })
                                        .ok();
                                    }
                                } else if publisher
                                    .listener_mask
                                    .is_enabled(&StatusKind::PublicationMatched)
                                {
                                    let status = data_writer.publication_matched_status.get();
                                    if let Some(l) = &publisher.listener_sender {
                                        l.send(ListenerMail::PublicationMatched {
                                            the_writer,
                                            status,
                                        })
                                        .ok();
                                    }
                                } else if self
                                    .domain_participant
                                    .listener_mask
                                    .is_enabled(&StatusKind::PublicationMatched)
                                {
                                    let status = data_writer.publication_matched_status.get();
                                    if let Some(l) = &self.domain_participant.listener_sender {
                                        l.send(ListenerMail::PublicationMatched {
                                            the_writer,
                                            status,
                                        })
                                        .ok();
                                    }
                                }

                                data_writer
                                    .status_condition
                                    .add_communication_state(StatusKind::PublicationMatched);
                            } else {
                                data_writer
                                    .incompatible_subscriptions
                                    .add_incompatible_subscription(
                                        InstanceHandle::new(
                                            discovered_reader_data
                                                .dds_subscription_data
                                                .key()
                                                .value,
                                        ),
                                        incompatible_qos_policy_list,
                                    );

                                if data_writer
                                    .listener_mask
                                    .is_enabled(&StatusKind::OfferedIncompatibleQos)
                                {
                                    let status = data_writer
                                        .incompatible_subscriptions
                                        .get_offered_incompatible_qos_status();

                                    if let Some(l) = &data_writer.listener_sender {
                                        l.send(ListenerMail::OfferedIncompatibleQos {
                                            the_writer,
                                            status,
                                        })
                                        .ok();
                                    }
                                } else if publisher
                                    .listener_mask
                                    .is_enabled(&StatusKind::OfferedIncompatibleQos)
                                {
                                    let status = data_writer
                                        .incompatible_subscriptions
                                        .get_offered_incompatible_qos_status();
                                    if let Some(l) = &publisher.listener_sender {
                                        l.send(ListenerMail::OfferedIncompatibleQos {
                                            the_writer,
                                            status,
                                        })
                                        .ok();
                                    }
                                } else if self
                                    .domain_participant
                                    .listener_mask
                                    .is_enabled(&StatusKind::OfferedIncompatibleQos)
                                {
                                    let status = data_writer
                                        .incompatible_subscriptions
                                        .get_offered_incompatible_qos_status();
                                    if let Some(l) = &self.domain_participant.listener_sender {
                                        l.send(ListenerMail::OfferedIncompatibleQos {
                                            the_writer,
                                            status,
                                        })
                                        .ok();
                                    }
                                }

                                data_writer
                                    .status_condition
                                    .add_communication_state(StatusKind::OfferedIncompatibleQos);
                            }
                        }
                    }
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
    pub fn process_discovered_writers(&mut self) {
        for subscriber in &mut self.domain_participant.user_defined_subscriber_list {
            for data_reader in &mut subscriber.data_reader_list {
                let reader_topic_name = if let Some(matched_topic) = self
                    .domain_participant
                    .content_filtered_topic_list
                    .iter()
                    .find(|t| t.topic_name == data_reader.topic_name)
                {
                    matched_topic.related_topic_name.clone()
                } else {
                    data_reader.topic_name.clone()
                };

                let discovered_writer_list = self.domain_participant.discovered_writer_list.clone();
                for discovered_writer_data in discovered_writer_list
                    .iter()
                    .filter(|x| x.dds_publication_data.topic_name() == reader_topic_name)
                {
                    if data_reader
                        .matched_publication_list
                        .contains(&discovered_writer_data.dds_publication_data)
                    {
                        continue;
                    }

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

                    let is_partition_matched =
                        discovered_writer_data.dds_publication_data.partition
                            == subscriber.qos.partition
                            || is_any_name_matched
                            || is_any_received_regex_matched_with_partition_qos
                            || is_any_local_regex_matched_with_received_partition_qos;

                    if is_partition_matched {
                        let subscriber_qos = subscriber.qos.clone();

                        let (reader_topic_name, reader_type_name) = if let Some(matched_topic) =
                            self.domain_participant
                                .content_filtered_topic_list
                                .iter()
                                .find(|t| t.topic_name == data_reader.topic_name)
                        {
                            if let Some(t) = self
                                .domain_participant
                                .locally_created_topic_list
                                .iter()
                                .find(|x| x.topic_name == matched_topic.related_topic_name)
                            {
                                (&t.topic_name, &t.type_name)
                            } else {
                                continue;
                            }
                        } else if let Some(t) = self
                            .domain_participant
                            .locally_created_topic_list
                            .iter()
                            .find(|x| x.topic_name == data_reader.topic_name)
                        {
                            (&t.topic_name, &t.type_name)
                        } else {
                            continue;
                        };

                        let is_matched_topic_name =
                            discovered_writer_data.dds_publication_data.topic_name()
                                == reader_topic_name;

                        let is_matched_type_name =
                            discovered_writer_data.dds_publication_data.get_type_name()
                                == reader_type_name;

                        let the_participant = DomainParticipantAsync::new(
                            self.dcps_sender,
                            self.domain_participant.domain_id,
                            self.domain_participant.instance_handle,
                        );
                        let the_subscriber = SubscriberAsync::new(
                            subscriber.instance_handle,
                            the_participant.clone(),
                        );
                        let the_reader = DataReaderAsync::new(
                            data_reader.instance_handle,
                            the_subscriber,
                            data_reader.topic_name.clone(),
                            reader_type_name.clone(),
                        );

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
                                    ReliabilityQosPolicyKind::BestEffort => {
                                        ReliabilityKind::BestEffort
                                    }
                                    ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                                };
                                let durability_kind = match data_reader.qos.durability.kind {
                                    DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                                    DurabilityQosPolicyKind::TransientLocal => {
                                        DurabilityKind::TransientLocal
                                    }
                                    DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
                                    DurabilityQosPolicyKind::Persistent => {
                                        DurabilityKind::Persistent
                                    }
                                };
                                let writer_proxy = transport::types::WriterProxy {
                                    remote_writer_guid: discovered_writer_data
                                        .writer_proxy
                                        .remote_writer_guid,
                                    remote_group_entity_id: discovered_writer_data
                                        .writer_proxy
                                        .remote_group_entity_id,
                                    unicast_locator_list,
                                    multicast_locator_list,
                                    reliability_kind,
                                    durability_kind,
                                };
                                if let RtpsReaderKind::Stateful(r) =
                                    &mut data_reader.transport_reader
                                {
                                    r.add_matched_writer(&writer_proxy);
                                }

                                if data_reader
                                    .listener_mask
                                    .is_enabled(&StatusKind::SubscriptionMatched)
                                {
                                    let status = data_reader.get_subscription_matched_status();
                                    if let Some(l) = &data_reader.listener_sender {
                                        l.send(ListenerMail::SubscriptionMatched {
                                            the_reader,
                                            status,
                                        })
                                        .ok();
                                    }
                                } else if subscriber
                                    .listener_mask
                                    .is_enabled(&StatusKind::SubscriptionMatched)
                                {
                                    let status = data_reader.get_subscription_matched_status();
                                    if let Some(l) = &subscriber.listener_sender {
                                        l.send(ListenerMail::SubscriptionMatched {
                                            the_reader,
                                            status,
                                        })
                                        .ok();
                                    }
                                } else if self
                                    .domain_participant
                                    .listener_mask
                                    .is_enabled(&StatusKind::SubscriptionMatched)
                                {
                                    let status = data_reader.get_subscription_matched_status();
                                    if let Some(l) = &self.domain_participant.listener_sender {
                                        l.send(ListenerMail::SubscriptionMatched {
                                            the_reader,
                                            status,
                                        })
                                        .ok();
                                    }
                                }

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
                                    let status =
                                        data_reader.get_requested_incompatible_qos_status();
                                    if let Some(l) = &data_reader.listener_sender {
                                        l.send(ListenerMail::RequestedIncompatibleQos {
                                            the_reader,
                                            status,
                                        })
                                        .ok();
                                    }
                                } else if subscriber
                                    .listener_mask
                                    .is_enabled(&StatusKind::RequestedIncompatibleQos)
                                {
                                    let status =
                                        data_reader.get_requested_incompatible_qos_status();
                                    if let Some(l) = &subscriber.listener_sender {
                                        l.send(ListenerMail::RequestedIncompatibleQos {
                                            the_reader,
                                            status,
                                        })
                                        .ok();
                                    }
                                } else if self
                                    .domain_participant
                                    .listener_mask
                                    .is_enabled(&StatusKind::RequestedIncompatibleQos)
                                {
                                    let status =
                                        data_reader.get_requested_incompatible_qos_status();
                                    if let Some(l) = &self.domain_participant.listener_sender {
                                        l.send(ListenerMail::RequestedIncompatibleQos {
                                            the_reader,
                                            status,
                                        })
                                        .ok();
                                    }
                                }

                                data_reader
                                    .status_condition
                                    .add_communication_state(StatusKind::RequestedIncompatibleQos);
                            }
                        }
                    }
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
            .find(|x| x.topic_name == DCPS_PARTICIPANT)
        {
            if let Ok(samples) = dcps_reader.read(
                i32::MAX,
                &[SampleStateKind::NotRead],
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                &None,
            ) {
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
            .find(|x| x.topic_name == DCPS_PUBLICATION)
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
                            if !self
                                .domain_participant
                                .discovered_topic_list
                                .iter()
                                .any(|x| {
                                    x.name.value == publication_builtin_topic_data.topic_name()
                                })
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
            .find(|x| x.topic_name == DCPS_SUBSCRIPTION)
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
                            if !self
                                .domain_participant
                                .discovered_topic_list
                                .iter()
                                .any(|x| {
                                    x.name.value
                                        == discovered_reader_data.dds_subscription_data.topic_name()
                                })
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

    pub fn process_builtin_topics_detector_cache_change(&mut self) {
        if let Some(sedp_topics) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.topic_name == DCPS_TOPIC)
        {
            if let Ok(samples) = sedp_topics.read(
                i32::MAX,
                &[SampleStateKind::NotRead],
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                &None,
            ) {
                for (sample, sample_info) in samples {
                    if sample_info.valid_data {
                        if let Ok(discovered_topic_data) =
                            DiscoveredTopicData::from_bytes(sample.as_ref())
                        {
                            let topic_builtin_topic_data =
                                discovered_topic_data.topic_builtin_topic_data;
                            self.domain_participant
                                .add_discovered_topic(topic_builtin_topic_data.clone());
                        }
                    }
                }
            }
        }
    }

    pub fn process_builtin_type_lookup_request_cache_change(&mut self, runtime: &impl DdsRuntime) {
        if let Some(type_lookup_request_reader) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.topic_name == TYPE_LOOKUP_REQUEST_TOPIC_NAME)
        {
            if let Ok(samples) = type_lookup_request_reader.take(
                i32::MAX,
                ANY_SAMPLE_STATE,
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                &None,
            ) {
                for (sample_data, _sample_info) in samples {
                    if let Ok(mut d) =
                        deserialize_top_level_type(TypeLookupRequest::TYPE, &sample_data)
                    {
                        let type_lookup_request = TypeLookupRequest::create_sample(&mut d);
                        match type_lookup_request.call {
                            TypeLookupCall::TypeLookupGetTypesHashId { get_types } => {
                                for type_id in get_types.type_ids {
                                    if let Some(topic) = self
                                        .domain_participant
                                        .locally_created_topic_list
                                        .iter()
                                        .filter(|x| {
                                            !BUILT_IN_TOPIC_NAME_LIST
                                                .contains(&x.topic_name.as_ref())
                                        })
                                        .find(|x| {
                                            TypeInformation::from(x.type_support)
                                                .complete
                                                .typeid_with_size
                                                .type_id
                                                == type_id
                                        })
                                    {
                                        if let Some(type_lookup_reply_writer) = self
                                            .domain_participant
                                            .builtin_publisher
                                            .data_writer_list
                                            .iter_mut()
                                            .find(|x| x.topic_name == TYPE_LOOKUP_REPLY_TOPIC_NAME)
                                        {
                                            let type_lookup_reply = TypeLookupReply {
                                                header: ReplyHeader {
                                                    related_request_id: type_lookup_request.header.request_id.clone(),
                                                    remote_ex: RemoteExceptionCode::Ok
                                                },
                                                r#return:
                                                    TypeLookupReturn::TypeLookupGetTypesHash {
                                                        get_type: TypeLookupGetTypesResult::Ok {
                                                            result: TypeLookupGetTypesOut {
                                                                types: vec![
                                                                    TypeIdentifierTypeObjectPair {
                                                                        type_identifier: type_id,
                                                                        type_object:
                                                                            TypeObject::EkComplete {
                                                                                complete: CompleteTypeObject::from(topic.type_support),
                                                                            },
                                                                    },
                                                                ],
                                                                complete_to_minimal: Vec::new(),
                                                            },
                                                        },
                                                    },
                                            };
                                            let serialized_data = serialize_cdr2_le(
                                                &type_lookup_reply.create_dynamic_sample(),
                                            )
                                            .unwrap();

                                            type_lookup_reply_writer
                                                .write_w_timestamp(
                                                    InstanceHandle::default(),
                                                    serialized_data,
                                                    runtime.clock().now(),
                                                    runtime.clock().now(),
                                                    self.transport.message_writer.as_ref(),
                                                    runtime,
                                                )
                                                .ok();
                                        }
                                    }
                                }
                            }
                            TypeLookupCall::TypeLookupGetDependenciesHash {
                                get_type_dependencies: _,
                            } => todo!(),
                        }
                    }
                }
            }
        }
    }

    pub fn process_builtin_type_lookup_reply_cache_change(&mut self) {
        if let Some(type_lookup_reply_reader) = self
            .domain_participant
            .builtin_subscriber
            .data_reader_list
            .iter_mut()
            .find(|x| x.topic_name == TYPE_LOOKUP_REPLY_TOPIC_NAME)
        {
            if let Ok(samples) = type_lookup_reply_reader.take(
                i32::MAX,
                ANY_SAMPLE_STATE,
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                &None,
            ) {
                for (sample_data, _sample_info) in samples {
                    if let Ok(mut d) =
                        deserialize_top_level_type(TypeLookupReply::TYPE, &sample_data)
                    {
                        let type_lookup_reply = TypeLookupReply::create_sample(&mut d);
                        if type_lookup_reply.header.remote_ex != RemoteExceptionCode::Ok {
                            tracing::warn!(return_code=?type_lookup_reply.header.remote_ex, "Received exception on type lookup reply");
                            continue;
                        }
                        match type_lookup_reply.r#return {
                            TypeLookupReturn::TypeLookupGetTypesHash { get_type } => match get_type
                            {
                                TypeLookupGetTypesResult::Ok { result } => {
                                    for type_identifier_pair in result.types {
                                        for topic in
                                            &mut self.domain_participant.locally_created_topic_list
                                        {
                                            if let Some((_,discovered_type_state)) = topic
                                                .discovered_type_representation
                                                .iter_mut()
                                                .filter(|(_,x)| matches!(x,DiscoveredTypeRepresentationState::Requested))
                                                .find(|(type_information, _)| {
                                                    type_information
                                                        .complete
                                                        .typeid_with_size
                                                        .type_id
                                                        == type_identifier_pair.type_identifier
                                                })
                                            {
                                                *discovered_type_state = DiscoveredTypeRepresentationState::Discovered(type_identifier_pair.type_object.clone());

                                                // If two types T1 and T2 are equivalent according to the MINIMAL relation (see Clause 7.3.4.7),
                                                // then they are mutually assignable, that is, T1 is-assignable-from T2 and T2 is-assignable-from
                                                // T1.
                                                let is_type_assignable = match &type_identifier_pair.type_object{
                                                    TypeObject::EkComplete { complete } => {
                                                        CompleteTypeObject::from(topic.type_support).is_assignable_from(complete)
                                                    },
                                                    TypeObject::EkMinimal { minimal } => &MinimalTypeObject::from(topic.type_support) == minimal,
                                                };
                                                if !is_type_assignable {
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
                            },
                            TypeLookupReturn::TypeLookupGetDependenciesHash {
                                get_type_dependencies: _,
                            } => todo!(),
                        }
                    }
                }
            }
        }
    }

    pub fn request_topic_type_representation(&mut self, runtime: &impl DdsRuntime) {
        for topic in &mut self.domain_participant.locally_created_topic_list {
            for discovered_topic in self
                .domain_participant
                .discovered_topic_list
                .iter()
                .filter(|t| t.name.value == topic.topic_name)
            {
                if let Some(discovered_type_information) = &discovered_topic.type_information {
                    if discovered_type_information.minimal != topic.type_information.minimal
                        && !topic
                            .discovered_type_representation
                            .iter()
                            .any(|x| x.0 != topic.type_information)
                    {
                        {
                            if let Some(type_request_writer) = self
                                .domain_participant
                                .builtin_publisher
                                .data_writer_list
                                .iter_mut()
                                .find(|x| x.topic_name == TYPE_LOOKUP_REQUEST_TOPIC_NAME)
                            {
                                let type_lookup_request = TypeLookupRequest {
                                    header: RequestHeader {
                                        request_id: SampleIdentity {
                                            writer_guid: type_request_writer
                                                .transport_writer
                                                .guid(),
                                            sequence_number: (type_request_writer
                                                .last_change_sequence_number
                                                + 1)
                                            .into(),
                                        },
                                        instance_name: format!(
                                            "dds.builtin.TOS.{:x}",
                                            self.domain_participant.instance_handle,
                                        ),
                                    },
                                    call: TypeLookupCall::TypeLookupGetTypesHashId {
                                        get_types: TypeLookupGetTypesIn {
                                            type_ids: vec![
                                                discovered_type_information
                                                    .complete
                                                    .typeid_with_size
                                                    .type_id
                                                    .clone(),
                                            ],
                                        },
                                    },
                                };
                                let sample_instance_handle = InstanceHandle::default();
                                let serialized_data =
                                    serialize_cdr2_le(&type_lookup_request.create_dynamic_sample())
                                        .unwrap();
                                type_request_writer
                                    .write_w_timestamp(
                                        sample_instance_handle,
                                        serialized_data,
                                        runtime.clock().now(),
                                        runtime.clock().now(),
                                        self.transport.message_writer.as_ref(),
                                        runtime,
                                    )
                                    .ok();
                                topic.discovered_type_representation.push((
                                    discovered_type_information.clone(),
                                    DiscoveredTypeRepresentationState::Requested,
                                ));
                            }
                        }
                    }
                }
            }
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

impl PublicationMatchedStatus {
    pub(crate) fn get(&mut self) -> Self {
        let status = self.clone();
        self.current_count_change = 0;
        self.total_count_change = 0;

        status
    }
}

impl IncompatibleSubscriptions {
    fn add_incompatible_subscription(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        if !self.incompatible_subscription_list.contains(&handle) {
            self.offered_incompatible_qos_status.total_count += 1;
            self.offered_incompatible_qos_status.total_count_change += 1;
            self.offered_incompatible_qos_status.last_policy_id = incompatible_qos_policy_list[0];

            self.incompatible_subscription_list.push(handle);
            for incompatible_qos_policy in incompatible_qos_policy_list.into_iter() {
                if let Some(policy_count) = self
                    .offered_incompatible_qos_status
                    .policies
                    .iter_mut()
                    .find(|x| x.policy_id == incompatible_qos_policy)
                {
                    policy_count.count += 1;
                } else {
                    self.offered_incompatible_qos_status
                        .policies
                        .push(QosPolicyCount {
                            policy_id: incompatible_qos_policy,
                            count: 1,
                        })
                }
            }
        }
    }

    fn get_offered_incompatible_qos_status(&mut self) -> OfferedIncompatibleQosStatus {
        let status = self.offered_incompatible_qos_status.clone();
        self.offered_incompatible_qos_status.total_count_change = 0;
        status
    }
}
