use super::{
    any_data_writer_listener::{AnyDataWriterListener, DataWriterListenerOperation},
    handle::DataWriterHandle,
    status_condition_actor::StatusConditionActor,
};
use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    dds_async::{publisher::PublisherAsync, topic::TopicAsync},
    implementation::{
        actor::ActorAddress,
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
        },
        data_representation_inline_qos::parameter_id_values::PID_KEY_HASH,
        runtime::{
            executor::{block_on, TaskHandle},
            mpsc::{mpsc_channel, MpscSender},
        },
        xtypes_glue::key_and_instance_handle::get_instance_handle_from_serialized_foo,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::{InstanceHandle, HANDLE_NIL},
        qos::{DataWriterQos, PublisherQos, TopicQos},
        qos_policy::{
            DurabilityQosPolicyKind, HistoryQosPolicyKind, Length, QosPolicyId,
            ReliabilityQosPolicyKind, DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, INVALID_QOS_POLICY_ID,
            LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID,
            PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
        status::{
            OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
            QosPolicyCount, StatusKind,
        },
        time::{DurationKind, Time},
    },
    rtps::{
        cache_change::RtpsCacheChange,
        messages::submessage_elements::{Parameter, ParameterList},
        reader_proxy::RtpsReaderProxy,
        stateful_writer::WriterHistoryCache,
        types::{
            ChangeKind, DurabilityKind, EntityId, Guid, Locator, ReliabilityKind, SequenceNumber,
            GUID_UNKNOWN, USER_DEFINED_UNKNOWN,
        },
    },
    xtypes::dynamic_type::DynamicType,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

struct MatchedSubscriptions {
    matched_subscription_list: HashMap<InstanceHandle, SubscriptionBuiltinTopicData>,
    total_count: i32,
    total_count_last_read: i32,
    current_count_last_read: i32,
    last_subscription_handle: InstanceHandle,
}

impl MatchedSubscriptions {
    fn new() -> Self {
        Self {
            matched_subscription_list: HashMap::new(),
            total_count: 0,
            total_count_last_read: 0,
            current_count_last_read: 0,
            last_subscription_handle: HANDLE_NIL,
        }
    }

    fn add_matched_subscription(
        &mut self,
        handle: InstanceHandle,
        subscription_data: SubscriptionBuiltinTopicData,
    ) {
        self.matched_subscription_list
            .insert(handle, subscription_data);
        self.total_count += 1;
        self.last_subscription_handle = handle;
    }

    fn remove_matched_subscription(&mut self, handle: InstanceHandle) {
        self.matched_subscription_list.remove(&handle);
    }

    fn get_matched_subscriptions(&self) -> Vec<InstanceHandle> {
        self.matched_subscription_list
            .iter()
            .map(|(&h, _)| h)
            .collect()
    }

    pub fn get_matched_subscription_data(
        &self,
        handle: InstanceHandle,
    ) -> Option<&SubscriptionBuiltinTopicData> {
        self.matched_subscription_list.get(&handle)
    }

    fn get_publication_matched_status(&mut self) -> PublicationMatchedStatus {
        let current_count = self.matched_subscription_list.len() as i32;
        let status = PublicationMatchedStatus {
            total_count: self.total_count,
            total_count_change: self.total_count - self.total_count_last_read,
            last_subscription_handle: self.last_subscription_handle,
            current_count,
            current_count_change: current_count - self.current_count_last_read,
        };

        self.total_count_last_read = self.total_count;
        self.current_count_last_read = current_count;

        status
    }
}

struct IncompatibleSubscriptions {
    incompatible_subscription_list: HashSet<InstanceHandle>,
    total_count: i32,
    total_count_last_read: i32,
    last_policy_id: QosPolicyId,
    policies: Vec<QosPolicyCount>,
}

impl IncompatibleSubscriptions {
    fn new() -> Self {
        Self {
            incompatible_subscription_list: HashSet::new(),
            total_count: 0,
            total_count_last_read: 0,
            last_policy_id: INVALID_QOS_POLICY_ID,
            policies: Vec::new(),
        }
    }

    fn add_offered_incompatible_qos(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        self.total_count += 1;
        self.last_policy_id = incompatible_qos_policy_list[0];

        self.incompatible_subscription_list.insert(handle);
        for incompatible_qos_policy in incompatible_qos_policy_list.into_iter() {
            if let Some(policy_count) = self
                .policies
                .iter_mut()
                .find(|x| x.policy_id == incompatible_qos_policy)
            {
                policy_count.count += 1;
            } else {
                self.policies.push(QosPolicyCount {
                    policy_id: incompatible_qos_policy,
                    count: 1,
                })
            }
        }
    }

