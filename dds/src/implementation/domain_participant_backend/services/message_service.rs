use crate::{
    builtin_topics::{
        BuiltInTopicKey, SubscriptionBuiltinTopicData, TopicBuiltinTopicData, DCPS_PARTICIPANT,
        DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC,
    },
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            entities::data_reader::{AddChangeResult, TransportReaderKind},
            services::discovery_service,
        },
        listeners::{data_reader_listener, domain_participant_listener, subscriber_listener},
        status_condition::status_condition_actor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos_policy::{
            DurabilityQosPolicyKind, HistoryQosPolicy, LifespanQosPolicy, ResourceLimitsQosPolicy,
            TransportPriorityQosPolicy,
        },
        status::StatusKind,
        time::DurationKind,
    },
    runtime::actor::{ActorAddress, Mail, MailHandler},
    topic_definition::type_support::DdsDeserialize,
    transport::{history_cache::CacheChange, types::ChangeKind},
};

use super::event_service;

pub struct AddCacheChange {
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub cache_change: CacheChange,
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for AddCacheChange {
    type Result = DdsResult<()>;
}
impl MailHandler<AddCacheChange> for DomainParticipantActor {
    fn handle(&mut self, message: AddCacheChange) -> <AddCacheChange as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_mut_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        let data_reader = subscriber
            .get_mut_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let writer_instance_handle = InstanceHandle::new(message.cache_change.writer_guid.into());

        if data_reader
            .get_matched_publication_data(&writer_instance_handle)
            .is_some()
        {
            match data_reader.add_reader_change(message.cache_change)? {
                AddChangeResult::Added(change_instance_handle) => {
                    if let DurationKind::Finite(deadline_missed_period) =
                        data_reader.qos().deadline.period
                    {
                        let timer_handle = self.timer_driver.handle();
                        let participant_address = message.participant_address.clone();
                        let requested_deadline_missed_task =
                            self.backend_executor.handle().spawn(async move {
                                loop {
                                    timer_handle.sleep(deadline_missed_period.into()).await;
                                    participant_address
                                        .send_actor_mail(event_service::RequestedDeadlineMissed {
                                            subscriber_handle: message.subscriber_handle,
                                            data_reader_handle: message.data_reader_handle,
                                            change_instance_handle,
                                            participant_address: participant_address.clone(),
                                        })
                                        .ok();
                                }
                            });

                        data_reader.insert_instance_deadline_missed_task(
                            change_instance_handle,
                            requested_deadline_missed_task,
                        );
                    }

                    if self
                        .domain_participant
                        .get_mut_subscriber(message.subscriber_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .listener_mask()
                        .contains(&StatusKind::DataOnReaders)
                    {
                        let the_subscriber = self.get_subscriber_async(
                            message.participant_address.clone(),
                            message.subscriber_handle,
                        )?;
                        if let Some(l) = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .listener()
                        {
                            l.send_actor_mail(subscriber_listener::TriggerDataOnReaders {
                                the_subscriber,
                            });
                        }
                    } else if self
                        .domain_participant
                        .get_mut_subscriber(message.subscriber_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .get_mut_data_reader(message.data_reader_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .listener_mask()
                        .contains(&StatusKind::DataAvailable)
                    {
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
                            l.send_actor_mail(data_reader_listener::TriggerDataAvailable {
                                the_reader,
                            });
                        }
                    }

                    self.domain_participant
                        .get_mut_subscriber(message.subscriber_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .status_condition()
                        .send_actor_mail(status_condition_actor::AddCommunicationState {
                            state: StatusKind::DataOnReaders,
                        });

                    self.domain_participant
                        .get_mut_subscriber(message.subscriber_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .get_mut_data_reader(message.data_reader_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .status_condition()
                        .send_actor_mail(status_condition_actor::AddCommunicationState {
                            state: StatusKind::DataAvailable,
                        });
                }
                AddChangeResult::NotAdded => (), // Do nothing
                AddChangeResult::Rejected(instance_handle, sample_rejected_status_kind) => {
                    data_reader.increment_sample_rejected_status(
                        instance_handle,
                        sample_rejected_status_kind,
                    );

                    if data_reader
                        .listener_mask()
                        .contains(&StatusKind::SampleRejected)
                    {
                        let status = data_reader.get_sample_rejected_status();
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
                            l.send_actor_mail(data_reader_listener::TriggerSampleRejected {
                                the_reader,
                                status,
                            });
                        }
                    } else if self
                        .domain_participant
                        .get_mut_subscriber(message.subscriber_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .listener_mask()
                        .contains(&StatusKind::SampleRejected)
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
                            .get_sample_rejected_status();
                        if let Some(l) = self
                            .domain_participant
                            .get_mut_subscriber(message.subscriber_handle)
                            .ok_or(DdsError::AlreadyDeleted)?
                            .listener()
                        {
                            l.send_actor_mail(subscriber_listener::TriggerSampleRejected {
                                status,
                                the_reader,
                            });
                        }
                    } else if self
                        .domain_participant
                        .listener_mask()
                        .contains(&StatusKind::SampleRejected)
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
                            .get_sample_rejected_status();
                        if let Some(l) = self.domain_participant.listener() {
                            l.send_actor_mail(domain_participant_listener::TriggerSampleRejected {
                                status,
                                the_reader,
                            });
                        }
                    }

                    self.domain_participant
                        .get_mut_subscriber(message.subscriber_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .get_mut_data_reader(message.data_reader_handle)
                        .ok_or(DdsError::AlreadyDeleted)?
                        .status_condition()
                        .send_actor_mail(status_condition_actor::AddCommunicationState {
                            state: StatusKind::SampleRejected,
                        });
                }
            }
        }
        Ok(())
    }
}

pub struct AddBuiltinParticipantsDetectorCacheChange {
    pub cache_change: CacheChange,
}
impl Mail for AddBuiltinParticipantsDetectorCacheChange {
    type Result = ();
}
impl MailHandler<AddBuiltinParticipantsDetectorCacheChange> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AddBuiltinParticipantsDetectorCacheChange,
    ) -> <AddBuiltinParticipantsDetectorCacheChange as Mail>::Result {
        match message.cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(discovered_participant_data) =
                    SpdpDiscoveredParticipantData::deserialize_data(
                        message.cache_change.data_value.as_ref(),
                    )
                {
                    self.domain_participant
                        .add_discovered_participant(discovered_participant_data);
                }
            }
            ChangeKind::NotAliveDisposed => {
                if let Ok(discovered_participant_handle) =
                    InstanceHandle::deserialize_data(message.cache_change.data_value.as_ref())
                {
                    self.domain_participant
                        .remove_discovered_participant(&discovered_participant_handle);
                }
            }
            ChangeKind::AliveFiltered
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => (), // Do nothing,
        }
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber_mut()
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_PARTICIPANT)
        {
            reader.add_reader_change(message.cache_change).ok();
        }
    }
}

