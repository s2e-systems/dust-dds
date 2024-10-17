use super::{
    any_data_writer_listener::{AnyDataWriterListener, DataWriterListenerOperation},
    domain_participant_backend::{
        ListenerKind, ParticipantListenerMessage, ParticipantListenerOperation,
    },
    message_sender_actor::MessageSenderActor,
    publisher_actor::{PublisherListenerMessage, PublisherListenerOperation},
    status_condition_actor::{self, AddCommunicationState, StatusConditionActor},
    topic_actor::TopicActor,
};
use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    dds_async::{publisher::PublisherAsync, topic::TopicAsync},
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
        },
        data_representation_inline_qos::parameter_id_values::PID_KEY_HASH,
        runtime::{
            executor::{block_on, ExecutorHandle, TaskHandle},
            mpsc::{mpsc_channel, MpscSender},
            timer::TimerHandle,
        },
        xtypes_glue::key_and_instance_handle::get_instance_handle_from_serialized_foo,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::{InstanceHandle, HANDLE_NIL},
        qos::{DataWriterQos, PublisherQos},
        qos_policy::{
            DurabilityQosPolicyKind, HistoryQosPolicyKind, Length, QosPolicyId,
            ReliabilityQosPolicyKind, TopicDataQosPolicy, DATA_REPRESENTATION_QOS_POLICY_ID,
            DEADLINE_QOS_POLICY_ID, DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID,
            INVALID_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID,
            OWNERSHIP_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID,
            XCDR_DATA_REPRESENTATION,
        },
        status::{
            OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
            QosPolicyCount, StatusKind,
        },
        time::{Duration, DurationKind, Time},
    },
    rtps::{
        cache_change::RtpsCacheChange,
        messages::submessage_elements::{Data, Parameter, ParameterList},
        reader_proxy::RtpsReaderProxy,
        stateful_writer::TransportWriter,
        types::{
            ChangeKind, DurabilityKind, EntityId, Guid, Locator, ReliabilityKind, SequenceNumber,
            GUID_UNKNOWN, USER_DEFINED_UNKNOWN,
        },
    },
    topic_definition::type_support::DdsSerialize,
    xtypes::dynamic_type::DynamicType,
};
use std::{
    collections::{HashMap, HashSet},
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
    rtps_writer: Arc<Mutex<dyn TransportWriter + Send + Sync + 'static>>,
    guid: Guid,
    heartbeat_period: Duration,
    topic_name: String,
    type_name: String,
    matched_subscriptions: MatchedSubscriptions,
    incompatible_subscriptions: IncompatibleSubscriptions,
    enabled: bool,
    status_condition: Actor<StatusConditionActor>,
    data_writer_listener_thread: Option<DataWriterListenerThread>,
    status_kind: Vec<StatusKind>,
    max_seq_num: Option<SequenceNumber>,
    last_change_sequence_number: SequenceNumber,
    qos: DataWriterQos,
    registered_instance_list: HashSet<InstanceHandle>,
    offered_deadline_missed_status: OfferedDeadlineMissedStatus,
    instance_deadline_missed_task: HashMap<InstanceHandle, TaskHandle>,
}

