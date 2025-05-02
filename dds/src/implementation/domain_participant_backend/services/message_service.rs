use crate::{
    builtin_topics::{
        BuiltInTopicKey, TopicBuiltinTopicData, DCPS_PARTICIPANT, DCPS_PUBLICATION,
        DCPS_SUBSCRIPTION,
    },
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor, services::discovery_service,
        },
    },
    infrastructure::{
        instance::InstanceHandle,
        qos_policy::{
            HistoryQosPolicy, LifespanQosPolicy, ResourceLimitsQosPolicy,
            TransportPriorityQosPolicy,
        },
    },
    runtime::actor::{ActorAddress, MailHandler},
    topic_definition::type_support::DdsDeserialize,
    transport::{history_cache::CacheChange, types::ChangeKind},
};

pub struct AddBuiltinParticipantsDetectorCacheChange {
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub cache_change: CacheChange,
}
impl MailHandler<AddBuiltinParticipantsDetectorCacheChange> for DomainParticipantActor {
    fn handle(&mut self, message: AddBuiltinParticipantsDetectorCacheChange) {
        match message.cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(discovered_participant_data) =
                    SpdpDiscoveredParticipantData::deserialize_data(
                        message.cache_change.data_value.as_ref(),
                    )
                {
                    message
                        .participant_address
                        .send_actor_mail(discovery_service::AddDiscoveredParticipant {
                            discovered_participant_data,
                        })
                        .ok();
                }
            }
            ChangeKind::NotAliveDisposed => {
                if let Ok(discovered_participant_handle) =
                    InstanceHandle::deserialize_data(message.cache_change.data_value.as_ref())
                {
                    message
                        .participant_address
                        .send_actor_mail(discovery_service::RemoveDiscoveredParticipant {
                            discovered_participant: discovered_participant_handle,
                        })
                        .ok();
                }
            }
            ChangeKind::AliveFiltered
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => (), // Do nothing,
        }

        let reception_timestamp = self.domain_participant.get_current_time();
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber_mut()
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_PARTICIPANT)
        {
            reader
                .add_reader_change(message.cache_change, reception_timestamp)
                .ok();
        }
    }
}

pub struct AddBuiltinPublicationsDetectorCacheChange {
    pub cache_change: CacheChange,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl MailHandler<AddBuiltinPublicationsDetectorCacheChange> for DomainParticipantActor {
    fn handle(&mut self, message: AddBuiltinPublicationsDetectorCacheChange) {
        match message.cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(discovered_writer_data) =
                    DiscoveredWriterData::deserialize_data(message.cache_change.data_value.as_ref())
                {
                    let publication_builtin_topic_data =
                        &discovered_writer_data.dds_publication_data;
                    if self
                        .domain_participant
                        .find_topic(&publication_builtin_topic_data.topic_name)
                        .is_none()
                    {
                        let writer_topic = TopicBuiltinTopicData {
                            key: BuiltInTopicKey::default(),
                            name: publication_builtin_topic_data.topic_name().to_owned(),
                            type_name: publication_builtin_topic_data.get_type_name().to_owned(),
                            durability: publication_builtin_topic_data.durability().clone(),
                            deadline: publication_builtin_topic_data.deadline().clone(),
                            latency_budget: publication_builtin_topic_data.latency_budget().clone(),
                            liveliness: publication_builtin_topic_data.liveliness().clone(),
                            reliability: publication_builtin_topic_data.reliability().clone(),
                            transport_priority: TransportPriorityQosPolicy::default(),
                            lifespan: publication_builtin_topic_data.lifespan().clone(),
                            destination_order: publication_builtin_topic_data
                                .destination_order()
                                .clone(),
                            history: HistoryQosPolicy::default(),
                            resource_limits: ResourceLimitsQosPolicy::default(),
                            ownership: publication_builtin_topic_data.ownership().clone(),
                            topic_data: publication_builtin_topic_data.topic_data().clone(),
                            representation: publication_builtin_topic_data.representation().clone(),
                        };
                        self.domain_participant.add_discovered_topic(writer_topic);
                    }

                    self.domain_participant
                        .add_discovered_writer(discovered_writer_data.clone());
                    for subscriber in self.domain_participant.subscriber_list() {
                        for data_reader in subscriber.data_reader_list() {
                            message
                                .participant_address
                                .send_actor_mail(discovery_service::AddDiscoveredWriter {
                                    discovered_writer_data: discovered_writer_data.clone(),
                                    subscriber_handle: subscriber.instance_handle(),
                                    data_reader_handle: data_reader.instance_handle(),
                                    participant_address: message.participant_address.clone(),
                                })
                                .ok();
                        }
                    }
                }
            }
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveDisposedUnregistered => {
                if let Ok(discovered_writer_handle) =
                    InstanceHandle::deserialize_data(message.cache_change.data_value.as_ref())
                {
                    self.domain_participant
                        .remove_discovered_writer(&discovered_writer_handle);
                    for subscriber in self.domain_participant.subscriber_list() {
                        for data_reader in subscriber.data_reader_list() {
                            message
                                .participant_address
                                .send_actor_mail(discovery_service::RemoveDiscoveredWriter {
                                    publication_handle: discovered_writer_handle,
                                    subscriber_handle: subscriber.instance_handle(),
                                    data_reader_handle: data_reader.instance_handle(),
                                })
                                .ok();
                        }
                    }
                }
            }
            ChangeKind::AliveFiltered | ChangeKind::NotAliveUnregistered => (),
        }

