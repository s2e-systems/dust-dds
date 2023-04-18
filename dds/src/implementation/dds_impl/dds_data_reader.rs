use std::{
    collections::{HashMap, HashSet},
    sync::RwLockWriteGuard,
};

use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_writer_data::DiscoveredWriterData,
        },
        rtps::{
            messages::{
                overall_structure::RtpsMessageHeader,
                submessages::{
                    DataFragSubmessage, DataSubmessage, GapSubmessage, HeartbeatFragSubmessage,
                    HeartbeatSubmessage,
                },
            },
            reader::{RtpsReaderCacheChange, RtpsReaderResult},
            stateful_reader::{RtpsStatefulReader, StatefulReaderDataReceivedResult},
            stateless_reader::{RtpsStatelessReader, StatelessReaderDataReceivedResult},
            transport::TransportWrite,
            types::{Guid, GuidPrefix, Locator, GUID_UNKNOWN},
            writer_proxy::RtpsWriterProxy,
        },
        utils::{
            condvar::DdsCondvar,
            node::RootNode,
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos, TopicQos},
        qos_policy::{
            DurabilityQosPolicyKind, QosPolicyId, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID,
        },
        status::{
            LivelinessChangedStatus, QosPolicyCount, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
            SampleRejectedStatusKind, StatusKind, SubscriptionMatchedStatus,
        },
        time::{Duration, DurationKind, Time},
    },
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
        subscriber_listener::SubscriberListener,
    },
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    any_data_reader_listener::AnyDataReaderListener, message_receiver::MessageReceiver,
    node_listener_data_reader::ListenerDataReaderNode, status_condition_impl::StatusConditionImpl,
    status_listener::StatusListener,
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

pub struct DdsDataReader<T> {
    rtps_reader: DdsRwLock<T>,
    type_name: &'static str,
    topic_name: String,
    status_listener: DdsRwLock<StatusListener<dyn AnyDataReaderListener + Send + Sync>>,
    liveliness_changed_status: DdsRwLock<LivelinessChangedStatus>,
    requested_deadline_missed_status: DdsRwLock<RequestedDeadlineMissedStatus>,
    requested_incompatible_qos_status: DdsRwLock<RequestedIncompatibleQosStatus>,
    sample_lost_status: DdsRwLock<SampleLostStatus>,
    sample_rejected_status: DdsRwLock<SampleRejectedStatus>,
    subscription_matched_status: DdsRwLock<SubscriptionMatchedStatus>,
    matched_publication_list: DdsRwLock<HashMap<InstanceHandle, PublicationBuiltinTopicData>>,
    enabled: DdsRwLock<bool>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    instance_reception_time: DdsRwLock<HashMap<InstanceHandle, Time>>,
    data_available_status_changed_flag: DdsRwLock<bool>,
    wait_for_historical_data_condvar: DdsCondvar,
    incompatible_writer_list: DdsRwLock<HashSet<InstanceHandle>>,
}

impl<T> DdsDataReader<T> {
    pub fn new(
        rtps_reader: T,
        type_name: &'static str,
        topic_name: String,
        listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsShared<Self> {
        DdsShared::new(DdsDataReader {
            rtps_reader: DdsRwLock::new(rtps_reader),
            type_name,
            topic_name,
            status_listener: DdsRwLock::new(StatusListener::new(listener, mask)),
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
            instance_reception_time: DdsRwLock::new(HashMap::new()),
            data_available_status_changed_flag: DdsRwLock::new(false),
            wait_for_historical_data_condvar: DdsCondvar::new(),
            incompatible_writer_list: DdsRwLock::new(HashSet::new()),
        })
    }

    pub fn get_status_listener_lock(
        &self,
    ) -> RwLockWriteGuard<StatusListener<dyn AnyDataReaderListener + Send + Sync>> {
        self.status_listener.write_lock()
    }

    pub fn get_type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
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

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_status_changes(&self) -> Vec<StatusKind> {
        self.status_condition.read_lock().get_status_changes()
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }

    pub fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;
        Ok(())
    }
}