impl DataWriterActor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rtps_writer: Arc<Mutex<dyn TransportWriter + Send + Sync + 'static>>,
        guid: Guid,
        heartbeat_period: Duration,
        topic_name: String,
        type_name: String,
        listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        status_kind: Vec<StatusKind>,
        qos: DataWriterQos,
        handle: &ExecutorHandle,
    ) -> Self {
        let status_condition = Actor::spawn(StatusConditionActor::default(), handle);
        let data_writer_listener_thread = listener.map(DataWriterListenerThread::new);

        DataWriterActor {
            rtps_writer,
            guid,
            heartbeat_period,
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
        }
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        InstanceHandle::new(self.guid.into())
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
    ) -> DdsResult<()> {
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
            .send_actor_mail(AddCommunicationState {
                state: StatusKind::PublicationMatched,
            });

        Ok(())
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
    ) -> DdsResult<()> {
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
            .send_actor_mail(AddCommunicationState {
                state: StatusKind::OfferedIncompatibleQos,
            });
        Ok(())
    }

    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
    }

    fn add_change(&mut self, change: RtpsCacheChange) {
        let mut rtps_writer = self.rtps_writer.lock().unwrap();
        if let HistoryQosPolicyKind::KeepLast(depth) = self.qos.history.kind {
            if rtps_writer
                .get_history_cache()
                .get_changes()
                .iter()
                .filter(|cc| cc.instance_handle() == change.instance_handle())
                .count()
                == depth as usize
            {
                if let Some(smallest_seq_num_instance) = rtps_writer
                    .get_history_cache()
                    .get_changes()
                    .iter()
                    .filter(|cc| cc.instance_handle() == change.instance_handle())
                    .map(|cc| cc.sequence_number())
                    .min()
                {
                    rtps_writer
                        .get_history_cache()
                        .remove_change(smallest_seq_num_instance);
                }
            }
        }

        let change_instance_handle = change.instance_handle();
        let change_timestamp = change.source_timestamp();
        let seq_num = change.sequence_number();

        if seq_num > self.max_seq_num.unwrap_or(0) {
            self.max_seq_num = Some(seq_num)
        }

        if let Some(t) = self
            .instance_deadline_missed_task
            .remove(&change_instance_handle.into())
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
        rtps_writer.get_history_cache().add_change(change);
        // }
    }

    pub fn write_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        timestamp: Time,
        type_support: &dyn DynamicType,
    ) -> DdsResult<()> {
        self.last_change_sequence_number += 1;

        let key = get_instance_handle_from_serialized_foo(&serialized_data, type_support)?;

        // let pid_key_hash = Parameter::new(PID_KEY_HASH, Arc::from(*instance_handle.as_ref()));
        // let parameter_list = ParameterList::new(vec![pid_key_hash]);

        let change = RtpsCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: self.guid,
            instance_handle: key.into(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            data_value: serialized_data.into(),
            inline_qos: ParameterList::new(vec![]),
        };
        self.add_change(change);
        Ok(())
    }

    pub fn add_matched_reader(
        &mut self,
        discovered_reader_data: DiscoveredReaderData,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
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
                let unicast_locator_list = if discovered_reader_data
                    .reader_proxy()
                    .unicast_locator_list
                    .is_empty()
                {
                    default_unicast_locator_list
                } else {
                    discovered_reader_data
                        .reader_proxy()
                        .unicast_locator_list
                        .to_vec()
                };

                let multicast_locator_list = if discovered_reader_data
                    .reader_proxy()
                    .multicast_locator_list
                    .is_empty()
                {
                    default_multicast_locator_list
                } else {
                    discovered_reader_data
                        .reader_proxy()
                        .multicast_locator_list
                        .to_vec()
                };

                let proxy_reliability = match discovered_reader_data
                    .subscription_builtin_topic_data()
                    .reliability()
                    .kind
                {
                    ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                    ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                };

                let first_relevant_sample_seq_num = match discovered_reader_data
                    .subscription_builtin_topic_data()
                    .durability()
                    .kind
                {
                    DurabilityQosPolicyKind::Volatile => self.max_seq_num.unwrap_or(0),
                    DurabilityQosPolicyKind::TransientLocal
                    | DurabilityQosPolicyKind::Transient
                    | DurabilityQosPolicyKind::Persistent => 0,
                };

                let reader_proxy = RtpsReaderProxy::new(
                    discovered_reader_data.reader_proxy().remote_reader_guid,
                    discovered_reader_data.reader_proxy().remote_group_entity_id,
                    &unicast_locator_list,
                    &multicast_locator_list,
                    discovered_reader_data.reader_proxy().expects_inline_qos,
                    true,
                    proxy_reliability,
                    first_relevant_sample_seq_num,
                );

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
                    self.rtps_writer.lock().unwrap().add_matched_reader(
                        discovered_reader_data.reader_proxy().clone(),
                        reliability_kind,
                        durability_kind,
                    );
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
}

pub struct GetInstanceHandle;
impl Mail for GetInstanceHandle {
    type Result = InstanceHandle;
}
impl MailHandler<GetInstanceHandle> for DataWriterActor {
    fn handle(&mut self, _: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
        InstanceHandle::new(self.guid.into())
    }
}

