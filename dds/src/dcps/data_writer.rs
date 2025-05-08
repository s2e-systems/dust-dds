use super::{
    builtin_topics::SubscriptionBuiltinTopicData,
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
    runtime::Clock,
    status_condition::StatusCondition,
    xtypes_glue::key_and_instance_handle::{
        get_instance_handle_from_serialized_foo, get_instance_handle_from_serialized_key,
    },
};
use crate::{
    transport::{
        history_cache::{CacheChange, HistoryCache},
        types::{ChangeKind, Guid},
        writer::{TransportStatefulWriter, TransportStatelessWriter},
    },
    xtypes::dynamic_type::DynamicType,
};
use alloc::{boxed::Box, collections::VecDeque, string::String, sync::Arc, vec::Vec};

pub enum TransportWriterKind {
    Stateful(Box<dyn TransportStatefulWriter>),
    Stateless(Box<dyn TransportStatelessWriter>),
}

impl TransportWriterKind {
    pub fn guid(&self) -> Guid {
        match self {
            TransportWriterKind::Stateful(w) => w.guid(),
            TransportWriterKind::Stateless(w) => w.guid(),
        }
    }

    pub fn history_cache(&mut self) -> &mut dyn HistoryCache {
        match self {
            TransportWriterKind::Stateful(w) => w.history_cache(),
            TransportWriterKind::Stateless(w) => w.history_cache(),
        }
    }
}

pub struct InstancePublicationTime {
    instance: InstanceHandle,
    last_write_time: Time,
}

pub struct InstanceSamples {
    instance: InstanceHandle,
    samples: VecDeque<i64>,
}

pub struct DataWriterEntity<S, L> {
    instance_handle: InstanceHandle,
    transport_writer: TransportWriterKind,
    topic_name: String,
    type_name: String,
    type_support: Arc<dyn DynamicType + Send + Sync>,
    matched_subscription_list: Vec<SubscriptionBuiltinTopicData>,
    publication_matched_status: PublicationMatchedStatus,
    incompatible_subscription_list: Vec<InstanceHandle>,
    offered_incompatible_qos_status: OfferedIncompatibleQosStatus,
    enabled: bool,
    status_condition: S,
    listener_sender: L,
    listener_mask: Vec<StatusKind>,
    max_seq_num: Option<i64>,
    last_change_sequence_number: i64,
    qos: DataWriterQos,
    registered_instance_list: Vec<InstanceHandle>,
    offered_deadline_missed_status: OfferedDeadlineMissedStatus,
    instance_publication_time: Vec<InstancePublicationTime>,
    instance_samples: Vec<InstanceSamples>,
}