impl DdsShared<DdsDataReader<RtpsStatefulReader>> {
    pub fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) -> UserDefinedReaderDataSubmessageReceivedResult {
        let data_submessage_received_result = self
            .rtps_reader
            .write_lock()
            .on_data_submessage_received(data_submessage, message_receiver);
        self.wait_for_historical_data_condvar.notify_all();
        match data_submessage_received_result {
            StatefulReaderDataReceivedResult::NoMatchedWriterProxy => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::NewSampleAdded(instance_handle) => {
                *self.data_available_status_changed_flag.write_lock() = true;
                self.instance_reception_time
                    .write_lock()
                    .insert(instance_handle, message_receiver.reception_timestamp());

                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::NewSampleAddedAndSamplesLost(instance_handle) => {
                self.instance_reception_time
                    .write_lock()
                    .insert(instance_handle, message_receiver.reception_timestamp());
                *self.data_available_status_changed_flag.write_lock() = true;
                self.on_sample_lost(subscriber_status_listener, participant_status_listener);
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::SampleRejected(instance_handle, rejected_reason) => {
                self.on_sample_rejected(
                    instance_handle,
                    rejected_reason,
                    subscriber_status_listener,
                    participant_status_listener,
                );
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
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) -> UserDefinedReaderDataSubmessageReceivedResult {
        let data_submessage_received_result = self
            .rtps_reader
            .write_lock()
            .on_data_frag_submessage_received(data_frag_submessage, message_receiver);
        self.wait_for_historical_data_condvar.notify_all();

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
                self.on_sample_lost(subscriber_status_listener, participant_status_listener);
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::SampleRejected(instance_handle, rejected_reason) => {
                self.on_sample_rejected(
                    instance_handle,
                    rejected_reason,
                    subscriber_status_listener,
                    participant_status_listener,
                );
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
        self.wait_for_historical_data_condvar.notify_all();
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
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
        subscriber_qos: &SubscriberQos,
    ) {
        let publication_builtin_topic_data = &discovered_writer_data.publication_builtin_topic_data;
        if publication_builtin_topic_data.topic_name == self.topic_name
            && publication_builtin_topic_data.type_name == self.type_name
        {
            let instance_handle = discovered_writer_data.get_serialized_key().into();
            let incompatible_qos_policy_list = self
                .get_discovered_writer_incompatible_qos_policy_list(
                    discovered_writer_data,
                    subscriber_qos,
                );
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
                let insert_matched_publication_result = self
                    .matched_publication_list
                    .write_lock()
                    .insert(instance_handle, publication_builtin_topic_data.clone());
                match insert_matched_publication_result {
                    Some(value) if &value != publication_builtin_topic_data => self
                        .on_subscription_matched(
                            instance_handle,
                            subscriber_status_listener,
                            participant_status_listener,
                        ),
                    None => self.on_subscription_matched(
                        instance_handle,
                        subscriber_status_listener,
                        participant_status_listener,
                    ),
                    _ => (),
                }
            } else if self
                .incompatible_writer_list
                .write_lock()
                .insert(instance_handle)
            {
                self.on_requested_incompatible_qos(
                    incompatible_qos_policy_list,
                    subscriber_status_listener,
                    participant_status_listener,
                );
            }
        }
    }

    fn get_discovered_writer_incompatible_qos_policy_list(
        &self,
        discovered_writer_data: &DiscoveredWriterData,
        subscriber_qos: &SubscriberQos,
    ) -> Vec<QosPolicyId> {
        let writer_info = &discovered_writer_data.publication_builtin_topic_data;
        let reader_qos = self.rtps_reader.read_lock().get_qos().clone();

        let mut incompatible_qos_policy_list = Vec::new();

        if subscriber_qos.presentation.access_scope > writer_info.presentation.access_scope
            || subscriber_qos.presentation.coherent_access
                != writer_info.presentation.coherent_access
            || subscriber_qos.presentation.ordered_access != writer_info.presentation.ordered_access
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

    pub fn remove_matched_writer(
        &self,
        discovered_writer_handle: InstanceHandle,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let matched_publication = self
            .matched_publication_list
            .write_lock()
            .remove(&discovered_writer_handle);
        if let Some(w) = matched_publication {
            self.rtps_reader
                .write_lock()
                .matched_writer_remove(w.key.value.into());

            self.on_subscription_matched(
                discovered_writer_handle,
                subscriber_status_listener,
                participant_status_listener,
            )
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

        self.rtps_reader.write_lock().read(
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

        self.rtps_reader.write_lock().take(
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

        self.rtps_reader.write_lock().read_next_instance(
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

        self.rtps_reader.write_lock().take_next_instance(
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

    pub fn wait_for_historical_data(&self, max_wait: Duration) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            Err(DdsError::NotEnabled)
        } else {
            Ok(())
        }?;

        match self.rtps_reader.read_lock().get_qos().durability.kind {
            DurabilityQosPolicyKind::Volatile => Err(DdsError::IllegalOperation),
            DurabilityQosPolicyKind::TransientLocal => Ok(()),
        }?;

        let start_time = std::time::Instant::now();

        while start_time.elapsed() < std::time::Duration::from(max_wait) {
            if self.rtps_reader.read_lock().is_historical_data_received() {
                return Ok(());
            }
            let duration_until_timeout = Duration::from(start_time.elapsed()) - max_wait;
            self.wait_for_historical_data_condvar
                .wait_timeout(duration_until_timeout)
                .ok();
        }
        Err(DdsError::Timeout)
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
                .get_qos()
                .check_immutability(&qos)?;
        }

        self.rtps_reader.write_lock().set_qos(qos)?;

        Ok(())
    }

    pub fn get_qos(&self) -> DataReaderQos {
        self.rtps_reader.read_lock().get_qos().clone()
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_reader.read_lock().guid().into()
    }

    pub fn as_discovered_reader_data(
        &self,
        topic_qos: &TopicQos,
        subscriber_qos: &SubscriberQos,
    ) -> DiscoveredReaderData {
        let guid = self.rtps_reader.read_lock().guid();
        let reader_qos = self.rtps_reader.read_lock().get_qos().clone();

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
                topic_name: self.topic_name.clone(),
                type_name: self.type_name.to_string(),
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
                topic_data: topic_qos.topic_data.clone(),
                group_data: subscriber_qos.group_data.clone(),
            },
        }
    }

    pub fn send_message(&self, header: RtpsMessageHeader, transport: &mut impl TransportWrite) {
        self.rtps_reader
            .write_lock()
            .send_message(header, transport);
    }

    pub fn update_communication_status(
        &self,
        now: Time,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let (missed_deadline_instances, instance_reception_time) = self
            .instance_reception_time
            .write_lock()
            .iter()
            .partition(|&(_, received_time)| {
                DurationKind::Finite(now - *received_time)
                    > self.rtps_reader.read_lock().get_qos().deadline.period
            });

        *self.instance_reception_time.write_lock() = instance_reception_time;

        for (missed_deadline_instance, _) in missed_deadline_instances {
            self.on_requested_deadline_missed(
                missed_deadline_instance,
                subscriber_status_listener,
                participant_status_listener,
            );
        }
    }

    pub fn on_gap_submessage_received(
        &self,
        gap_submessage: &GapSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        self.rtps_reader
            .write_lock()
            .on_gap_submessage_received(gap_submessage, source_guid_prefix);
    }

    pub fn on_data_available(
        &self,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        self.trigger_on_data_available_listener(
            &mut self.status_listener.write_lock(),
            participant_status_listener,
        );

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::DataAvailable);
    }

    fn trigger_on_data_available_listener(
        &self,
        reader_status_listener: &mut StatusListener<dyn AnyDataReaderListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let on_data_available_status_kind = &StatusKind::DataAvailable;
        if reader_status_listener.is_enabled(on_data_available_status_kind) {
            reader_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .trigger_on_data_available(ListenerDataReaderNode::new(RootNode::new(
                    self.downgrade(),
                )))
        } else if participant_status_listener.is_enabled(on_data_available_status_kind) {
            participant_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_data_available(&ListenerDataReaderNode::new(RootNode::new(
                    self.downgrade(),
                )))
        }
    }

    fn on_sample_lost(
        &self,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        self.sample_lost_status.write_lock().increment();

        self.trigger_on_sample_lost_listener(
            &mut self.status_listener.write_lock(),
            subscriber_status_listener,
            participant_status_listener,
        );

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::SampleLost);
    }

    fn trigger_on_sample_lost_listener(
        &self,
        reader_status_listener: &mut StatusListener<dyn AnyDataReaderListener + Send + Sync>,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let sample_lost_status_kind = &StatusKind::SampleLost;
        if reader_status_listener.is_enabled(sample_lost_status_kind) {
            reader_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .trigger_on_sample_lost(
                    ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_sample_lost_status(),
                )
        } else if subscriber_status_listener.is_enabled(sample_lost_status_kind) {
            subscriber_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_sample_lost(
                    &ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_sample_lost_status(),
                )
        } else if participant_status_listener.is_enabled(sample_lost_status_kind) {
            participant_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_sample_lost(
                    &ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_sample_lost_status(),
                )
        }
    }

    fn on_subscription_matched(
        &self,
        instance_handle: InstanceHandle,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        self.subscription_matched_status
            .write_lock()
            .increment(instance_handle);

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::SubscriptionMatched);

        self.trigger_on_subscription_matched_listener(
            &mut self.get_status_listener_lock(),
            subscriber_status_listener,
            participant_status_listener,
        );
    }

    fn trigger_on_subscription_matched_listener(
        &self,
        reader_status_listener: &mut StatusListener<dyn AnyDataReaderListener + Send + Sync>,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let subscription_matched_status_kind = &StatusKind::SubscriptionMatched;
        if reader_status_listener.is_enabled(subscription_matched_status_kind) {
            reader_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .trigger_on_subscription_matched(
                    ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_subscription_matched_status(),
                )
        } else if subscriber_status_listener.is_enabled(subscription_matched_status_kind) {
            subscriber_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_subscription_matched(
                    &ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_subscription_matched_status(),
                )
        } else if participant_status_listener.is_enabled(subscription_matched_status_kind) {
            participant_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_subscription_matched(
                    &ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_subscription_matched_status(),
                )
        }
    }

    fn on_sample_rejected(
        &self,
        instance_handle: InstanceHandle,
        rejected_reason: SampleRejectedStatusKind,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        self.sample_rejected_status
            .write_lock()
            .increment(instance_handle, rejected_reason);

        self.trigger_on_sample_rejected_listener(
            &mut self.status_listener.write_lock(),
            subscriber_status_listener,
            participant_status_listener,
        );

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::SampleRejected);
    }

    fn trigger_on_sample_rejected_listener(
        &self,
        reader_status_listener: &mut StatusListener<dyn AnyDataReaderListener + Send + Sync>,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let sample_rejected_status_kind = &StatusKind::SampleRejected;
        if reader_status_listener.is_enabled(sample_rejected_status_kind) {
            reader_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .trigger_on_sample_rejected(
                    ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_sample_rejected_status(),
                )
        } else if subscriber_status_listener.is_enabled(sample_rejected_status_kind) {
            subscriber_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_sample_rejected(
                    &ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_sample_rejected_status(),
                )
        } else if participant_status_listener.is_enabled(sample_rejected_status_kind) {
            participant_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_sample_rejected(
                    &ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_sample_rejected_status(),
                )
        }
    }

    fn on_requested_deadline_missed(
        &self,
        instance_handle: InstanceHandle,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        self.requested_deadline_missed_status
            .write_lock()
            .increment(instance_handle);

        self.trigger_on_requested_deadline_missed_listener(
            &mut self.status_listener.write_lock(),
            subscriber_status_listener,
            participant_status_listener,
        );

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::RequestedDeadlineMissed);
    }

    fn trigger_on_requested_deadline_missed_listener(
        &self,
        reader_status_listener: &mut StatusListener<dyn AnyDataReaderListener + Send + Sync>,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let requested_deadline_missed_status_kind = &StatusKind::RequestedDeadlineMissed;
        if reader_status_listener.is_enabled(requested_deadline_missed_status_kind) {
            reader_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .trigger_on_requested_deadline_missed(
                    ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_requested_deadline_missed_status(),
                )
        } else if subscriber_status_listener.is_enabled(requested_deadline_missed_status_kind) {
            subscriber_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_requested_deadline_missed(
                    &ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_requested_deadline_missed_status(),
                )
        } else if participant_status_listener.is_enabled(requested_deadline_missed_status_kind) {
            participant_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_requested_deadline_missed(
                    &ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_requested_deadline_missed_status(),
                )
        }
    }

    fn on_requested_incompatible_qos(
        &self,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        self.requested_incompatible_qos_status
            .write_lock()
            .increment(incompatible_qos_policy_list);

        self.trigger_on_requested_incompatible_qos_listener(
            &mut self.status_listener.write_lock(),
            subscriber_status_listener,
            participant_status_listener,
        );

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::RequestedIncompatibleQos);
    }

    fn trigger_on_requested_incompatible_qos_listener(
        &self,
        reader_status_listener: &mut StatusListener<dyn AnyDataReaderListener + Send + Sync>,
        subscriber_status_listener: &mut StatusListener<dyn SubscriberListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let requested_incompatible_qos_status_kind = &StatusKind::RequestedIncompatibleQos;
        if reader_status_listener.is_enabled(requested_incompatible_qos_status_kind) {
            reader_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .trigger_on_requested_incompatible_qos(
                    ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_requested_incompatible_qos_status(),
                )
        } else if subscriber_status_listener.is_enabled(requested_incompatible_qos_status_kind) {
            subscriber_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_requested_incompatible_qos(
                    &ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_requested_incompatible_qos_status(),
                )
        } else if participant_status_listener.is_enabled(requested_incompatible_qos_status_kind) {
            participant_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_requested_incompatible_qos(
                    &ListenerDataReaderNode::new(RootNode::new(self.downgrade())),
                    self.get_requested_incompatible_qos_status(),
                )
        }
    }
}

