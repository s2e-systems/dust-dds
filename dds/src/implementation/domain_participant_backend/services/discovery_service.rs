use fnmatch_regex::glob_to_regex;

use crate::{
    builtin_topics::{PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::DiscoveredWriterData,
        },
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            entities::{
                data_reader::{DataReaderEntity, TransportReaderKind},
                data_writer::TransportWriterKind,
            },
        },
        listeners::{
            data_reader_listener, data_writer_listener, domain_participant_listener,
            publisher_listener, subscriber_listener,
        },
        status_condition::status_condition_actor,
    },
    infrastructure::{
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, SubscriberQos},
        qos_policy::{
            DurabilityQosPolicyKind, QosPolicyId, ReliabilityQosPolicyKind,
            DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
            RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
        status::StatusKind,
    },
    runtime::actor::{ActorAddress, MailHandler},
    transport::{
        self,
        types::{DurabilityKind, ReliabilityKind},
    },
};

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