pub struct AddBuiltinTopicsDetectorCacheChange {
    pub cache_change: CacheChange,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for AddBuiltinTopicsDetectorCacheChange {
    type Result = ();
}
impl MailHandler<AddBuiltinTopicsDetectorCacheChange> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AddBuiltinTopicsDetectorCacheChange,
    ) -> <AddBuiltinTopicsDetectorCacheChange as Mail>::Result {
        match message.cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(topic_builtin_topic_data) = TopicBuiltinTopicData::deserialize_data(
                    message.cache_change.data_value.as_ref(),
                ) {
                    self.domain_participant
                        .add_discovered_topic(topic_builtin_topic_data.clone());
                    for topic in self.domain_participant.topic_list() {
                        message
                            .participant_address
                            .send_actor_mail(discovery_service::AddDiscoveredTopic {
                                topic_builtin_topic_data: topic_builtin_topic_data.clone(),
                                topic_name: topic.topic_name().to_owned(),
                            })
                            .ok();
                    }
                }
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::AliveFiltered
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => (),
        }

        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber_mut()
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_TOPIC)
        {
            reader.add_reader_change(message.cache_change).ok();
        }
    }
}

pub struct AddBuiltinPublicationsDetectorCacheChange {
    pub cache_change: CacheChange,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for AddBuiltinPublicationsDetectorCacheChange {
    type Result = ();
}
impl MailHandler<AddBuiltinPublicationsDetectorCacheChange> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AddBuiltinPublicationsDetectorCacheChange,
    ) -> <AddBuiltinPublicationsDetectorCacheChange as Mail>::Result {
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
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber_mut()
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_PUBLICATION)
        {
            reader.add_reader_change(message.cache_change).ok();
        }
    }
}