pub struct AddMatchedPublication {
    pub handle: InstanceHandle,
    pub subscription_data: SubscriptionBuiltinTopicData,
}
impl Mail for AddMatchedPublication {
    type Result = ();
}
impl MailHandler<AddMatchedPublication> for DataWriterActor {
    fn handle(
        &mut self,
        message: AddMatchedPublication,
    ) -> <AddMatchedPublication as Mail>::Result {
        self.matched_subscriptions
            .add_matched_subscription(message.handle, message.subscription_data);
    }
}

pub struct RemoveMatchedSubscription {
    pub handle: InstanceHandle,
}
impl Mail for RemoveMatchedSubscription {
    type Result = ();
}
impl MailHandler<RemoveMatchedSubscription> for DataWriterActor {
    fn handle(
        &mut self,
        message: RemoveMatchedSubscription,
    ) -> <RemoveMatchedSubscription as Mail>::Result {
        self.matched_subscriptions
            .remove_matched_subscription(message.handle)
    }
}

pub struct GetMatchedSubscriptions;
impl Mail for GetMatchedSubscriptions {
    type Result = Vec<InstanceHandle>;
}
impl MailHandler<GetMatchedSubscriptions> for DataWriterActor {
    fn handle(&mut self, _: GetMatchedSubscriptions) -> <GetMatchedSubscriptions as Mail>::Result {
        self.matched_subscriptions.get_matched_subscriptions()
    }
}

pub struct GetMatchedSubscriptionData {
    pub handle: InstanceHandle,
}
impl Mail for GetMatchedSubscriptionData {
    type Result = Option<SubscriptionBuiltinTopicData>;
}
impl MailHandler<GetMatchedSubscriptionData> for DataWriterActor {
    fn handle(
        &mut self,
        message: GetMatchedSubscriptionData,
    ) -> <GetMatchedSubscriptionData as Mail>::Result {
        self.matched_subscriptions
            .get_matched_subscription_data(message.handle)
            .cloned()
    }
}

pub struct GetOfferedIncompatibleQosStatus;
impl Mail for GetOfferedIncompatibleQosStatus {
    type Result = OfferedIncompatibleQosStatus;
}
impl MailHandler<GetOfferedIncompatibleQosStatus> for DataWriterActor {
    fn handle(
        &mut self,
        _: GetOfferedIncompatibleQosStatus,
    ) -> <GetOfferedIncompatibleQosStatus as Mail>::Result {
        self.incompatible_subscriptions
            .get_offered_incompatible_qos_status()
    }
}

pub struct GetIncompatibleSubscriptions;
impl Mail for GetIncompatibleSubscriptions {
    type Result = Vec<InstanceHandle>;
}
impl MailHandler<GetIncompatibleSubscriptions> for DataWriterActor {
    fn handle(
        &mut self,
        _: GetIncompatibleSubscriptions,
    ) -> <GetIncompatibleSubscriptions as Mail>::Result {
        self.incompatible_subscriptions
            .get_incompatible_subscriptions()
    }
}

pub struct Enable {
    pub data_writer_address: ActorAddress<DataWriterActor>,
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
    pub executor_handle: ExecutorHandle,
    pub timer_handle: TimerHandle,
}
impl Mail for Enable {
    type Result = ();
}
impl MailHandler<Enable> for DataWriterActor {
    fn handle(&mut self, message: Enable) -> <Enable as Mail>::Result {
        self.enabled = true;

        if self.qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let half_heartbeat_period =
                std::time::Duration::from(Duration::from(self.heartbeat_period)) / 2;
            let message_sender_actor = message.message_sender_actor;
            let data_writer_address = message.data_writer_address;
            let timer_handle = message.timer_handle;
            message.executor_handle.spawn(async move {
                loop {
                    timer_handle.sleep(half_heartbeat_period).await;

                    // let r =data_writer_address.send_actor_mail(SendMessage {
                    //     message_sender_actor: message_sender_actor.clone(),
                    // });
                    // if r.is_err() {
                    // break;
                    // }
                }
            });
        }
    }
}

pub struct IsEnabled;
impl Mail for IsEnabled {
    type Result = bool;
}
impl MailHandler<IsEnabled> for DataWriterActor {
    fn handle(&mut self, _: IsEnabled) -> <IsEnabled as Mail>::Result {
        self.enabled
    }
}