impl<S, L> DataWriterEntity<S, L> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance_handle: InstanceHandle,
        transport_writer: TransportWriterKind,
        topic_name: String,
        type_name: String,
        type_support: Arc<dyn DynamicType + Send + Sync>,
        status_condition: S,
        listener_sender: L,
        listener_mask: Vec<StatusKind>,
        qos: DataWriterQos,
    ) -> Self {
        Self {
            instance_handle,
            transport_writer,
            topic_name,
            type_name,
            type_support,
            matched_subscription_list: Vec::new(),
            publication_matched_status: PublicationMatchedStatus::default(),
            incompatible_subscription_list: Vec::new(),
            offered_incompatible_qos_status: OfferedIncompatibleQosStatus::default(),
            enabled: false,
            status_condition,
            listener_sender,
            listener_mask,
            max_seq_num: None,
            last_change_sequence_number: 0,
            qos,
            registered_instance_list: Vec::new(),
            offered_deadline_missed_status: OfferedDeadlineMissedStatus::default(),
            instance_publication_time: Vec::new(),
            instance_samples: Vec::new(),
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

    pub fn transport_writer(&self) -> &TransportWriterKind {
        &self.transport_writer
    }

    pub fn transport_writer_mut(&mut self) -> &mut TransportWriterKind {
        &mut self.transport_writer
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
        clock: impl Clock,
    ) -> DdsResult<i64> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.last_change_sequence_number += 1;

        let instance_handle =
            get_instance_handle_from_serialized_foo(&serialized_data, self.type_support.as_ref())?;

        if !self.registered_instance_list.contains(&instance_handle) {
            if self.registered_instance_list.len() < self.qos.resource_limits.max_instances {
                self.registered_instance_list.push(instance_handle);
            } else {
                return Err(DdsError::OutOfResources);
            }
        }

        if let Length::Limited(max_instances) = self.qos.resource_limits.max_instances {
            if !self
                .instance_samples
                .iter()
                .any(|x| x.instance == instance_handle)
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
                    if let Some(s) = self
                        .instance_samples
                        .iter()
                        .find(|x| x.instance == instance_handle)
                    {
                        // Only Alive changes count towards the resource limits
                        if s.samples.len() >= max_samples_per_instance as usize {
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
                .fold(0, |acc, x| acc + x.samples.len());

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
            if let Some(s) = self
                .instance_samples
                .iter_mut()
                .find(|x| x.instance == instance_handle)
            {
                if s.samples.len() == depth as usize {
                    if let Some(&smallest_seq_num_instance) = s.samples.front() {
                        if self.qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
                            let start_time = clock.now();
                            while let TransportWriterKind::Stateful(w) = &self.transport_writer {
                                if w.is_change_acknowledged(smallest_seq_num_instance) {
                                    break;
                                }

                                if let DurationKind::Finite(t) =
                                    self.qos.reliability.max_blocking_time
                                {
                                    if (clock.now() - start_time) > t.into() {
                                        return Err(DdsError::Timeout);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(smallest_seq_num_instance) = s.samples.pop_front() {
                        self.transport_writer
                            .history_cache()
                            .remove_change(smallest_seq_num_instance);
                    }
                }
            }
        }

        let seq_num = change.sequence_number();

        if seq_num > self.max_seq_num.unwrap_or(0) {
            self.max_seq_num = Some(seq_num)
        }

        match self
            .instance_publication_time
            .iter_mut()
            .find(|x| x.instance == instance_handle)
        {
            Some(x) => {
                if x.last_write_time < timestamp {
                    x.last_write_time = timestamp;
                }
            }
            None => self
                .instance_publication_time
                .push(InstancePublicationTime {
                    instance: instance_handle,
                    last_write_time: timestamp,
                }),
        }

        match self
            .instance_samples
            .iter_mut()
            .find(|x| x.instance == instance_handle)
        {
            Some(s) => s.samples.push_back(change.sequence_number),
            None => {
                let s = InstanceSamples {
                    instance: instance_handle,
                    samples: VecDeque::from([change.sequence_number]),
                };
                self.instance_samples.push(s);
            }
        }
        self.transport_writer.history_cache().add_change(change);
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

        if let Some(i) = self
            .instance_publication_time
            .iter()
            .position(|x| x.instance == instance_handle)
        {
            self.instance_publication_time.remove(i);
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
        self.transport_writer
            .history_cache()
            .add_change(cache_change);

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

        if let Some(i) = self
            .instance_publication_time
            .iter()
            .position(|x| x.instance == instance_handle)
        {
            self.instance_publication_time.remove(i);
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
        self.transport_writer
            .history_cache()
            .add_change(cache_change);
        Ok(())
    }

    pub fn remove_change(&mut self, sequence_number: i64) {
        self.transport_writer
            .history_cache()
            .remove_change(sequence_number);
    }

    pub fn add_matched_subscription(
        &mut self,
        subscription_builtin_topic_data: SubscriptionBuiltinTopicData,
    ) {
        match self
            .matched_subscription_list
            .iter_mut()
            .find(|x| x.key() == subscription_builtin_topic_data.key())
        {
            Some(x) => *x = subscription_builtin_topic_data,
            None => self
                .matched_subscription_list
                .push(subscription_builtin_topic_data),
        };
        self.publication_matched_status.current_count = self.matched_subscription_list.len() as i32;
        self.publication_matched_status.current_count_change += 1;
        self.publication_matched_status.total_count += 1;
        self.publication_matched_status.total_count_change += 1;
    }

    pub fn remove_matched_subscription(&mut self, subscription_handle: &InstanceHandle) {
        let Some(i) = self
            .matched_subscription_list
            .iter()
            .position(|x| &x.key().value == subscription_handle.as_ref())
        else {
            return;
        };
        self.matched_subscription_list.remove(i);
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

    pub fn get_offered_incompatible_qos_status(&mut self) -> OfferedIncompatibleQosStatus {
        let status = self.offered_incompatible_qos_status.clone();
        self.offered_incompatible_qos_status.total_count_change = 0;
        status
    }

    pub fn get_matched_subscriptions(&self) -> Vec<InstanceHandle> {
        self.matched_subscription_list
            .iter()
            .map(|x| InstanceHandle::new(x.key().value))
            .collect()
    }

    pub fn get_matched_subscription_data(
        &self,
        subscription_handle: &InstanceHandle,
    ) -> Option<&SubscriptionBuiltinTopicData> {
        self.matched_subscription_list
            .iter()
            .find(|x| subscription_handle.as_ref() == &x.key().value)
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

    pub fn status_condition(&self) -> &S {
        &self.status_condition
    }

    pub fn get_instance_write_time(&self, instance_handle: InstanceHandle) -> Option<Time> {
        self.instance_publication_time
            .iter()
            .find(|x| x.instance == instance_handle)
            .map(|x| x.last_write_time)
    }

    pub fn set_listener(&mut self, listener_sender: L, listener_mask: Vec<StatusKind>) {
        self.listener_sender = listener_sender;
        self.listener_mask = listener_mask;
    }

    pub fn listener(&self) -> &L {
        &self.listener_sender
    }

    pub fn listener_mask(&self) -> &[StatusKind] {
        &self.listener_mask
    }

    pub fn are_all_changes_acknowledged(&self) -> bool {
        match &self.transport_writer {
            TransportWriterKind::Stateful(w) => {
                w.is_change_acknowledged(self.last_change_sequence_number)
            }
            TransportWriterKind::Stateless(_) => true,
        }
    }
}

impl<S, L> DataWriterEntity<S, L>
where
    S: StatusCondition,
{
    pub fn get_offered_deadline_missed_status(&mut self) -> OfferedDeadlineMissedStatus {
        let status = self.offered_deadline_missed_status.clone();
        self.offered_deadline_missed_status.total_count_change = 0;
        self.status_condition
            .remove_state(StatusKind::OfferedDeadlineMissed);

        status
    }
}
