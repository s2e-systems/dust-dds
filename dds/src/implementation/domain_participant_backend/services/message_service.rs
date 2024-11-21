use crate::{
    builtin_topics::{
        PublicationBuiltinTopicData, SubscriptionBuiltinTopicData, DCPS_PARTICIPANT,
        DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC,
    },
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor, services::discovery_service,
        },
        status_condition::status_condition_actor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos_policy::DurabilityQosPolicyKind,
        status::StatusKind,
    },
    rtps::{
        reader::ReaderCacheChange,
        types::{ChangeKind, SequenceNumber},
    },
    runtime::actor::{ActorAddress, Mail, MailHandler},
    subscription::sample_info::{
        InstanceStateKind, SampleStateKind, ANY_INSTANCE_STATE, ANY_VIEW_STATE,
    },
    topic_definition::type_support::DdsDeserialize,
};

pub struct AddCacheChange {
    pub domain_participant_address: ActorAddress<DomainParticipantActor>,
    pub cache_change: ReaderCacheChange,
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
            if let Ok(change_instance_handle) = data_reader.add_reader_change(message.cache_change)
            {
                subscriber.status_condition().send_actor_mail(
                    status_condition_actor::AddCommunicationState {
                        state: StatusKind::DataOnReaders,
                    },
                );
            }
        }
        Ok(())
    }
}

pub struct AddBuiltinParticipantsDetectorCacheChange {
    pub cache_change: ReaderCacheChange,
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
    pub cache_change: ReaderCacheChange,
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
                if let Ok(discovered_topic_data) =
                    DiscoveredTopicData::deserialize_data(message.cache_change.data_value.as_ref())
                {
                }
            }
            ChangeKind::NotAliveDisposed => todo!(),
            ChangeKind::AliveFiltered
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
            if let Ok(samples) = reader.read(
                i32::MAX,
                &[SampleStateKind::NotRead],
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                None,
            ) {
                for (sample_data, sample_info) in samples {
                    match sample_info.instance_state {
                        InstanceStateKind::Alive => {
                            if let Ok(discovered_topic_data) = DiscoveredTopicData::deserialize_data(
                                sample_data
                                    .expect("Alive samples must contain data")
                                    .as_ref(),
                            ) {
                                todo!()
                                // self.add_discovered_topic(discovered_topic_data);
                            }
                        }
                        InstanceStateKind::NotAliveDisposed => (), // Discovered topics are not deleted,
                        InstanceStateKind::NotAliveNoWriters => (),
                    }
                }
            }
        }
    }
}

pub struct AddBuiltinPublicationsDetectorCacheChange {
    pub cache_change: ReaderCacheChange,
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
                if let Ok(publication_builtin_topic_data) =
                    PublicationBuiltinTopicData::deserialize_data(
                        message.cache_change.data_value.as_ref(),
                    )
                {
                    self.domain_participant
                        .add_discovered_writer(publication_builtin_topic_data.clone());
                    for subscriber in self.domain_participant.subscriber_list() {
                        for data_reader in subscriber.data_reader_list() {
                            message
                                .participant_address
                                .send_actor_mail(discovery_service::AddDiscoveredWriter {
                                    publication_builtin_topic_data: publication_builtin_topic_data
                                        .clone(),
                                    subscriber_handle: subscriber.instance_handle(),
                                    data_reader_handle: data_reader.instance_handle(),
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
                                    publication_handle: discovered_writer_handle.clone(),
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
    pub cache_change: ReaderCacheChange,
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
                                    subscription_handle: discovered_reader_handle.clone(),
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

pub struct AnnounceDeletedParticipant;
impl Mail for AnnounceDeletedParticipant {
    type Result = DdsResult<()>;
}
impl MailHandler<AnnounceDeletedParticipant> for DomainParticipantActor {
    fn handle(
        &mut self,
        _: AnnounceDeletedParticipant,
    ) -> <AnnounceDeletedParticipant as Mail>::Result {
        self.domain_participant.announce_deleted_participant()
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
            .transport_writer()
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

        Ok(data_reader.transport_reader().is_historical_data_received())
    }
}

pub struct RemoveWriterChange {
    pub publisher_handle: InstanceHandle,
    pub data_writer_handle: InstanceHandle,
    pub sequence_number: SequenceNumber,
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