pub struct GetStatuscondition;
impl Mail for GetStatuscondition {
    type Result = ActorAddress<StatusConditionActor>;
}
impl MailHandler<GetStatuscondition> for DataWriterActor {
    fn handle(&mut self, _: GetStatuscondition) -> <GetStatuscondition as Mail>::Result {
        self.status_condition.address()
    }
}

pub struct GetGuid;
impl Mail for GetGuid {
    type Result = Guid;
}
impl MailHandler<GetGuid> for DataWriterActor {
    fn handle(&mut self, _: GetGuid) -> <GetGuid as Mail>::Result {
        self.guid
    }
}

pub struct GetQos;
impl Mail for GetQos {
    type Result = DataWriterQos;
}
impl MailHandler<GetQos> for DataWriterActor {
    fn handle(&mut self, _: GetQos) -> <GetQos as Mail>::Result {
        self.qos.clone()
    }
}

pub struct SetQos {
    pub qos: DataWriterQos,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for DataWriterActor {
    fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        message.qos.is_consistent()?;
        if self.enabled {
            message.qos.check_immutability(&self.qos)?;
        }
        self.qos = message.qos;
        Ok(())
    }
}

pub struct RegisterInstanceWTimestamp {
    pub instance_handle: InstanceHandle,
}
impl Mail for RegisterInstanceWTimestamp {
    type Result = DdsResult<Option<InstanceHandle>>;
}
impl MailHandler<RegisterInstanceWTimestamp> for DataWriterActor {
    fn handle(
        &mut self,
        message: RegisterInstanceWTimestamp,
    ) -> <RegisterInstanceWTimestamp as Mail>::Result {
        if !self
            .registered_instance_list
            .contains(&message.instance_handle)
        {
            if self.registered_instance_list.len() < self.qos.resource_limits.max_instances {
                self.registered_instance_list
                    .insert(message.instance_handle);
            } else {
                return Err(DdsError::OutOfResources);
            }
        }
        Ok(Some(message.instance_handle))
    }
}

pub struct LookupInstance {
    pub instance_handle: InstanceHandle,
}
impl Mail for LookupInstance {
    type Result = DdsResult<Option<InstanceHandle>>;
}
impl MailHandler<LookupInstance> for DataWriterActor {
    fn handle(&mut self, message: LookupInstance) -> <LookupInstance as Mail>::Result {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        Ok(
            if self
                .registered_instance_list
                .contains(&message.instance_handle)
            {
                Some(message.instance_handle)
            } else {
                None
            },
        )
    }
}

pub struct AreAllChangesAcknowledge;
impl Mail for AreAllChangesAcknowledge {
    type Result = bool;
}
impl MailHandler<AreAllChangesAcknowledge> for DataWriterActor {
    fn handle(
        &mut self,
        _: AreAllChangesAcknowledge,
    ) -> <AreAllChangesAcknowledge as Mail>::Result {
        self.rtps_writer
            .lock()
            .unwrap()
            .are_all_changes_acknowledged()
    }
}

pub struct AsDiscoveredWriterData {
    pub publisher_qos: PublisherQos,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub topic_data: TopicDataQosPolicy,
    pub xml_type: String,
}
impl Mail for AsDiscoveredWriterData {
    type Result = DdsResult<DiscoveredWriterData>;
}
impl MailHandler<AsDiscoveredWriterData> for DataWriterActor {
    fn handle(
        &mut self,
        message: AsDiscoveredWriterData,
    ) -> <AsDiscoveredWriterData as Mail>::Result {
        let type_name = self.type_name.clone();
        let topic_name = self.topic_name.clone();
        let writer_qos = &self.qos;

        Ok(DiscoveredWriterData {
            dds_publication_data: PublicationBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.guid.into(),
                },
                participant_key: BuiltInTopicKey {
                    value: GUID_UNKNOWN.into(),
                },
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
                presentation: message.publisher_qos.presentation,
                partition: message.publisher_qos.partition,
                topic_data: message.topic_data,
                group_data: message.publisher_qos.group_data,
                xml_type: message.xml_type,
                representation: writer_qos.representation.clone(),
            },
            writer_proxy: WriterProxy {
                remote_writer_guid: self.guid,
                remote_group_entity_id: EntityId::new([0; 3], USER_DEFINED_UNKNOWN),
                unicast_locator_list: message.default_unicast_locator_list,
                multicast_locator_list: message.default_multicast_locator_list,
                data_max_size_serialized: Default::default(),
            },
        })
    }
}

