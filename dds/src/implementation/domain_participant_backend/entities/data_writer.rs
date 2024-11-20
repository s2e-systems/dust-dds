use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        data_representation_inline_qos::{
            parameter_id_values::{PID_KEY_HASH, PID_STATUS_INFO},
            types::{
                STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED, STATUS_INFO_UNREGISTERED,
            },
        },
        listeners::data_writer_listener::DataWriterListenerThread,
        status_condition::status_condition_actor::{self, StatusConditionActor},
        xtypes_glue::key_and_instance_handle::{
            get_instance_handle_from_serialized_foo, get_instance_handle_from_serialized_key,
            get_serialized_key_from_serialized_foo,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataWriterQos,
        qos_policy::{HistoryQosPolicyKind, Length, QosPolicyId},
        status::{
            OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
            QosPolicyCount, StatusKind,
        },
        time::{DurationKind, Time},
    },
    rtps::{
        cache_change::RtpsCacheChange,
        messages::submessage_elements::{Parameter, ParameterList},
        stateful_writer::WriterHistoryCache,
        types::{ChangeKind, SequenceNumber},
    },
    runtime::{actor::Actor, executor::TaskHandle},
    xtypes::{
        dynamic_type::DynamicType, serialize::XTypesSerialize, xcdr_serializer::Xcdr1LeSerializer,
    },
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

pub struct DataWriterEntity {
    instance_handle: InstanceHandle,
    transport_writer: Box<dyn WriterHistoryCache>,
    topic_name: String,
    type_name: String,
    type_support: Arc<dyn DynamicType + Send + Sync>,
    matched_subscription_list: HashMap<InstanceHandle, SubscriptionBuiltinTopicData>,
    publication_matched_status: PublicationMatchedStatus,
    incompatible_subscription_list: HashSet<InstanceHandle>,
    offered_incompatible_qos_status: OfferedIncompatibleQosStatus,
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
    instance_samples: HashMap<InstanceHandle, VecDeque<SequenceNumber>>,
}

impl DataWriterEntity {
    pub fn new(
        instance_handle: InstanceHandle,
        transport_writer: Box<dyn WriterHistoryCache>,
        topic_name: String,
        type_name: String,
        type_support: Arc<dyn DynamicType + Send + Sync>,
        status_condition: Actor<StatusConditionActor>,
        data_writer_listener_thread: Option<DataWriterListenerThread>,
        status_kind: Vec<StatusKind>,
        qos: DataWriterQos,
    ) -> Self {
        Self {
            instance_handle,
            transport_writer,
            topic_name,
            type_name,
            type_support,
            matched_subscription_list: HashMap::new(),
            publication_matched_status: PublicationMatchedStatus::default(),
            incompatible_subscription_list: HashSet::new(),
            offered_incompatible_qos_status: OfferedIncompatibleQosStatus::default(),
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

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    pub fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }

    pub fn transport_writer(&self) -> &dyn WriterHistoryCache {
        self.transport_writer.as_ref()
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn qos(&self) -> &DataWriterQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: DataWriterQos) -> DdsResult<()> {
        qos.is_consistent()?;
        if self.enabled {
            self.qos.check_immutability(&qos)?;
        }
        self.qos = qos;
        Ok(())
    }

    pub fn contains_instance(&mut self, instance_handle: &InstanceHandle) -> bool {
        self.registered_instance_list.contains(&instance_handle)
    }

    pub fn write_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<SequenceNumber> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.last_change_sequence_number += 1;

        let instance_handle =
            get_instance_handle_from_serialized_foo(&serialized_data, self.type_support.as_ref())?;

        if !self.registered_instance_list.contains(&instance_handle) {
            if self.registered_instance_list.len() < self.qos.resource_limits.max_instances {
                self.registered_instance_list.insert(instance_handle);
            } else {
                return Err(DdsError::OutOfResources);
            }
        }

        if let Length::Limited(max_instances) = self.qos.resource_limits.max_instances {
            if !self.instance_samples.contains_key(&instance_handle)
                && self.instance_samples.len() == max_instances as usize
            {
                return Err(DdsError::OutOfResources);
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
                            return Err(DdsError::OutOfResources);
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
                return Err(DdsError::OutOfResources);
            }
        }

        let pid_key_hash = Parameter::new(PID_KEY_HASH, Arc::from(*instance_handle.as_ref()));
        let parameter_list = ParameterList::new(vec![pid_key_hash]);

        let change = RtpsCacheChange {
            kind: ChangeKind::Alive,
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            data_value: serialized_data.into(),
            inline_qos: parameter_list,
        };
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

        self.instance_samples
            .entry(instance_handle)
            .or_insert(VecDeque::new())
            .push_back(change.sequence_number);
        self.transport_writer.add_change(change);
        Ok(self.last_change_sequence_number)
    }

    pub fn dispose_w_timestamp(
        &mut self,
        serialized_key: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let has_key = {
            let mut has_key = false;
            for index in 0..self.type_support.get_member_count() {
                if self
                    .type_support
                    .get_member_by_index(index)?
                    .get_descriptor()?
                    .is_key
                {
                    has_key = true;
                    break;
                }
            }
            has_key
        };
        if !has_key {
            return Err(DdsError::IllegalOperation);
        }

        let instance_handle =
            get_instance_handle_from_serialized_key(&serialized_key, self.type_support.as_ref())?;
        if !self.registered_instance_list.contains(&instance_handle) {
            return Err(DdsError::BadParameter);
        }

        if let Some(t) = self.instance_deadline_missed_task.remove(&instance_handle) {
            t.abort();
        }

        self.last_change_sequence_number += 1;

        let mut serialized_status_info = Vec::new();
        let mut serializer = Xcdr1LeSerializer::new(&mut serialized_status_info);
        XTypesSerialize::serialize(&STATUS_INFO_DISPOSED, &mut serializer)?;
        let pid_status_info = Parameter::new(PID_STATUS_INFO, Arc::from(serialized_status_info));

        let pid_key_hash = Parameter::new(PID_KEY_HASH, Arc::from(*instance_handle.as_ref()));
        let parameter_list = ParameterList::new(vec![pid_status_info, pid_key_hash]);

        let cache_change = RtpsCacheChange {
            kind: ChangeKind::NotAliveDisposed,
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            data_value: serialized_key.into(),
            inline_qos: parameter_list,
        };
        self.transport_writer.add_change(cache_change);

        Ok(())
    }

