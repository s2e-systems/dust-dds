use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        listeners::data_writer_listener::DataWriterListenerActor,
        status_condition::status_condition_actor::{self, StatusConditionActor},
        xtypes_glue::key_and_instance_handle::{
            get_instance_handle_from_serialized_foo, get_instance_handle_from_serialized_key,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataWriterQos,
        qos_policy::{HistoryQosPolicyKind, Length, QosPolicyId, ReliabilityQosPolicyKind},
        status::{
            OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
            QosPolicyCount, StatusKind,
        },
        time::{DurationKind, Time},
    },
    runtime::{actor::Actor, executor::TaskHandle},
    transport::{cache_change::CacheChange, types::ChangeKind, writer::WriterHistoryCache},
    xtypes::dynamic_type::DynamicType,
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
    listener: Option<Actor<DataWriterListenerActor>>,
    listener_mask: Vec<StatusKind>,
    max_seq_num: Option<i64>,
    last_change_sequence_number: i64,
    qos: DataWriterQos,
    registered_instance_list: HashSet<InstanceHandle>,
    offered_deadline_missed_status: OfferedDeadlineMissedStatus,
    instance_deadline_missed_task: HashMap<InstanceHandle, TaskHandle>,
    instance_samples: HashMap<InstanceHandle, VecDeque<i64>>,
}

impl DataWriterEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance_handle: InstanceHandle,
        transport_writer: Box<dyn WriterHistoryCache>,
        topic_name: String,
        type_name: String,
        type_support: Arc<dyn DynamicType + Send + Sync>,
        status_condition: Actor<StatusConditionActor>,
        listener: Option<Actor<DataWriterListenerActor>>,
        listener_mask: Vec<StatusKind>,
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
            listener,
            listener_mask,
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
        self.registered_instance_list.contains(instance_handle)
    }

    pub fn write_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        timestamp: Time,
    ) -> DdsResult<i64> {
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

        let change = CacheChange {
            kind: ChangeKind::Alive,
            writer_guid: self.transport_writer().guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_data.into(),
        };
        if let HistoryQosPolicyKind::KeepLast(depth) = self.qos.history.kind {
            if let Some(s) = self.instance_samples.get_mut(&instance_handle) {
                if s.len() == depth as usize {
                    if let Some(&smallest_seq_num_instance) = s.front() {
                        if self.qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
                            let start_time = std::time::Instant::now();
                            loop {
                                if self
                                    .transport_writer
                                    .is_change_acknowledged(smallest_seq_num_instance)
                                {
                                    break;
                                }

                                if let DurationKind::Finite(t) =
                                    self.qos.reliability.max_blocking_time
                                {
                                    if start_time.elapsed() > t.into() {
                                        return Err(DdsError::Timeout);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(smallest_seq_num_instance) = s.pop_front() {
                        self.transport_writer
                            .remove_change(smallest_seq_num_instance);
                    }
                }
            }
        }

        let seq_num = change.sequence_number();

        if seq_num > self.max_seq_num.unwrap_or(0) {
            self.max_seq_num = Some(seq_num)
        }

        if let Some(t) = self.instance_deadline_missed_task.remove(&instance_handle) {
            t.abort();
        }

        if let Some(t) = self.instance_deadline_missed_task.remove(&instance_handle) {
            t.abort();
        }

        self.instance_samples
            .entry(instance_handle)
            .or_default()
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

        let cache_change = CacheChange {
            kind: ChangeKind::NotAliveDisposed,
            writer_guid: self.transport_writer().guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_key.into(),
        };
        self.transport_writer.add_change(cache_change);

        Ok(())
    }

    pub fn unregister_w_timestamp(
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

        let cache_change = CacheChange {
            kind: ChangeKind::NotAliveDisposed,
            writer_guid: self.transport_writer().guid(),
            sequence_number: self.last_change_sequence_number,
            source_timestamp: Some(timestamp.into()),
            instance_handle: Some(instance_handle.into()),
            data_value: serialized_key.into(),
        };
        self.transport_writer.add_change(cache_change);
        Ok(())
    }

    pub fn remove_change(&mut self, sequence_number: i64) {
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
    }

    pub fn remove_matched_subscription(&mut self, subscription_handle: &InstanceHandle) {
        self.matched_subscription_list.remove(subscription_handle);
        self.publication_matched_status.current_count = self.matched_subscription_list.len() as i32;
        self.publication_matched_status.current_count_change -= 1;
    }

    pub fn add_incompatible_subscription(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        if !self.incompatible_subscription_list.contains(&handle) {
            self.offered_incompatible_qos_status.total_count += 1;
            self.offered_incompatible_qos_status.total_count_change += 1;
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
        }
    }

    pub fn get_offered_incompatible_qos_status(&mut self) -> OfferedIncompatibleQosStatus {
        let status = self.offered_incompatible_qos_status.clone();
        self.offered_incompatible_qos_status.total_count_change = 0;
        status
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

        status
    }

    pub fn increment_offered_deadline_missed_status(&mut self, instance_handle: InstanceHandle) {
        self.offered_deadline_missed_status.last_instance_handle = instance_handle;
        self.offered_deadline_missed_status.total_count += 1;
        self.offered_deadline_missed_status.total_count_change += 1;
    }

    pub fn type_support(&self) -> &(dyn DynamicType + Send + Sync) {
        self.type_support.as_ref()
    }

    pub fn insert_instance_deadline_missed_task(
        &mut self,
        instance_handle: InstanceHandle,
        task: TaskHandle,
    ) {
        self.instance_deadline_missed_task
            .insert(instance_handle, task);
    }

    pub fn status_condition(&self) -> &Actor<StatusConditionActor> {
        &self.status_condition
    }

    pub fn set_listener(
        &mut self,
        listener: Option<Actor<DataWriterListenerActor>>,
        listener_mask: Vec<StatusKind>,
    ) {
        self.listener = listener;
        self.listener_mask = listener_mask;
    }

    pub fn listener(&self) -> Option<&Actor<DataWriterListenerActor>> {
        self.listener.as_ref()
    }

    pub fn listener_mask(&self) -> &[StatusKind] {
        &self.listener_mask
    }

    pub fn are_all_changes_acknowledged(&self) -> bool {
        self.transport_writer
            .is_change_acknowledged(self.last_change_sequence_number)
    }
}