pub struct GetPublicationMatchedStatus;
impl Mail for GetPublicationMatchedStatus {
    type Result = PublicationMatchedStatus;
}
impl MailHandler<GetPublicationMatchedStatus> for DataWriterActor {
    fn handle(
        &mut self,
        _: GetPublicationMatchedStatus,
    ) -> <GetPublicationMatchedStatus as Mail>::Result {
        self.status_condition
            .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                state: StatusKind::PublicationMatched,
            });

        self.matched_subscriptions.get_publication_matched_status()
    }
}

pub struct GetTopicName;
impl Mail for GetTopicName {
    type Result = DdsResult<String>;
}
impl MailHandler<GetTopicName> for DataWriterActor {
    fn handle(&mut self, _: GetTopicName) -> <GetTopicName as Mail>::Result {
        Ok(self.topic_name.clone())
    }
}

pub struct GetTypeName;
impl Mail for GetTypeName {
    type Result = DdsResult<String>;
}
impl MailHandler<GetTypeName> for DataWriterActor {
    fn handle(&mut self, _: GetTypeName) -> <GetTypeName as Mail>::Result {
        Ok(self.type_name.clone())
    }
}

pub struct AddMatchedReader {
    pub discovered_reader_data: DiscoveredReaderData,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub data_writer_address: ActorAddress<DataWriterActor>,
    // pub publisher: PublisherAsync,
    pub publisher_qos: PublisherQos,
    pub publisher_mask_listener: (
        Option<MpscSender<PublisherListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
}
impl Mail for AddMatchedReader {
    type Result = DdsResult<()>;
}
impl MailHandler<AddMatchedReader> for DataWriterActor {
    fn handle(&mut self, message: AddMatchedReader) -> <AddMatchedReader as Mail>::Result {
        let type_name = self.type_name.clone();
        let topic_name = self.topic_name.clone();
        let is_matched_topic_name = message
            .discovered_reader_data
            .subscription_builtin_topic_data()
            .topic_name()
            == topic_name;
        let is_matched_type_name = message
            .discovered_reader_data
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
                message
                    .discovered_reader_data
                    .subscription_builtin_topic_data(),
                &message.publisher_qos,
            );
            let instance_handle = InstanceHandle::new(
                message
                    .discovered_reader_data
                    .subscription_builtin_topic_data()
                    .key
                    .value,
            );
            if incompatible_qos_policy_list.is_empty() {
                let unicast_locator_list = if message
                    .discovered_reader_data
                    .reader_proxy()
                    .unicast_locator_list
                    .is_empty()
                {
                    message.default_unicast_locator_list
                } else {
                    message
                        .discovered_reader_data
                        .reader_proxy()
                        .unicast_locator_list
                        .to_vec()
                };

                let multicast_locator_list = if message
                    .discovered_reader_data
                    .reader_proxy()
                    .multicast_locator_list
                    .is_empty()
                {
                    message.default_multicast_locator_list
                } else {
                    message
                        .discovered_reader_data
                        .reader_proxy()
                        .multicast_locator_list
                        .to_vec()
                };

                let proxy_reliability = match message
                    .discovered_reader_data
                    .subscription_builtin_topic_data()
                    .reliability()
                    .kind
                {
                    ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                    ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                };

                let first_relevant_sample_seq_num = match message
                    .discovered_reader_data
                    .subscription_builtin_topic_data()
                    .durability()
                    .kind
                {
                    DurabilityQosPolicyKind::Volatile => self.max_seq_num.unwrap_or(0),
                    DurabilityQosPolicyKind::TransientLocal
                    | DurabilityQosPolicyKind::Transient
                    | DurabilityQosPolicyKind::Persistent => 0,
                };

                let reader_proxy = RtpsReaderProxy::new(
                    message
                        .discovered_reader_data
                        .reader_proxy()
                        .remote_reader_guid,
                    message
                        .discovered_reader_data
                        .reader_proxy()
                        .remote_group_entity_id,
                    &unicast_locator_list,
                    &multicast_locator_list,
                    message
                        .discovered_reader_data
                        .reader_proxy()
                        .expects_inline_qos,
                    true,
                    proxy_reliability,
                    first_relevant_sample_seq_num,
                );

                if !self
                    .matched_subscriptions
                    .get_matched_subscriptions()
                    .contains(&instance_handle)
                    || self
                        .matched_subscriptions
                        .get_matched_subscription_data(instance_handle)
                        != Some(
                            message
                                .discovered_reader_data
                                .subscription_builtin_topic_data(),
                        )
                {
                    let reliability_kind = match message
                        .discovered_reader_data
                        .subscription_builtin_topic_data()
                        .reliability()
                        .kind
                    {
                        ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                        ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                    };
                    let durability_kind = match message
                        .discovered_reader_data
                        .subscription_builtin_topic_data()
                        .durability()
                        .kind
                    {
                        DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
                        DurabilityQosPolicyKind::TransientLocal => DurabilityKind::TransientLocal,
                        DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
                        DurabilityQosPolicyKind::Persistent => DurabilityKind::Persistent,
                    };
                    self.rtps_writer.lock().unwrap().add_matched_reader(
                        message.discovered_reader_data.reader_proxy().clone(),
                        reliability_kind,
                        durability_kind,
                    );
                    self.matched_subscriptions.add_matched_subscription(
                        instance_handle,
                        message
                            .discovered_reader_data
                            .subscription_builtin_topic_data()
                            .clone(),
                    );
                    self.on_publication_matched(
                        // message.data_writer_address,
                        // // message.publisher,
                        // message.publisher_mask_listener,
                        // message.participant_mask_listener,
                    )?;
                }
            } else if !self.incompatible_subscriptions.contains(&instance_handle) {
                self.incompatible_subscriptions
                    .add_offered_incompatible_qos(instance_handle, incompatible_qos_policy_list);
                self.on_offered_incompatible_qos(
                    // message.data_writer_address,
                    // message.publisher,
                    // message.publisher_mask_listener,
                    // message.participant_mask_listener,
                )?;
            }
        }
        Ok(())
    }
}

