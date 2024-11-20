use fnmatch_regex::glob_to_regex;

use crate::{
    builtin_topics::{
        BuiltInTopicKey, ParticipantBuiltinTopicData, PublicationBuiltinTopicData,
        SubscriptionBuiltinTopicData, DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION,
        DCPS_TOPIC,
    },
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
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
        qos::{DataWriterQos, PublisherQos},
        qos_policy::{
            DurabilityQosPolicyKind, QosPolicyId, DATA_REPRESENTATION_QOS_POLICY_ID,
            DEADLINE_QOS_POLICY_ID, DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID,
            LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID,
            PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
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
    topic_definition::type_support::{DdsDeserialize, DdsSerialize},
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
        if let Some(mut reader) = self
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
}
impl Mail for AddBuiltinPublicationsDetectorCacheChange {
    type Result = ();
}
impl MailHandler<AddBuiltinPublicationsDetectorCacheChange> for DomainParticipantActor {
    fn handle(
        &mut self,
        message: AddBuiltinPublicationsDetectorCacheChange,
    ) -> <AddBuiltinPublicationsDetectorCacheChange as Mail>::Result {
        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber_mut()
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_PUBLICATION)
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
                            if let Ok(discovered_writer_data) =
                                DiscoveredWriterData::deserialize_data(
                                    sample_data
                                        .expect("Alive samples must contain data")
                                        .as_ref(),
                                )
                            {
                                todo!()
                                // self.add_discovered_writer(discovered_writer_data);
                            }
                        }
                        InstanceStateKind::NotAliveDisposed => {
                            todo!()
                            // self.remove_discovered_writer(sample_info.instance_handle)
                        }
                        InstanceStateKind::NotAliveNoWriters => (),
                    }
                }
            }
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
                    self.domain_participant.process_discovered_readers();
                }
            }
            ChangeKind::AliveFiltered => todo!(),
            ChangeKind::NotAliveDisposed => todo!(),
            ChangeKind::NotAliveUnregistered => todo!(),
            ChangeKind::NotAliveDisposedUnregistered => todo!(),
        }

        if let Some(reader) = self
            .domain_participant
            .builtin_subscriber_mut()
            .data_reader_list_mut()
            .find(|dr| dr.topic_name() == DCPS_SUBSCRIPTION)
        {
            reader.add_reader_change(message.cache_change).ok();
        }

        // todo!();
        // self.status_condition
        //     .send_actor_mail(status_condition_actor::AddCommunicationState {
        //         state: StatusKind::DataOnReaders,
        //     });

        // if let Ok(samples) = reader.read(
        //     i32::MAX,
        //     &[SampleStateKind::NotRead],
        //     ANY_VIEW_STATE,
        //     ANY_INSTANCE_STATE,
        //     None,
        // ) {
        //     for (sample_data, sample_info) in samples {
        //         match sample_info.instance_state {
        //             InstanceStateKind::Alive => {
        //                 if let Ok(discovered_reader_data) =
        //                     DiscoveredReaderData::deserialize_data(
        //                         sample_data
        //                             .expect("Alive samples must contain data")
        //                             .as_ref(),
        //                     )
        //                 {
        //                     todo!()
        //                     // self.add_discovered_reader(discovered_reader_data);
        //                 }
        //             }
        //             InstanceStateKind::NotAliveDisposed => {
        //                 // for publisher in self.user_defined_publisher_list.iter_mut() {
        //                 // for data_writer in publisher.data_writer_list_mut() {
        //                 todo!()
        //                 // if let Some(r) = data_writer
        //                 //     .matched_subscription_list
        //                 //     .remove(&sample_info.instance_handle)
        //                 // {
        //                 // let type_name = self.type_name.clone();
        //                 // let topic_name = self.topic_name.clone();
        //                 // let participant = publisher.get_participant();
        //                 // let status_condition_address = self.status_condition.address();
        //                 // let topic_status_condition_address = self.topic_status_condition.clone();
        //                 // let topic = TopicAsync::new(
        //                 //     self.topic_address.clone(),
        //                 //     topic_status_condition_address,
        //                 //     type_name,
        //                 //     topic_name,
        //                 //     participant,
        //                 // );
        //                 // if self.status_kind.contains(&StatusKind::PublicationMatched) {
        //                 //     let status = self.matched_subscriptions.get_publication_matched_status();
        //                 //     if let Some(listener) = &self.data_writer_listener_thread {
        //                 //         listener.sender().send(DataWriterListenerMessage {
        //                 //             listener_operation: DataWriterListenerOperation::PublicationMatched(status),
        //                 //             writer_address: data_writer_address,
        //                 //             status_condition_address,
        //                 //             publisher,
        //                 //             topic,
        //                 //         })?;
        //                 //     }
        //                 // } else if publisher_listener_mask.contains(&StatusKind::PublicationMatched) {
        //                 //     let status = self.matched_subscriptions.get_publication_matched_status();
        //                 //     if let Some(listener) = publisher_listener {
        //                 //         listener.send(PublisherListenerMessage {
        //                 //             listener_operation: PublisherListenerOperation::PublicationMatched(status),
        //                 //             writer_address: data_writer_address,
        //                 //             status_condition_address,
        //                 //             publisher,
        //                 //             topic,
        //                 //         })?;
        //                 //     }
        //                 // } else if participant_listener_mask.contains(&StatusKind::PublicationMatched) {
        //                 //     let status = self.matched_subscriptions.get_publication_matched_status();
        //                 //     if let Some(listener) = participant_listener {
        //                 //         listener.send(ParticipantListenerMessage {
        //                 //             listener_operation: ParticipantListenerOperation::PublicationMatched(status),
        //                 //             listener_kind: ListenerKind::Writer {
        //                 //                 writer_address: data_writer_address,
        //                 //                 status_condition_address,
        //                 //                 publisher,
        //                 //                 topic,
        //                 //             },
        //                 //         })?;
        //                 //     }
        //                 // }
        //                 //     data_writer.status_condition.send_actor_mail(
        //                 //         status_condition_actor::AddCommunicationState {
        //                 //             state: StatusKind::PublicationMatched,
        //                 //         },
        //                 //     );
        //                 // }
        //                 // }
        //                 // }
        //             }
        //             InstanceStateKind::NotAliveNoWriters => (),
        //         }
        //     }
        // }
        // }
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
