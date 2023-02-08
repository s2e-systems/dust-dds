use std::collections::HashMap;

use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_writer_data::DiscoveredWriterData,
        },
        rtps::{
            messages::{
                overall_structure::RtpsMessageHeader,
                submessages::{
                    DataFragSubmessage, DataSubmessage, HeartbeatFragSubmessage,
                    HeartbeatSubmessage,
                },
            },
            stateful_reader::{RtpsStatefulReader, StatefulReaderDataReceivedResult},
            transport::TransportWrite,
            types::{GuidPrefix, Locator, GUID_UNKNOWN},
            writer_proxy::RtpsWriterProxy,
        },
        utils::{
            condvar::DdsCondvar,
            shared_object::{DdsRwLock, DdsShared, DdsWeak},
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind},
        qos_policy::{
            QosPolicyId, DEADLINE_QOS_POLICY_ID, DESTINATIONORDER_QOS_POLICY_ID,
            DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID,
            PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID,
        },
        status::{
            LivelinessChangedStatus, QosPolicyCount, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
            SampleRejectedStatusKind, StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, Time},
    },
    subscription::{
        data_reader::{AnyDataReader, Sample},
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    any_data_reader_listener::AnyDataReaderListener, message_receiver::MessageReceiver,
    status_condition_impl::StatusConditionImpl, topic_impl::TopicImpl,
    user_defined_subscriber::UserDefinedSubscriber,
};

pub enum UserDefinedReaderDataSubmessageReceivedResult {
    NoChange,
    NewDataAvailable,
}

impl SampleLostStatus {
    fn increment(&mut self) {
        self.total_count += 1;
        self.total_count_change += 1;
    }

    fn read_and_reset(&mut self) -> Self {
        let status = self.clone();
        self.total_count_change = 0;
        status
    }
}

impl SampleRejectedStatus {
    fn increment(
        &mut self,
        instance_handle: InstanceHandle,
        rejected_reason: SampleRejectedStatusKind,
    ) {
        self.total_count += 1;
        self.total_count_change += 1;
        self.last_instance_handle = instance_handle;
        self.last_reason = rejected_reason;
    }

    fn read_and_reset(&mut self) -> Self {
        let status = self.clone();

        self.total_count_change = 0;

        status
    }
}

impl RequestedDeadlineMissedStatus {
    fn increment(&mut self, instance_handle: InstanceHandle) {
        self.total_count += 1;
        self.total_count_change += 1;
        self.last_instance_handle = instance_handle;
    }

    fn read_and_reset(&mut self) -> Self {
        let status = self.clone();

        self.total_count_change = 0;

        status
    }
}

impl LivelinessChangedStatus {
    fn read_and_reset(&mut self) -> Self {
        let status = self.clone();

        self.alive_count_change = 0;
        self.not_alive_count_change = 0;

        status
    }
}

impl RequestedIncompatibleQosStatus {
    fn increment(&mut self, incompatible_qos_policy_list: Vec<QosPolicyId>) {
        self.total_count += 1;
        self.total_count_change += 1;
        self.last_policy_id = incompatible_qos_policy_list[0];
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

    fn read_and_reset(&mut self) -> Self {
        let status = self.clone();
        self.total_count_change = 0;
        status
    }
}

impl SubscriptionMatchedStatus {
    fn increment(&mut self, instance_handle: InstanceHandle) {
        self.total_count += 1;
        self.total_count_change += 1;
        self.last_publication_handle = instance_handle;
        self.current_count += 1;
        self.current_count_change += 1;
    }