        let reception_timestamp = self.domain_participant.get_current_time();
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber_mut()
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_PUBLICATION)
        {
            reader
                .add_reader_change(message.cache_change, reception_timestamp)
                .ok();
        }
    }
}

pub struct AddBuiltinSubscriptionsDetectorCacheChange {
    pub cache_change: CacheChange,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl MailHandler<AddBuiltinSubscriptionsDetectorCacheChange> for DomainParticipantActor {
    fn handle(&mut self, message: AddBuiltinSubscriptionsDetectorCacheChange) {
        match message.cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(discovered_reader_data) =
                    DiscoveredReaderData::deserialize_data(message.cache_change.data_value.as_ref())
                {
                    if self
                        .domain_participant
                        .find_topic(&discovered_reader_data.dds_subscription_data.topic_name)
                        .is_none()
                    {
                        let reader_topic = TopicBuiltinTopicData {
                            key: BuiltInTopicKey::default(),
                            name: discovered_reader_data
                                .dds_subscription_data
                                .topic_name()
                                .to_string(),
                            type_name: discovered_reader_data
                                .dds_subscription_data
                                .get_type_name()
                                .to_string(),

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
                    for publisher in self.domain_participant.publisher_list() {
                        for data_writer in publisher.data_writer_list() {
                            message
                                .participant_address
                                .send_actor_mail(discovery_service::AddDiscoveredReader {
                                    discovered_reader_data: discovered_reader_data.clone(),
                                    publisher_handle: publisher.instance_handle(),
                                    data_writer_handle: data_writer.instance_handle(),
                                    participant_address: message.participant_address.clone(),
                                })
                                .ok();
                        }
                    }
                }
            }
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveDisposedUnregistered => {
                if let Ok(discovered_reader_handle) =
                    InstanceHandle::deserialize_data(message.cache_change.data_value.as_ref())
                {
                    self.domain_participant
                        .remove_discovered_reader(&discovered_reader_handle);
                    for publisher in self.domain_participant.publisher_list_mut() {
                        for data_writer in publisher.data_writer_list() {
                            message
                                .participant_address
                                .send_actor_mail(discovery_service::RemoveDiscoveredReader {
                                    subscription_handle: discovered_reader_handle,
                                    publisher_handle: publisher.instance_handle(),
                                    data_writer_handle: data_writer.instance_handle(),
                                })
                                .ok();
                        }
                    }
                }
            }
            ChangeKind::AliveFiltered | ChangeKind::NotAliveUnregistered => (),
        }

        let reception_timestamp = self.domain_participant.get_current_time();
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber_mut()
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_SUBSCRIPTION)
        {
            reader
                .add_reader_change(message.cache_change, reception_timestamp)
                .ok();
        }
    }
}