pub struct RemoveMatchedReader {
    pub discovered_reader_handle: InstanceHandle,
    // pub data_writer_address: ActorAddress<DataWriterActor>,
    // pub publisher: PublisherAsync,
    // pub publisher_mask_listener: (
    //     Option<MpscSender<PublisherListenerMessage>>,
    //     Vec<StatusKind>,
    // ),
    // pub participant_mask_listener: (
    //     Option<MpscSender<ParticipantListenerMessage>>,
    //     Vec<StatusKind>,
    // ),
}
impl Mail for RemoveMatchedReader {
    type Result = DdsResult<()>;
}
impl MailHandler<RemoveMatchedReader> for DataWriterActor {
    fn handle(&mut self, message: RemoveMatchedReader) -> <RemoveMatchedReader as Mail>::Result {
        if let Some(r) = self
            .matched_subscriptions
            .get_matched_subscription_data(message.discovered_reader_handle)
        {
            let handle = r.key().value.into();
            self.rtps_writer
                .lock()
                .unwrap()
                .delete_matched_reader(handle);
            self.matched_subscriptions
                .remove_matched_subscription(InstanceHandle::new(handle.into()));

            self.on_publication_matched(
                // message.data_writer_address,
                // message.publisher,
                // message.publisher_mask_listener,
                // message.participant_mask_listener,
            )?;
        }

        Ok(())
    }
}