    fn read_and_reset(&mut self, current_count: i32) -> Self {
        let last_current_count = self.current_count;
        self.current_count = current_count;
        self.current_count_change = current_count - last_current_count;
        let status = self.clone();

        self.total_count_change = 0;

        status
    }
}

pub struct UserDefinedDataReader {
    rtps_reader: DdsRwLock<RtpsStatefulReader>,
    topic: DdsShared<TopicImpl>,
    listener: DdsRwLock<Option<Box<dyn AnyDataReaderListener + Send + Sync>>>,
    listener_status_mask: DdsRwLock<Vec<StatusKind>>,
    parent_subscriber: DdsWeak<UserDefinedSubscriber>,
    liveliness_changed_status: DdsRwLock<LivelinessChangedStatus>,
    requested_deadline_missed_status: DdsRwLock<RequestedDeadlineMissedStatus>,
    requested_incompatible_qos_status: DdsRwLock<RequestedIncompatibleQosStatus>,
    sample_lost_status: DdsRwLock<SampleLostStatus>,
    sample_rejected_status: DdsRwLock<SampleRejectedStatus>,
    subscription_matched_status: DdsRwLock<SubscriptionMatchedStatus>,
    matched_publication_list: DdsRwLock<HashMap<InstanceHandle, PublicationBuiltinTopicData>>,
    enabled: DdsRwLock<bool>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    user_defined_data_send_condvar: DdsCondvar,
    instance_reception_time: DdsRwLock<HashMap<InstanceHandle, Time>>,
    data_available_status_changed_flag: DdsRwLock<bool>,
}

impl UserDefinedDataReader {
    pub fn new(
        rtps_reader: RtpsStatefulReader,
        topic: DdsShared<TopicImpl>,
        listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        mask: &[StatusKind],
        parent_subscriber: DdsWeak<UserDefinedSubscriber>,
        user_defined_data_send_condvar: DdsCondvar,
    ) -> DdsShared<Self> {
        DdsShared::new(UserDefinedDataReader {
            rtps_reader: DdsRwLock::new(rtps_reader),
            topic,
            listener: DdsRwLock::new(listener),
            listener_status_mask: DdsRwLock::new(mask.to_vec()),
            parent_subscriber,
            liveliness_changed_status: DdsRwLock::new(LivelinessChangedStatus::default()),
            requested_deadline_missed_status: DdsRwLock::new(
                RequestedDeadlineMissedStatus::default(),
            ),
            requested_incompatible_qos_status: DdsRwLock::new(
                RequestedIncompatibleQosStatus::default(),
            ),
            sample_lost_status: DdsRwLock::new(SampleLostStatus::default()),
            sample_rejected_status: DdsRwLock::new(SampleRejectedStatus::default()),
            subscription_matched_status: DdsRwLock::new(SubscriptionMatchedStatus::default()),
            matched_publication_list: DdsRwLock::new(HashMap::new()),
            enabled: DdsRwLock::new(false),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            user_defined_data_send_condvar,
            instance_reception_time: DdsRwLock::new(HashMap::new()),
            data_available_status_changed_flag: DdsRwLock::new(false),
        })
    }
}

impl DdsShared<UserDefinedDataReader> {
    pub fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) -> UserDefinedReaderDataSubmessageReceivedResult {
        let data_submessage_received_result = self
            .rtps_reader
            .write_lock()
            .on_data_submessage_received(data_submessage, message_receiver);

        match data_submessage_received_result {
            StatefulReaderDataReceivedResult::NoMatchedWriterProxy => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::NewSampleAdded(instance_handle) => {
                self.instance_reception_time
                    .write_lock()
                    .insert(instance_handle, message_receiver.reception_timestamp());
                *self.data_available_status_changed_flag.write_lock() = true;
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::NewSampleAddedAndSamplesLost(instance_handle) => {
                self.instance_reception_time
                    .write_lock()
                    .insert(instance_handle, message_receiver.reception_timestamp());
                *self.data_available_status_changed_flag.write_lock() = true;
                self.on_sample_lost();
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::SampleRejected(instance_handle, rejected_reason) => {
                self.on_sample_rejected(instance_handle, rejected_reason);
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::InvalidData(_) => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
        }
    }

    pub fn on_data_frag_submessage_received(
        &self,
        data_frag_submessage: &DataFragSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) -> UserDefinedReaderDataSubmessageReceivedResult {
        let data_submessage_received_result = self
            .rtps_reader
            .write_lock()
            .on_data_frag_submessage_received(data_frag_submessage, message_receiver);

        match data_submessage_received_result {
            StatefulReaderDataReceivedResult::NoMatchedWriterProxy => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::NewSampleAdded(instance_handle) => {
                self.instance_reception_time
                    .write_lock()
                    .insert(instance_handle, message_receiver.reception_timestamp());
                *self.data_available_status_changed_flag.write_lock() = true;
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::NewSampleAddedAndSamplesLost(instance_handle) => {
                self.instance_reception_time
                    .write_lock()
                    .insert(instance_handle, message_receiver.reception_timestamp());
                *self.data_available_status_changed_flag.write_lock() = true;
                self.on_sample_lost();
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::SampleRejected(instance_handle, rejected_reason) => {
                self.on_sample_rejected(instance_handle, rejected_reason);
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::InvalidData(_) => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
        }
    }

    pub fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        self.rtps_reader
            .write_lock()
            .on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
        self.user_defined_data_send_condvar.notify_all();
    }

    pub fn on_heartbeat_frag_submessage_received(
        &self,
        heartbeat_frag_submessage: &HeartbeatFragSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        self.rtps_reader
            .write_lock()
            .on_heartbeat_frag_submessage_received(heartbeat_frag_submessage, source_guid_prefix);
    }

    pub fn add_matched_writer(
        &self,
        discovered_writer_data: &DiscoveredWriterData,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
    ) {
        let publication_builtin_topic_data = &discovered_writer_data.publication_builtin_topic_data;
        if publication_builtin_topic_data.topic_name == self.topic.get_name()
            && publication_builtin_topic_data.type_name == self.topic.get_type_name()
        {
            let incompatible_qos_policy_list =
                self.get_discovered_writer_incompatible_qos_policy_list(discovered_writer_data);
            if incompatible_qos_policy_list.is_empty() {
                let unicast_locator_list = if discovered_writer_data
                    .writer_proxy
                    .unicast_locator_list
                    .is_empty()
                {
                    default_unicast_locator_list
                } else {
                    discovered_writer_data
                        .writer_proxy
                        .unicast_locator_list
                        .as_ref()
                };

                let multicast_locator_list = if discovered_writer_data
                    .writer_proxy
                    .multicast_locator_list
                    .is_empty()
                {
                    default_multicast_locator_list
                } else {
                    discovered_writer_data
                        .writer_proxy
                        .multicast_locator_list
                        .as_ref()
                };

                let writer_proxy = RtpsWriterProxy::new(
                    discovered_writer_data.writer_proxy.remote_writer_guid,
                    unicast_locator_list,
                    multicast_locator_list,
                    discovered_writer_data.writer_proxy.data_max_size_serialized,
                    discovered_writer_data.writer_proxy.remote_group_entity_id,
                );

                self.rtps_reader
                    .write_lock()
                    .matched_writer_add(writer_proxy);
                let instance_handle = discovered_writer_data.get_serialized_key().into();
                self.matched_publication_list
                    .write_lock()
                    .insert(instance_handle, publication_builtin_topic_data.clone());

                self.on_subscription_matched(instance_handle);
            } else {
                self.on_requested_incompatible_qos(incompatible_qos_policy_list);
            }
        }
    }

    fn get_discovered_writer_incompatible_qos_policy_list(
        &self,
        discovered_writer_data: &DiscoveredWriterData,
    ) -> Vec<QosPolicyId> {
        let writer_info = &discovered_writer_data.publication_builtin_topic_data;
        let reader_qos = self.rtps_reader.read_lock().reader().get_qos().clone();
        let parent_subscriber_qos = self.get_subscriber().get_qos();

        let mut incompatible_qos_policy_list = Vec::new();

        if parent_subscriber_qos.presentation.access_scope > writer_info.presentation.access_scope
            || parent_subscriber_qos.presentation.coherent_access
                != writer_info.presentation.coherent_access
            || parent_subscriber_qos.presentation.ordered_access
                != writer_info.presentation.ordered_access
        {
            incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
        }
        if reader_qos.durability > writer_info.durability {
            incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
        }
        if reader_qos.deadline > writer_info.deadline {
            incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
        }
        if reader_qos.latency_budget > writer_info.latency_budget {
            incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
        }
        if reader_qos.liveliness > writer_info.liveliness {
            incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
        }
        if reader_qos.reliability.kind > writer_info.reliability.kind {
            incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
        }
        if reader_qos.destination_order > writer_info.destination_order {
            incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
        }

        incompatible_qos_policy_list
    }

    pub fn remove_matched_writer(&self, discovered_writer_handle: InstanceHandle) {
        let matched_publication = self
            .matched_publication_list
            .write_lock()
            .remove(&discovered_writer_handle);
        if let Some(w) = matched_publication {
            self.rtps_reader
                .write_lock()
                .matched_writer_remove(w.key.value.into());

            self.on_subscription_matched(discovered_writer_handle)
        }
    }

    pub fn read<Foo>(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader.write_lock().reader_mut().read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn take<Foo>(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader.write_lock().reader_mut().take(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn read_next_instance<Foo>(
        &self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader
            .write_lock()
            .reader_mut()
            .read_next_instance(
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            )
    }

    pub fn take_next_instance<Foo>(
        &self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader
            .write_lock()
            .reader_mut()
            .take_next_instance(
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            )
    }

    pub fn get_key_value<Foo>(
        &self,
        _key_holder: &mut Foo,
        _handle: InstanceHandle,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn lookup_instance<Foo>(&self, _instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }

    pub fn get_liveliness_changed_status(&self) -> LivelinessChangedStatus {
        self.liveliness_changed_status.write_lock().read_and_reset()
    }

    pub fn get_requested_deadline_missed_status(&self) -> RequestedDeadlineMissedStatus {
        self.requested_deadline_missed_status
            .write_lock()
            .read_and_reset()
    }

    pub fn get_requested_incompatible_qos_status(&self) -> RequestedIncompatibleQosStatus {
        self.requested_incompatible_qos_status
            .write_lock()
            .read_and_reset()
    }

    pub fn get_sample_lost_status(&self) -> SampleLostStatus {
        self.sample_lost_status.write_lock().read_and_reset()
    }

    pub fn get_sample_rejected_status(&self) -> SampleRejectedStatus {
        self.status_condition
            .write_lock()
            .remove_communication_state(StatusKind::SampleRejected);
        self.sample_rejected_status.write_lock().read_and_reset()
    }

    pub fn get_subscription_matched_status(&self) -> SubscriptionMatchedStatus {
        self.status_condition
            .write_lock()
            .remove_communication_state(StatusKind::SubscriptionMatched);
        self.subscription_matched_status
            .write_lock()
            .read_and_reset(self.matched_publication_list.read_lock().len() as i32)
    }

    pub fn get_topicdescription(&self) -> DdsShared<TopicImpl> {
        self.topic.clone()
    }

    pub fn get_subscriber(&self) -> DdsShared<UserDefinedSubscriber> {
        self.parent_subscriber
            .upgrade()
            .expect("Parent subscriber of data reader must exist")
    }

    pub fn wait_for_historical_data(&self, _max_wait: Duration) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.matched_publication_list
            .read_lock()
            .get(&publication_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    pub fn get_matched_publications(&self) -> Vec<InstanceHandle> {
        self.matched_publication_list
            .read_lock()
            .iter()
            .map(|(&key, _)| key)
            .collect()
    }

    pub fn set_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        if self.is_enabled() {
            self.rtps_reader
                .write_lock()
                .reader()
                .get_qos()
                .check_immutability(&qos)?;
        }

        self.rtps_reader.write_lock().reader_mut().set_qos(qos)?;

        if self.is_enabled() {
            self.get_subscriber()
                .get_participant()
                .announce_created_datareader(self.as_discovered_reader_data());
        }

        Ok(())
    }

    pub fn get_qos(&self) -> DataReaderQos {
        self.rtps_reader.read_lock().reader().get_qos().clone()
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        mask: &[StatusKind],
    ) {
        *self.listener.write_lock() = a_listener;
        *self.listener_status_mask.write_lock() = mask.to_vec();
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_status_changes(&self) -> Vec<StatusKind> {
        self.status_condition.write_lock().get_enabled_statuses()
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }

    pub fn enable(&self) -> DdsResult<()> {
        if !self.get_subscriber().is_enabled() {
            return Err(DdsError::PreconditionNotMet(
                "Parent subscriber disabled".to_string(),
            ));
        }

        self.get_subscriber()
            .get_participant()
            .announce_created_datareader(self.as_discovered_reader_data());
        *self.enabled.write_lock() = true;

        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_reader.read_lock().reader().guid().into()
    }

    pub fn as_discovered_reader_data(&self) -> DiscoveredReaderData {
        let guid = self.rtps_reader.read_lock().reader().guid();
        let reader_qos = self.rtps_reader.read_lock().reader().get_qos().clone();
        let topic_qos = self.topic.get_qos();
        let subscriber_qos = self.get_subscriber().get_qos();

        DiscoveredReaderData {
            reader_proxy: ReaderProxy {
                remote_reader_guid: guid,
                remote_group_entity_id: guid.entity_id(),
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                expects_inline_qos: false,
            },

            subscription_builtin_topic_data: SubscriptionBuiltinTopicData {
                key: BuiltInTopicKey { value: guid.into() },
                participant_key: BuiltInTopicKey {
                    value: GUID_UNKNOWN.into(),
                },
                topic_name: self.topic.get_name(),
                type_name: self.topic.get_type_name().to_string(),
                durability: reader_qos.durability.clone(),
                deadline: reader_qos.deadline.clone(),
                latency_budget: reader_qos.latency_budget.clone(),
                liveliness: reader_qos.liveliness.clone(),
                reliability: reader_qos.reliability.clone(),
                ownership: reader_qos.ownership.clone(),
                destination_order: reader_qos.destination_order.clone(),
                user_data: reader_qos.user_data.clone(),
                time_based_filter: reader_qos.time_based_filter,
                presentation: subscriber_qos.presentation.clone(),
                partition: subscriber_qos.partition.clone(),
                topic_data: topic_qos.topic_data,
                group_data: subscriber_qos.group_data,
            },
        }
    }

    pub fn send_message(&self, header: RtpsMessageHeader, transport: &mut impl TransportWrite) {
        self.rtps_reader
            .write_lock()
            .send_message(header, transport);
    }

    pub fn update_communication_status(&self, now: Time) {
        let (missed_deadline_instances, instance_reception_time) = self
            .instance_reception_time
            .write_lock()
            .iter()
            .partition(|&(_, received_time)| {
                now - *received_time
                    > self
                        .rtps_reader
                        .read_lock()
                        .reader()
                        .get_qos()
                        .deadline
                        .period
            });

        *self.instance_reception_time.write_lock() = instance_reception_time;

        for (missed_deadline_instance, _) in missed_deadline_instances {
            self.on_requested_deadline_missed(missed_deadline_instance);
        }
    }

    pub fn on_data_available(&self) {
        if *self.data_available_status_changed_flag.read_lock() {
            if self
                .listener_status_mask
                .read_lock()
                .contains(&StatusKind::DataAvailable)
            {
                if let Some(listener) = self.listener.write_lock().as_mut() {
                    *self.data_available_status_changed_flag.write_lock() = false;
                    listener.trigger_on_data_available(self);
                }
            }

            self.status_condition
                .write_lock()
                .add_communication_state(StatusKind::DataAvailable);
        }
    }

    fn on_sample_lost(&self) {
        self.sample_lost_status.write_lock().increment();

        if self
            .listener_status_mask
            .read_lock()
            .contains(&StatusKind::SampleLost)
        {
            if let Some(listener) = self.listener.write_lock().as_mut() {
                listener.trigger_on_sample_lost(self);
            }
        }

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::SampleLost);
    }

    fn on_subscription_matched(&self, instance_handle: InstanceHandle) {
        self.subscription_matched_status
            .write_lock()
            .increment(instance_handle);

        match self.listener.write_lock().as_mut() {
            Some(l)
                if self
                    .listener_status_mask
                    .read_lock()
                    .contains(&StatusKind::SubscriptionMatched) =>
            {
                l.trigger_on_subscription_matched(self)
            }
            _ => self.get_subscriber().on_subscription_matched(self),
        }

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::SubscriptionMatched);
    }

    fn on_sample_rejected(
        &self,
        instance_handle: InstanceHandle,
        rejected_reason: SampleRejectedStatusKind,
    ) {
        self.sample_rejected_status
            .write_lock()
            .increment(instance_handle, rejected_reason);

        match self.listener.write_lock().as_mut() {
            Some(l)
                if self
                    .listener_status_mask
                    .read_lock()
                    .contains(&StatusKind::SampleRejected) =>
            {
                l.trigger_on_sample_rejected(self)
            }
            _ => self.get_subscriber().on_sample_rejected(self),
        }

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::SampleRejected);
    }

    fn on_requested_deadline_missed(&self, instance_handle: InstanceHandle) {
        self.requested_deadline_missed_status
            .write_lock()
            .increment(instance_handle);

        match self.listener.write_lock().as_mut() {
            Some(l)
                if self
                    .listener_status_mask
                    .read_lock()
                    .contains(&StatusKind::RequestedDeadlineMissed) =>
            {
                l.trigger_on_requested_deadline_missed(self)
            }
            _ => self.get_subscriber().on_requested_deadline_missed(self),
        }

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::RequestedDeadlineMissed);
    }

    fn on_requested_incompatible_qos(&self, incompatible_qos_policy_list: Vec<QosPolicyId>) {
        self.requested_incompatible_qos_status
            .write_lock()
            .increment(incompatible_qos_policy_list);

        match self.listener.write_lock().as_mut() {
            Some(l)
                if self
                    .listener_status_mask
                    .read_lock()
                    .contains(&StatusKind::RequestedIncompatibleQos) =>
            {
                l.trigger_on_requested_incompatible_qos(self)
            }
            _ => self.get_subscriber().on_requested_incompatible_qos(self),
        }

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::RequestedIncompatibleQos);
    }
}

impl AnyDataReader for DdsShared<UserDefinedDataReader> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        implementation::rtps::{
            endpoint::RtpsEndpoint,
            reader::RtpsReader,
            types::{EntityId, EntityKey, Guid, TopicKind, BUILT_IN_PARTICIPANT, GUID_UNKNOWN},
        },
        infrastructure::qos::TopicQos,
        infrastructure::time::DURATION_ZERO,
        topic_definition::type_support::DdsSerialize,
    };
    use crate::{
        implementation::{dds_impl::topic_impl::TopicImpl, utils::shared_object::DdsShared},
        topic_definition::type_support::{DdsType, Endianness},
    };

    use mockall::mock;
    use std::io::Write;

    struct UserData(u8);

    impl DdsType for UserData {
        fn type_name() -> &'static str {
            "UserData"
        }

        fn has_key() -> bool {
            false
        }
    }

    impl<'de> DdsDeserialize<'de> for UserData {
        fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
            Ok(UserData(buf[0]))
        }
    }

    impl DdsSerialize for UserData {
        fn serialize<W: Write, E: Endianness>(&self, mut writer: W) -> DdsResult<()> {
            writer
                .write(&[self.0])
                .map(|_| ())
                .map_err(|e| DdsError::PreconditionNotMet(format!("{}", e)))
        }
    }

    mock! {
        Listener {}
        impl AnyDataReaderListener for Listener {
            fn trigger_on_data_available(&mut self, reader: &DdsShared<UserDefinedDataReader>);
            fn trigger_on_sample_rejected(
                &mut self,
                reader: &DdsShared<UserDefinedDataReader>,
            );
            fn trigger_on_liveliness_changed(
                &mut self,
                reader: &DdsShared<UserDefinedDataReader>,

            );
            fn trigger_on_requested_deadline_missed(
                &mut self,
                reader: &DdsShared<UserDefinedDataReader>,
            );
            fn trigger_on_requested_incompatible_qos(
                &mut self,
                reader: &DdsShared<UserDefinedDataReader>,
            );
            fn trigger_on_subscription_matched(
                &mut self,
                reader: &DdsShared<UserDefinedDataReader>,
            );
            fn trigger_on_sample_lost(
                &mut self,
                reader: &DdsShared<UserDefinedDataReader>,
            );
        }
    }

    #[test]
    fn get_instance_handle() {
        let guid = Guid::new(
            GuidPrefix::new([4; 12]),
            EntityId::new(EntityKey::new([3; 3]), BUILT_IN_PARTICIPANT),
        );
        let dummy_topic = TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            "",
            "",
            None,
            &[],
            DdsWeak::new(),
        );
        let qos = DataReaderQos::default();
        let stateful_reader = RtpsStatefulReader::new(RtpsReader::new::<UserData>(
            RtpsEndpoint::new(guid, TopicKind::NoKey, &[], &[]),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
            qos,
        ));

        let data_reader: DdsShared<UserDefinedDataReader> = UserDefinedDataReader::new(
            stateful_reader,
            dummy_topic,
            None,
            &[],
            DdsWeak::new(),
            DdsCondvar::new(),
        );
        *data_reader.enabled.write_lock() = true;

        let expected_instance_handle: InstanceHandle = guid.into();
        let instance_handle = data_reader.get_instance_handle();
        assert_eq!(expected_instance_handle, instance_handle);
    }
}