    pub fn unregister_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let has_key = {
            let mut has_key = false;
            for index in 0..self.type_support.get_member_count() {
                if self
                    .type_support
                    .get_member_by_index(index)?
                    .get_descriptor()?
                    .is_key
                {
                    has_key = true;
                    break;
                }
            }
            has_key
        };
        if !has_key {
            return Err(DdsError::IllegalOperation);
        }

        let serialized_key =
            get_serialized_key_from_serialized_foo(&serialized_data, self.type_support.as_ref())?;

        let instance_handle =
            get_instance_handle_from_serialized_key(&serialized_key, self.type_support.as_ref())?;
        if !self.registered_instance_list.contains(&instance_handle) {
            return Err(DdsError::BadParameter);
        }

        if let Some(t) = self.instance_deadline_missed_task.remove(&instance_handle) {
            t.abort();
        }

        self.last_change_sequence_number += 1;

        let mut serialized_status_info = Vec::new();
        let mut serializer = Xcdr1LeSerializer::new(&mut serialized_status_info);
        match self
            .qos
            .writer_data_lifecycle
            .autodispose_unregistered_instances
        {
            true => {
                XTypesSerialize::serialize(&STATUS_INFO_DISPOSED_UNREGISTERED, &mut serializer)?
            }
            false => XTypesSerialize::serialize(&STATUS_INFO_UNREGISTERED, &mut serializer)?,
        }
        let pid_status_info = Parameter::new(PID_STATUS_INFO, Arc::from(serialized_status_info));

        let pid_key_hash = Parameter::new(PID_KEY_HASH, Arc::from(*instance_handle.as_ref()));
        let parameter_list = ParameterList::new(vec![pid_status_info, pid_key_hash]);

        let cache_change = RtpsCacheChange {
            kind: ChangeKind::NotAliveDisposed,
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            data_value: serialized_key.into(),
            inline_qos: parameter_list,
        };
        self.transport_writer.add_change(cache_change);
        Ok(())
    }

    pub fn remove_change(&mut self, sequence_number: SequenceNumber) {
        self.transport_writer.remove_change(sequence_number);
    }

    pub fn add_matched_subscription(
        &mut self,
        subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
    ) {
        let handle = InstanceHandle::new(subscription_builtin_topic_data.key().value);
        self.matched_subscription_list
            .insert(handle, subscription_builtin_topic_data);
        self.publication_matched_status.current_count = self.matched_subscription_list.len() as i32;
        self.publication_matched_status.current_count_change += 1;
        self.publication_matched_status.total_count += 1;
        self.publication_matched_status.total_count_change += 1;
        self.status_condition
            .send_actor_mail(status_condition_actor::AddCommunicationState {
                state: StatusKind::PublicationMatched,
            });
    }

    pub fn remove_matched_subscription(&mut self, subscription_handle: &InstanceHandle) {
        self.matched_subscription_list.remove(subscription_handle);
        self.publication_matched_status.current_count = self.matched_subscription_list.len() as i32;
        self.publication_matched_status.current_count_change -= 1;
        self.publication_matched_status.total_count -= 1;
        self.publication_matched_status.total_count_change -= 1;
        self.status_condition
            .send_actor_mail(status_condition_actor::AddCommunicationState {
                state: StatusKind::PublicationMatched,
            });
    }

    pub fn add_incompatible_subscription(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        if !self.incompatible_subscription_list.contains(&handle) {
            self.offered_incompatible_qos_status.total_count += 1;
            self.offered_incompatible_qos_status.last_policy_id = incompatible_qos_policy_list[0];

            self.incompatible_subscription_list.insert(handle);
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
            self.status_condition
                .send_actor_mail(status_condition_actor::AddCommunicationState {
                    state: StatusKind::OfferedIncompatibleQos,
                });
        }
    }

    pub fn get_matched_subscriptions(&self) -> Vec<InstanceHandle> {
        self.matched_subscription_list.keys().cloned().collect()
    }

    pub fn get_matched_subscription_data(
        &self,
        subscription_handle: &InstanceHandle,
    ) -> Option<&SubscriptionBuiltinTopicData> {
        self.matched_subscription_list.get(subscription_handle)
    }

    pub fn get_offered_deadline_missed_status(&mut self) -> OfferedDeadlineMissedStatus {
        let status = self.offered_deadline_missed_status.clone();
        self.offered_deadline_missed_status.total_count_change = 0;
        self.status_condition
            .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                state: StatusKind::OfferedDeadlineMissed,
            });
        status
    }

    pub fn get_publication_matched_status(&mut self) -> PublicationMatchedStatus {
        let status = self.publication_matched_status.clone();
        self.publication_matched_status.current_count_change = 0;
        self.publication_matched_status.total_count_change = 0;
        self.status_condition
            .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                state: StatusKind::PublicationMatched,
            });
        status
    }
}