pub struct SetListener {
    pub listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for SetListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetListener> for DataWriterActor {
    fn handle(&mut self, message: SetListener) -> <SetListener as Mail>::Result {
        if let Some(listener) = self.data_writer_listener_thread.take() {
            listener.join()?;
        }

        self.data_writer_listener_thread = message.listener.map(DataWriterListenerThread::new);
        self.status_kind = message.status_kind;
        Ok(())
    }
}

pub struct NewChange {
    pub kind: ChangeKind,
    pub data: Data,
    pub inline_qos: ParameterList,
    pub handle: InstanceHandle,
    pub timestamp: Time,
}
impl Mail for NewChange {
    type Result = RtpsCacheChange;
}
impl MailHandler<NewChange> for DataWriterActor {
    fn handle(&mut self, message: NewChange) -> <NewChange as Mail>::Result {
        self.last_change_sequence_number += 1;
        RtpsCacheChange {
            kind: message.kind,
            writer_guid: self.guid,
            instance_handle: message.handle.into(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(message.timestamp.into()),
            data_value: message.data,
            inline_qos: message.inline_qos,
        }
    }
}

pub struct AddChange {
    pub change: RtpsCacheChange,
    pub now: Time,
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
    pub writer_address: ActorAddress<DataWriterActor>,
    pub publisher_mask_listener: (
        Option<MpscSender<PublisherListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub publisher: PublisherAsync,
    pub executor_handle: ExecutorHandle,
    pub timer_handle: TimerHandle,
}
impl Mail for AddChange {
    type Result = ();
}
impl MailHandler<AddChange> for DataWriterActor {
    fn handle(&mut self, message: AddChange) -> <AddChange as Mail>::Result {
        {
            let mut rtps_writer = self.rtps_writer.lock().unwrap();
            if let HistoryQosPolicyKind::KeepLast(depth) = self.qos.history.kind {
                if rtps_writer
                    .get_history_cache()
                    .get_changes()
                    .iter()
                    .filter(|cc| cc.instance_handle() == message.change.instance_handle())
                    .count()
                    == depth as usize
                {
                    if let Some(smallest_seq_num_instance) = rtps_writer
                        .get_history_cache()
                        .get_changes()
                        .iter()
                        .filter(|cc| cc.instance_handle() == message.change.instance_handle())
                        .map(|cc| cc.sequence_number())
                        .min()
                    {
                        rtps_writer
                            .get_history_cache()
                            .remove_change(smallest_seq_num_instance);
                    }
                }
            }

            let change_instance_handle = message.change.instance_handle();
            let change_timestamp = message.change.source_timestamp();
            let seq_num = message.change.sequence_number();

            if seq_num > self.max_seq_num.unwrap_or(0) {
                self.max_seq_num = Some(seq_num)
            }

            if let Some(t) = self
                .instance_deadline_missed_task
                .remove(&change_instance_handle.into())
            {
                t.abort();
            }

            if let DurationKind::Finite(deadline_missed_period) = self.qos.deadline.period {
                let deadline_missed_interval = std::time::Duration::new(
                    deadline_missed_period.sec() as u64,
                    deadline_missed_period.nanosec(),
                );
                let writer_status_condition = self.status_condition.address();
                let writer_address = message.writer_address.clone();
                let timer_handle = message.timer_handle.clone();
                let writer_listener_mask = self.status_kind.clone();
                let data_writer_listener_sender = self
                    .data_writer_listener_thread
                    .as_ref()
                    .map(|l| l.sender().clone());
                let publisher_listener = message.publisher_mask_listener.0.clone();
                let publisher_listener_mask = message.publisher_mask_listener.1.clone();
                let participant_listener = message.participant_mask_listener.0.clone();
                let participant_listener_mask = message.participant_mask_listener.1.clone();
                let status_condition_address = self.status_condition.address();
                // let topic_address = self.topic_address.clone();
                // let topic_status_condition_address = self.topic_status_condition.clone();
                let type_name = self.type_name.clone();
                let topic_name = self.topic_name.clone();
                let publisher = message.publisher.clone();

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

            if let DurationKind::Finite(lifespan) = self.qos.lifespan.duration {
                if let Some(timestamp) = change_timestamp {
                    let change_lifespan =
                        crate::infrastructure::time::Time::from(timestamp) - message.now + lifespan;
                    if change_lifespan > Duration::new(0, 0) {
                        rtps_writer.get_history_cache().add_change(message.change);
                        message.executor_handle.spawn(async move {
                            message.timer_handle.sleep(change_lifespan.into()).await;

                            message
                                .writer_address
                                .send_actor_mail(RemoveChange { seq_num })
                                .ok();
                        });
                    }
                }
            } else {
                rtps_writer.get_history_cache().add_change(message.change);
            }
        }
    }
}

pub struct RemoveChange {
    pub seq_num: SequenceNumber,
}
impl Mail for RemoveChange {
    type Result = ();
}
impl MailHandler<RemoveChange> for DataWriterActor {
    fn handle(&mut self, message: RemoveChange) -> <RemoveChange as Mail>::Result {
        self.rtps_writer
            .lock()
            .unwrap()
            .get_history_cache()
            .remove_change(message.seq_num);
    }
}

pub struct IsResourcesLimitReached {
    pub instance_handle: InstanceHandle,
}
impl Mail for IsResourcesLimitReached {
    type Result = bool;
}
impl MailHandler<IsResourcesLimitReached> for DataWriterActor {
    fn handle(
        &mut self,
        message: IsResourcesLimitReached,
    ) -> <IsResourcesLimitReached as Mail>::Result {
        let mut rtps_writer = self.rtps_writer.lock().unwrap();
        if let Length::Limited(max_instances) = self.qos.resource_limits.max_instances {
            if !rtps_writer
                .get_history_cache()
                .get_changes()
                .iter()
                .any(|cc| cc.instance_handle() == message.instance_handle.into())
                && rtps_writer.get_history_cache().get_changes().len() == max_instances as usize
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
                    // Only Alive changes count towards the resource limits
                    if rtps_writer
                        .get_history_cache()
                        .get_changes()
                        .iter()
                        .filter(|cc| cc.instance_handle() == message.instance_handle.into())
                        .filter(|cc| cc.kind() == ChangeKind::Alive)
                        .count()
                        >= max_samples_per_instance as usize
                    {
                        return true;
                    }
                }
            }
        }