    fn get_incompatible_subscriptions(&self) -> Vec<InstanceHandle> {
        self.incompatible_subscription_list
            .iter()
            .cloned()
            .collect()
    }

    fn get_offered_incompatible_qos_status(&mut self) -> OfferedIncompatibleQosStatus {
        let status = OfferedIncompatibleQosStatus {
            total_count: self.total_count,
            total_count_change: self.total_count - self.total_count_last_read,
            last_policy_id: self.last_policy_id,
            policies: self.policies.clone(),
        };

        self.total_count_last_read = self.total_count;

        status
    }

    fn contains(&self, handle: &InstanceHandle) -> bool {
        self.incompatible_subscription_list.contains(handle)
    }
}

struct DataWriterListenerMessage {
    listener_operation: DataWriterListenerOperation,
    writer_address: ActorAddress<DataWriterActor>,
    status_condition_address: ActorAddress<StatusConditionActor>,
    publisher: PublisherAsync,
    topic: TopicAsync,
}

struct DataWriterListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<DataWriterListenerMessage>,
}

impl DataWriterListenerThread {
    fn new(mut listener: Box<dyn AnyDataWriterListener + Send>) -> Self {
        let (sender, receiver) = mpsc_channel::<DataWriterListenerMessage>();
        let thread = std::thread::Builder::new()
            .name("Data writer listener".to_string())
            .spawn(move || {
                block_on(async {
                    while let Some(m) = receiver.recv().await {
                        listener
                            .call_listener_function(
                                m.listener_operation,
                                m.writer_address,
                                m.status_condition_address,
                                m.publisher,
                                m.topic,
                            )
                            .await;
                    }
                });
            })
            .expect("failed to spawn thread");
        Self { thread, sender }
    }

    fn sender(&self) -> &MpscSender<DataWriterListenerMessage> {
        &self.sender
    }

    fn join(self) -> DdsResult<()> {
        self.sender.close();
        self.thread.join()?;
        Ok(())
    }
}

pub struct DataWriterActor {
    transport_writer: Box<dyn WriterHistoryCache>,
    data_writer_handle: DataWriterHandle,
    topic_name: String,
    type_name: String,
    matched_subscriptions: MatchedSubscriptions,
    incompatible_subscriptions: IncompatibleSubscriptions,
    enabled: bool,
    status_condition: StatusConditionActor,
    data_writer_listener_thread: Option<DataWriterListenerThread>,
    status_kind: Vec<StatusKind>,
    max_seq_num: Option<SequenceNumber>,
    last_change_sequence_number: SequenceNumber,
    qos: DataWriterQos,
    registered_instance_list: HashSet<InstanceHandle>,
    offered_deadline_missed_status: OfferedDeadlineMissedStatus,
    instance_deadline_missed_task: HashMap<InstanceHandle, TaskHandle>,
    instance_samples: HashMap<InstanceHandle, VecDeque<SequenceNumber>>,
}