pub struct AddBuiltinSubscriptionsDetectorCacheChange {
    pub cache_change: CacheChange,
    pub participant_address: ActorAddress<DomainParticipantActor>,
}
impl Mail for AddBuiltinSubscriptionsDetectorCacheChange {
    type Result = ();
}
impl MailHandler<AddBuiltinSubscriptionsDetectorCacheChange> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AddBuiltinSubscriptionsDetectorCacheChange,
    ) -> <AddBuiltinSubscriptionsDetectorCacheChange as Mail>::Result {
        match message.cache_change.kind {
            ChangeKind::Alive => {
                if let Ok(subscription_builtin_topic_data) =
                    SubscriptionBuiltinTopicData::deserialize_data(
                        message.cache_change.data_value.as_ref(),
                    )
                {
                    if self
                        .domain_participant
                        .find_topic(&subscription_builtin_topic_data.topic_name)
                        .is_none()
                    {
                        let reader_topic = TopicBuiltinTopicData {
                            key: BuiltInTopicKey::default(),
                            name: subscription_builtin_topic_data.topic_name().to_string(),
                            type_name: subscription_builtin_topic_data.get_type_name().to_string(),

                            topic_data: subscription_builtin_topic_data.topic_data().clone(),
                            durability: subscription_builtin_topic_data.durability().clone(),
                            deadline: subscription_builtin_topic_data.deadline().clone(),
                            latency_budget: subscription_builtin_topic_data
                                .latency_budget()
                                .clone(),
                            liveliness: subscription_builtin_topic_data.liveliness().clone(),
                            reliability: subscription_builtin_topic_data.reliability().clone(),
                            destination_order: subscription_builtin_topic_data
                                .destination_order()
                                .clone(),
                            history: HistoryQosPolicy::default(),
                            resource_limits: ResourceLimitsQosPolicy::default(),
                            transport_priority: TransportPriorityQosPolicy::default(),
                            lifespan: LifespanQosPolicy::default(),
                            ownership: subscription_builtin_topic_data.ownership().clone(),
                            representation: subscription_builtin_topic_data
                                .representation()
                                .clone(),
                        };
                        self.domain_participant.add_discovered_topic(reader_topic);
                    }

                    self.domain_participant
                        .add_discovered_reader(subscription_builtin_topic_data.clone());
                    for publisher in self.domain_participant.publisher_list() {
                        for data_writer in publisher.data_writer_list() {
                            message
                                .participant_address
                                .send_actor_mail(discovery_service::AddDiscoveredReader {
                                    subscription_builtin_topic_data:
                                        subscription_builtin_topic_data.clone(),
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

        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber_mut()
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_SUBSCRIPTION)
        {
            reader.add_reader_change(message.cache_change).ok();
        }
    }
}

pub struct AreAllChangesAcknowledged {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
}
impl Mail for AreAllChangesAcknowledged {
    type Result = DdsResult<bool>;
}
impl MailHandler<AreAllChangesAcknowledged> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AreAllChangesAcknowledged,
    ) -> <AreAllChangesAcknowledged as Mail>::Result {
        Ok(self
            .domain_participant
            .get_publisher(message.publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(message.data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .are_all_changes_acknowledged())
    }
}

pub struct IsHistoricalDataReceived {
    pub subscriber_handle: InstanceHandle,
    pub data_reader_handle: InstanceHandle,
}
impl Mail for IsHistoricalDataReceived {
    type Result = DdsResult<bool>;
}
impl MailHandler<IsHistoricalDataReceived> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: IsHistoricalDataReceived,
    ) -> <IsHistoricalDataReceived as Mail>::Result {
        let subscriber = self
            .domain_participant
            .get_subscriber(message.subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        let data_reader = subscriber
            .get_data_reader(message.data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        if !data_reader.enabled() {
            return Err(DdsError::NotEnabled);
        };

        match data_reader.qos().durability.kind {
            DurabilityQosPolicyKind::Volatile => Err(DdsError::IllegalOperation),
            DurabilityQosPolicyKind::TransientLocal
            | DurabilityQosPolicyKind::Transient
            | DurabilityQosPolicyKind::Persistent => Ok(()),
        }?;

        if let TransportReaderKind::Stateful(r) = data_reader.transport_reader() {
            Ok(r.is_historical_data_received())
        } else {
            Ok(true)
        }
    }
}

pub struct RemoveWriterChange {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub sequence_number: i64,
}
impl Mail for RemoveWriterChange {
    type Result = ();
}
impl MailHandler<RemoveWriterChange> for DomainParticipantActor {
    fn handle(&mut self, message: RemoveWriterChange) -> <RemoveWriterChange as Mail>::Result {
        if let Some(p) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        {
            if let Some(dw) = p.get_mut_data_writer(message.data_writer_handle) {
                dw.remove_change(message.sequence_number);
            }
        }
    }
}