        if let Length::Limited(max_samples) = self.qos.resource_limits.max_samples {
            let total_samples = rtps_writer
                .get_history_cache()
                .get_changes()
                .iter()
                .filter(|cc| cc.kind() == ChangeKind::Alive)
                .count();

            if total_samples >= max_samples as usize {
                return true;
            }
        }

        false
    }
}

pub struct IsDataLostAfterAddingChange {
    pub instance_handle: InstanceHandle,
}
impl Mail for IsDataLostAfterAddingChange {
    type Result = bool;
}
impl MailHandler<IsDataLostAfterAddingChange> for DataWriterActor {
    fn handle(
        &mut self,
        message: IsDataLostAfterAddingChange,
    ) -> <IsDataLostAfterAddingChange as Mail>::Result {
        let mut rtps_writer = self.rtps_writer.lock().unwrap();
        if let HistoryQosPolicyKind::KeepLast(depth) = self.qos.history.kind {
            if rtps_writer
                .get_history_cache()
                .get_changes()
                .iter()
                .filter(|cc| cc.instance_handle() == message.instance_handle.into())
                .count()
                == depth as usize
            {
                if !rtps_writer.are_all_changes_acknowledged() {
                    return true;
                }
            }
        }
        false
    }
}

pub struct IncrementOfferedDeadlineMissedStatus {
    pub instance_handle: InstanceHandle,
}
impl Mail for IncrementOfferedDeadlineMissedStatus {
    type Result = ();
}
impl MailHandler<IncrementOfferedDeadlineMissedStatus> for DataWriterActor {
    fn handle(
        &mut self,
        message: IncrementOfferedDeadlineMissedStatus,
    ) -> <IncrementOfferedDeadlineMissedStatus as Mail>::Result {
        self.offered_deadline_missed_status.total_count += 1;
        self.offered_deadline_missed_status.total_count_change += 1;
        self.offered_deadline_missed_status.last_instance_handle = message.instance_handle;
    }
}

pub struct GetOfferedDeadlineMissedStatus;
impl Mail for GetOfferedDeadlineMissedStatus {
    type Result = OfferedDeadlineMissedStatus;
}
impl MailHandler<GetOfferedDeadlineMissedStatus> for DataWriterActor {
    fn handle(
        &mut self,
        _: GetOfferedDeadlineMissedStatus,
    ) -> <GetOfferedDeadlineMissedStatus as Mail>::Result {
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