impl DataWriterActor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        transport_writer: Box<dyn WriterHistoryCache>,
        data_writer_handle: DataWriterHandle,
        topic_name: String,
        type_name: String,
        listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        status_kind: Vec<StatusKind>,
        qos: DataWriterQos,
    ) -> Self {
        let status_condition = StatusConditionActor::default();
        let data_writer_listener_thread = listener.map(DataWriterListenerThread::new);

        DataWriterActor {
            transport_writer,
            data_writer_handle,
            topic_name,
            type_name,
            matched_subscriptions: MatchedSubscriptions::new(),
            incompatible_subscriptions: IncompatibleSubscriptions::new(),
            enabled: false,
            status_condition,
            data_writer_listener_thread,
            status_kind,
            max_seq_num: None,
            last_change_sequence_number: 0,
            qos,
            registered_instance_list: HashSet::new(),
            offered_deadline_missed_status: OfferedDeadlineMissedStatus::default(),
            instance_deadline_missed_task: HashMap::new(),
            instance_samples: HashMap::new(),
        }
    }

    pub fn get_handle(&self) -> DataWriterHandle {
        self.data_writer_handle
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    fn on_publication_matched(
        &mut self,
        // data_writer_address: ActorAddress<DataWriterActor>,
        // publisher: PublisherAsync,
        // (publisher_listener, publisher_listener_mask): (
        //     Option<MpscSender<PublisherListenerMessage>>,
        //     Vec<StatusKind>,
        // ),
        // (participant_listener, participant_listener_mask): (
        //     Option<MpscSender<ParticipantListenerMessage>>,
        //     Vec<StatusKind>,
        // ),
    ) {
        // let type_name = self.type_name.clone();
        // let topic_name = self.topic_name.clone();
        // let participant = publisher.get_participant();
        // let status_condition_address = self.status_condition.address();
        // let topic_status_condition_address = self.topic_status_condition.clone();
        // let topic = TopicAsync::new(
        //     self.topic_address.clone(),
        //     topic_status_condition_address,
        //     type_name,
        //     topic_name,
        //     participant,
        // );
        // if self.status_kind.contains(&StatusKind::PublicationMatched) {
        //     let status = self.matched_subscriptions.get_publication_matched_status();
        //     if let Some(listener) = &self.data_writer_listener_thread {
        //         listener.sender().send(DataWriterListenerMessage {
        //             listener_operation: DataWriterListenerOperation::PublicationMatched(status),
        //             writer_address: data_writer_address,
        //             status_condition_address,
        //             publisher,
        //             topic,
        //         })?;
        //     }
        // } else if publisher_listener_mask.contains(&StatusKind::PublicationMatched) {
        //     let status = self.matched_subscriptions.get_publication_matched_status();
        //     if let Some(listener) = publisher_listener {
        //         listener.send(PublisherListenerMessage {
        //             listener_operation: PublisherListenerOperation::PublicationMatched(status),
        //             writer_address: data_writer_address,
        //             status_condition_address,
        //             publisher,
        //             topic,
        //         })?;
        //     }
        // } else if participant_listener_mask.contains(&StatusKind::PublicationMatched) {
        //     let status = self.matched_subscriptions.get_publication_matched_status();
        //     if let Some(listener) = participant_listener {
        //         listener.send(ParticipantListenerMessage {
        //             listener_operation: ParticipantListenerOperation::PublicationMatched(status),
        //             listener_kind: ListenerKind::Writer {
        //                 writer_address: data_writer_address,
        //                 status_condition_address,
        //                 publisher,
        //                 topic,
        //             },
        //         })?;
        //     }
        // }
        self.status_condition
            .add_communication_state(StatusKind::PublicationMatched);
    }

    fn on_offered_incompatible_qos(
        &mut self,
        // data_writer_address: ActorAddress<DataWriterActor>,
        // publisher: PublisherAsync,
        // (publisher_listener, publisher_listener_mask): (
        //     Option<MpscSender<PublisherListenerMessage>>,
        //     Vec<StatusKind>,
        // ),
        // (participant_listener, participant_listener_mask): (
        //     Option<MpscSender<ParticipantListenerMessage>>,
        //     Vec<StatusKind>,
        // ),
    ) {
        // let type_name = self.type_name.clone();
        // let topic_name = self.topic_name.clone();
        // let participant = publisher.get_participant();
        // let status_condition_address = self.status_condition.address();
        // let topic_status_condition_address = self.topic_status_condition.clone();
        // let topic = TopicAsync::new(
        //     self.topic_address.clone(),
        //     topic_status_condition_address,
        //     type_name,
        //     topic_name,
        //     participant,
        // );

        // if self
        //     .status_kind
        //     .contains(&StatusKind::OfferedIncompatibleQos)
        // {
        //     let status = self
        //         .incompatible_subscriptions
        //         .get_offered_incompatible_qos_status();
        //     if let Some(listener) = &self.data_writer_listener_thread {
        //         listener.sender().send(DataWriterListenerMessage {
        //             listener_operation: DataWriterListenerOperation::OfferedIncompatibleQos(status),
        //             writer_address: data_writer_address,
        //             status_condition_address,
        //             publisher,
        //             topic,
        //         })?;
        //     }
        // } else if publisher_listener_mask.contains(&StatusKind::OfferedIncompatibleQos) {
        //     let status = self
        //         .incompatible_subscriptions
        //         .get_offered_incompatible_qos_status();

        //     if let Some(listener) = publisher_listener {
        //         listener.send(PublisherListenerMessage {
        //             listener_operation: PublisherListenerOperation::OfferedIncompatibleQos(status),
        //             writer_address: data_writer_address,
        //             status_condition_address,
        //             publisher,
        //             topic,
        //         })?;
        //     }
        // } else if participant_listener_mask.contains(&StatusKind::OfferedIncompatibleQos) {
        //     let status = self
        //         .incompatible_subscriptions
        //         .get_offered_incompatible_qos_status();
        //     if let Some(listener) = participant_listener {
        //         listener.send(ParticipantListenerMessage {
        //             listener_operation: ParticipantListenerOperation::OfferedIncompatibleQos(
        //                 status,
        //             ),
        //             listener_kind: ListenerKind::Writer {
        //                 writer_address: data_writer_address,
        //                 status_condition_address,
        //                 publisher,
        //                 topic,
        //             },
        //         })?;
        //     }
        // }
        self.status_condition
            .add_communication_state(StatusKind::OfferedIncompatibleQos);
    }

    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
    }

    fn add_sample(&mut self, instance_handle: InstanceHandle, change: RtpsCacheChange) {
        if let HistoryQosPolicyKind::KeepLast(depth) = self.qos.history.kind {
            if let Some(s) = self.instance_samples.get_mut(&instance_handle) {
                if s.len() == depth as usize {
                    if let Some(smallest_seq_num_instance) = s.pop_front() {
                        self.transport_writer
                            .remove_change(smallest_seq_num_instance);
                    }
                }
            }
        }

        let change_timestamp = change.source_timestamp();
        let seq_num = change.sequence_number();

        if seq_num > self.max_seq_num.unwrap_or(0) {
            self.max_seq_num = Some(seq_num)
        }

        if let Some(t) = self
            .instance_deadline_missed_task
            .remove(&instance_handle.into())
        {
            t.abort();
        }

        if let DurationKind::Finite(deadline_missed_period) = self.qos.deadline.period {
            let deadline_missed_interval = std::time::Duration::new(
                deadline_missed_period.sec() as u64,
                deadline_missed_period.nanosec(),
            );
            // let writer_status_condition = self.status_condition.address();
            // let writer_address = message.writer_address.clone();
            // let timer_handle = message.timer_handle.clone();
            // let writer_listener_mask = self.status_kind.clone();
            // let data_writer_listener_sender = self
            //     .data_writer_listener_thread
            //     .as_ref()
            //     .map(|l| l.sender().clone());
            // let publisher_listener = message.publisher_mask_listener.0.clone();
            // let publisher_listener_mask = message.publisher_mask_listener.1.clone();
            // let participant_listener = message.participant_mask_listener.0.clone();
            // let participant_listener_mask = message.participant_mask_listener.1.clone();
            // let status_condition_address = self.status_condition.address();
            // // let topic_address = self.topic_address.clone();
            // // let topic_status_condition_address = self.topic_status_condition.clone();
            // let type_name = self.type_name.clone();
            // let topic_name = self.topic_name.clone();
            // let publisher = message.publisher.clone();

            // let deadline_missed_task = message.executor_handle.spawn(async move {
            //     loop {
            //         timer_handle.sleep(deadline_missed_interval).await;
            //         let publisher_listener = publisher_listener.clone();
            //         let participant_listener = participant_listener.clone();

            //         let r: DdsResult<()> = async {
            //             writer_address.send_actor_mail(
            //                 IncrementOfferedDeadlineMissedStatus {
            //                     instance_handle: change_instance_handle.into(),
            //                 },
            //             )?;

            //             let writer_address = writer_address.clone();
            //             let status_condition_address = status_condition_address.clone();
            //             let publisher = publisher.clone();
            //             let topic = TopicAsync::new(
            //                 topic_address.clone(),
            //                 topic_status_condition_address.clone(),
            //                 type_name.clone(),
            //                 topic_name.clone(),
            //                 publisher.get_participant(),
            //             );
            //             if writer_listener_mask.contains(&StatusKind::OfferedDeadlineMissed) {
            //                 let status = writer_address
            //                     .send_actor_mail(GetOfferedDeadlineMissedStatus)?
            //                     .receive_reply()
            //                     .await;
            //                 if let Some(listener) = &data_writer_listener_sender {
            //                     listener
            //                         .send(DataWriterListenerMessage {
            //                             listener_operation:
            //                                 DataWriterListenerOperation::OfferedDeadlineMissed(
            //                                     status,
            //                                 ),
            //                             writer_address,
            //                             status_condition_address,
            //                             publisher,
            //                             topic,
            //                         })
            //                         .ok();
            //                 }
            //             } else if publisher_listener_mask
            //                 .contains(&StatusKind::OfferedDeadlineMissed)
            //             {
            //                 let status = writer_address
            //                     .send_actor_mail(GetOfferedDeadlineMissedStatus)?
            //                     .receive_reply()
            //                     .await;
            //                 if let Some(listener) = publisher_listener {
            //                     listener
            //                         .send(PublisherListenerMessage {
            //                             listener_operation:
            //                                 PublisherListenerOperation::OfferedDeadlineMissed(
            //                                     status,
            //                                 ),
            //                             writer_address,
            //                             status_condition_address,
            //                             publisher,
            //                             topic,
            //                         })
            //                         .ok();
            //                 }
            //             } else if participant_listener_mask
            //                 .contains(&StatusKind::OfferedDeadlineMissed)
            //             {
            //                 let status = writer_address
            //                     .send_actor_mail(GetOfferedDeadlineMissedStatus)?
            //                     .receive_reply()
            //                     .await;
            //                 if let Some(listener) = participant_listener {
            //                     listener
            //                     .send(ParticipantListenerMessage {
            //                         listener_operation:
            //                             ParticipantListenerOperation::_OfferedDeadlineMissed(
            //                                 status,
            //                             ),
            //                         listener_kind: ListenerKind::Writer {
            //                             writer_address,
            //                             status_condition_address,
            //                             publisher,
            //                             topic,
            //                         },
            //                     })
            //                     .ok();
            //                 }
            //             }
            //             writer_status_condition
            //                 .send_actor_mail(AddCommunicationState {
            //                     state: StatusKind::OfferedDeadlineMissed,
            //                 })?
            //                 .receive_reply()
            //                 .await;
            //             Ok(())
            //         }
            //         .await;
            //         if r.is_err() {
            //             break;
            //         }
            //     }
            // });
            // self.instance_deadline_missed_task
            //     .insert(change_instance_handle.into(), deadline_missed_task);
        }

        // if let DurationKind::Finite(lifespan) = self.qos.lifespan.duration {
        //     if let Some(timestamp) = change_timestamp {
        //         let change_lifespan =
        //             crate::infrastructure::time::Time::from(timestamp) - message.now + lifespan;
        //         if change_lifespan > Duration::new(0, 0) {
        //             rtps_writer.get_history_cache().add_change(message.change);
        //             message.executor_handle.spawn(async move {
        //                 message.timer_handle.sleep(change_lifespan.into()).await;

        //                 message
        //                     .writer_address
        //                     .send_actor_mail(RemoveChange { seq_num })
        //                     .ok();
        //             });
        //         }
        //     }
        // } else {
        self.instance_samples
            .entry(instance_handle)
            .or_insert(VecDeque::new())
            .push_back(change.sequence_number);
        self.transport_writer.add_change(change);
        // }
    }

    pub fn write_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        timestamp: Time,
        type_support: &dyn DynamicType,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.last_change_sequence_number += 1;

        let key = get_instance_handle_from_serialized_foo(&serialized_data, type_support)?;

        let pid_key_hash = Parameter::new(PID_KEY_HASH, Arc::from(*key.as_ref()));
        let parameter_list = ParameterList::new(vec![pid_key_hash]);

        let change = RtpsCacheChange {
            kind: ChangeKind::Alive,
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            data_value: serialized_data.into(),
            inline_qos: parameter_list,
        };
        self.add_sample(key, change);
        Ok(())
    }

    pub fn add_matched_reader(
        &mut self,
        discovered_reader_data: &DiscoveredReaderData,
        publisher_qos: &PublisherQos,
    ) {
        let type_name = self.type_name.clone();
        let topic_name = self.topic_name.clone();
        let is_matched_topic_name = discovered_reader_data
            .subscription_builtin_topic_data()
            .topic_name()
            == topic_name;
        let is_matched_type_name = discovered_reader_data
            .subscription_builtin_topic_data()
            .get_type_name()
            == type_name;

        if is_matched_topic_name && is_matched_type_name {
            tracing::trace!(
                topic_name = topic_name,
                type_name = type_name,
                "Reader with matched topic and type found",
            );
            let incompatible_qos_policy_list = get_discovered_reader_incompatible_qos_policy_list(
                &self.qos,
                discovered_reader_data.subscription_builtin_topic_data(),
                &publisher_qos,
            );
            let instance_handle = InstanceHandle::new(
                discovered_reader_data
                    .subscription_builtin_topic_data()
                    .key
                    .value,
            );
            if incompatible_qos_policy_list.is_empty() {
                if !self
                    .matched_subscriptions
                    .get_matched_subscriptions()
                    .contains(&instance_handle)
                    || self
                        .matched_subscriptions
                        .get_matched_subscription_data(instance_handle)
                        != Some(discovered_reader_data.subscription_builtin_topic_data())
                {
                    let reliability_kind = match discovered_reader_data
                        .subscription_builtin_topic_data()
                        .reliability()
                        .kind
                    {
                        ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                        ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                    };
                    let durability_kind = match discovered_reader_data
                        .subscription_builtin_topic_data()
                        .durability()
                        .kind
                    {
                        DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                        DurabilityQosPolicyKind::TransientLocal => DurabilityKind::TransientLocal,
                        DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
                        DurabilityQosPolicyKind::Persistent => DurabilityKind::Persistent,
                    };

                    self.matched_subscriptions.add_matched_subscription(
                        instance_handle,
                        discovered_reader_data
                            .subscription_builtin_topic_data()
                            .clone(),
                    );
                    self.on_publication_matched(
                        // message.data_writer_address,
                        // // message.publisher,
                        // message.publisher_mask_listener,
                        // message.participant_mask_listener,
                    );
                }
            } else if !self.incompatible_subscriptions.contains(&instance_handle) {
                self.incompatible_subscriptions
                    .add_offered_incompatible_qos(instance_handle, incompatible_qos_policy_list);
                self.on_offered_incompatible_qos(
                    // message.data_writer_address,
                    // message.publisher,
                    // message.publisher_mask_listener,
                    // message.participant_mask_listener,
                );
            }
        }
    }

    pub fn as_publication_builtin_topic_data(
        &self,
        publisher_qos: &PublisherQos,
        topic_qos: &TopicQos,
    ) -> PublicationBuiltinTopicData {
        let type_name = self.type_name.clone();
        let topic_name = self.topic_name.clone();
        let writer_qos = &self.qos;

        PublicationBuiltinTopicData {
            key: BuiltInTopicKey {
                value: self.transport_writer.guid(),
            },
            participant_key: BuiltInTopicKey { value: [0; 16] },
            topic_name,
            type_name,
            durability: writer_qos.durability.clone(),
            deadline: writer_qos.deadline.clone(),
            latency_budget: writer_qos.latency_budget.clone(),
            liveliness: writer_qos.liveliness.clone(),
            reliability: writer_qos.reliability.clone(),
            lifespan: writer_qos.lifespan.clone(),
            user_data: writer_qos.user_data.clone(),
            ownership: writer_qos.ownership.clone(),
            ownership_strength: writer_qos.ownership_strength.clone(),
            destination_order: writer_qos.destination_order.clone(),
            presentation: publisher_qos.presentation.clone(),
            partition: publisher_qos.partition.clone(),
            topic_data: topic_qos.topic_data.clone(),
            group_data: publisher_qos.group_data.clone(),
            representation: writer_qos.representation.clone(),
        }
    }

    pub fn remove_matched_reader(&mut self, discovered_reader_handle: InstanceHandle) {
        if let Some(r) = self
            .matched_subscriptions
            .get_matched_subscription_data(discovered_reader_handle)
        {
            // let handle = r.key().value.into();
            self.matched_subscriptions
                .remove_matched_subscription(InstanceHandle::new(todo!()));

            self.on_publication_matched(
            // message.data_writer_address,
            // message.publisher,
            // message.publisher_mask_listener,
            // message.participant_mask_listener,
            );
        }
    }

    pub fn add_matched_publication(
        &mut self,
        handle: InstanceHandle,
        subscription_data: SubscriptionBuiltinTopicData,
    ) {
        self.matched_subscriptions
            .add_matched_subscription(handle, subscription_data);
    }

    pub fn remove_matched_subscription(&mut self, handle: InstanceHandle) {
        self.matched_subscriptions
            .remove_matched_subscription(handle)
    }

    pub fn get_matched_subscriptions(&self) -> Vec<InstanceHandle> {
        self.matched_subscriptions.get_matched_subscriptions()
    }

    pub fn get_matched_subscription_data(
        &self,
        handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        self.matched_subscriptions
            .get_matched_subscription_data(handle)
            .ok_or(DdsError::BadParameter)
            .cloned()
    }

    pub fn get_offered_incompatible_qos_status(&mut self) -> OfferedIncompatibleQosStatus {
        self.incompatible_subscriptions
            .get_offered_incompatible_qos_status()
    }

    pub fn get_incompatible_subscriptions(&mut self) -> Vec<InstanceHandle> {
        self.incompatible_subscriptions
            .get_incompatible_subscriptions()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_qos(&self) -> &DataWriterQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: DataWriterQos) -> DdsResult<()> {
        qos.is_consistent()?;
        if self.enabled {
            qos.check_immutability(&self.qos)?;
        }
        self.qos = qos;
        Ok(())
    }

    pub fn register_instance_w_timestamp(
        &mut self,
        instance_handle: InstanceHandle,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !self.registered_instance_list.contains(&instance_handle) {
            if self.registered_instance_list.len() < self.qos.resource_limits.max_instances {
                self.registered_instance_list.insert(instance_handle);
            } else {
                return Err(DdsError::OutOfResources);
            }
        }
        Ok(Some(instance_handle))
    }

    pub fn lookup_instance(
        &self,
        instance_handle: InstanceHandle,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        Ok(
            if self.registered_instance_list.contains(&instance_handle) {
                Some(instance_handle)
            } else {
                None
            },
        )
    }

    pub fn are_all_changes_acknowledged(&mut self) -> bool {
        self.transport_writer.are_all_changes_acknowledged()
    }

    pub fn get_publication_matched_status(&mut self) -> PublicationMatchedStatus {
        self.status_condition
            .remove_communication_state(StatusKind::PublicationMatched);

        self.matched_subscriptions.get_publication_matched_status()
    }

    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }

    pub fn set_listener(
        &mut self,
        listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        status_kind: Vec<StatusKind>,
    ) -> DdsResult<()> {
        if let Some(listener) = self.data_writer_listener_thread.take() {
            listener.join()?;
        }

        self.data_writer_listener_thread = listener.map(DataWriterListenerThread::new);
        self.status_kind = status_kind;
        Ok(())
    }

    pub fn remove_change(&mut self, seq_num: SequenceNumber) {
        self.transport_writer.remove_change(seq_num);
    }

    pub fn is_resources_limit_reached(&self, instance_handle: InstanceHandle) -> bool {
        if let Length::Limited(max_instances) = self.qos.resource_limits.max_instances {
            if !self.instance_samples.contains_key(&instance_handle)
                && self.instance_samples.len() == max_instances as usize
            {
                return true;
            }
        }

        if let Length::Limited(max_samples_per_instance) =
            self.qos.resource_limits.max_samples_per_instance
        {
            // If the history Qos guarantess that the number of samples
            // is below the limit there is no need to check
            match self.qos.history.kind {
                HistoryQosPolicyKind::KeepLast(depth) if depth <= max_samples_per_instance => {}
                _ => {
                    if let Some(s) = self.instance_samples.get(&instance_handle) {
                        // Only Alive changes count towards the resource limits
                        if s.len() >= max_samples_per_instance as usize {
                            return true;
                        }
                    }
                }
            }
        }

        if let Length::Limited(max_samples) = self.qos.resource_limits.max_samples {
            let total_samples = self
                .instance_samples
                .iter()
                .fold(0, |acc, (_, x)| acc + x.len());

            if total_samples >= max_samples as usize {
                return true;
            }
        }

        false
    }

    pub fn is_data_lost_after_adding_change(&self, instance_handle: InstanceHandle) -> bool {
        if let HistoryQosPolicyKind::KeepLast(depth) = self.qos.history.kind {
            if let Some(s) = self.instance_samples.get(&instance_handle) {
                if s.len() == depth as usize {
                    if !self.transport_writer.are_all_changes_acknowledged() {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn increment_offered_deadline_missed_status(&mut self, instance_handle: InstanceHandle) {
        self.offered_deadline_missed_status.total_count += 1;
        self.offered_deadline_missed_status.total_count_change += 1;
        self.offered_deadline_missed_status.last_instance_handle = instance_handle;
    }

    pub fn get_offered_deadline_missed_status(&mut self) -> OfferedDeadlineMissedStatus {
        let status = self.offered_deadline_missed_status.clone();
        self.offered_deadline_missed_status.total_count_change = 0;
        status
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