impl DdsDataReader<RtpsStatefulReader> {
    pub fn matched_writer_add(&self, a_writer_proxy: RtpsWriterProxy) {
        self.rtps_reader
            .write_lock()
            .matched_writer_add(a_writer_proxy)
    }

    pub fn _matched_writer_remove(&self, a_writer_guid: Guid) {
        self.rtps_reader
            .write_lock()
            .matched_writer_remove(a_writer_guid)
    }
}

impl DdsDataReader<RtpsStatelessReader> {
    pub fn guid(&self) -> Guid {
        self.rtps_reader.read_lock().guid()
    }

    pub fn _convert_data_to_cache_change(
        &self,
        data_submessage: &DataSubmessage,
        source_timestamp: Option<Time>,
        source_guid_prefix: GuidPrefix,
        reception_timestamp: Time,
    ) -> RtpsReaderResult<RtpsReaderCacheChange> {
        self.rtps_reader.read_lock()._convert_data_to_cache_change(
            data_submessage,
            source_timestamp,
            source_guid_prefix,
            reception_timestamp,
        )
    }

    pub fn _add_change(&self, change: RtpsReaderCacheChange) -> RtpsReaderResult<InstanceHandle> {
        self.rtps_reader.write_lock()._add_change(change)
    }

    pub fn get_qos(&self) -> DataReaderQos {
        self.rtps_reader.read_lock().get_qos().clone()
    }

    pub fn _set_qos(&self, qos: DataReaderQos) -> DdsResult<()> {
        self.rtps_reader.write_lock()._set_qos(qos)
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
        self.rtps_reader.write_lock().read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn _take<Foo>(
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
        self.rtps_reader.write_lock()._take(
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
        self.rtps_reader.write_lock().read_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn _take_next_instance<Foo>(
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
        self.rtps_reader.write_lock()._take_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    ) -> StatelessReaderDataReceivedResult {
        self.rtps_reader
            .write_lock()
            .on_data_submessage_received(data_submessage, message_receiver)
    }

    pub fn _get_instance_handle(&self) -> InstanceHandle {
        self.rtps_reader.read_lock().guid().into()
    }
}
