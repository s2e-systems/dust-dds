use std::collections::{HashMap, HashSet};

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
                    DataFragSubmessageRead, DataSubmessageRead,
                    HeartbeatFragSubmessage, HeartbeatSubmessageRead, GapSubmessageRead,
                },
            },
            reader::{RtpsReaderCacheChange, RtpsReaderResult},
            stateful_reader::{RtpsStatefulReader, StatefulReaderDataReceivedResult},
            stateless_reader::{RtpsStatelessReader, StatelessReaderDataReceivedResult},
            transport::TransportWrite,
            types::{Guid, GuidPrefix, Locator, GUID_UNKNOWN},
            writer_proxy::RtpsWriterProxy,
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
            SampleRejectedStatusKind, SubscriptionMatchedStatus,
        },
        time::{DurationKind, Time},
    },
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    message_receiver::MessageReceiver, nodes::DataReaderNode, status_listener::ListenerTriggerKind,
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
    rtps_reader: T,
    type_name: &'static str,
    topic_name: String,
    liveliness_changed_status: LivelinessChangedStatus,
    requested_deadline_missed_status: RequestedDeadlineMissedStatus,
    requested_incompatible_qos_status: RequestedIncompatibleQosStatus,
    sample_lost_status: SampleLostStatus,
    sample_rejected_status: SampleRejectedStatus,
    subscription_matched_status: SubscriptionMatchedStatus,
    matched_publication_list: HashMap<InstanceHandle, PublicationBuiltinTopicData>,
    enabled: bool,
    instance_reception_time: HashMap<InstanceHandle, Time>,
    data_available_status_changed_flag: bool,
    incompatible_writer_list: HashSet<InstanceHandle>,
}

impl<T> DdsDataReader<T> {
    pub fn new(rtps_reader: T, type_name: &'static str, topic_name: String) -> Self {
        DdsDataReader {
            rtps_reader,
            type_name,
            topic_name,
            liveliness_changed_status: LivelinessChangedStatus::default(),
            requested_deadline_missed_status: RequestedDeadlineMissedStatus::default(),
            requested_incompatible_qos_status: RequestedIncompatibleQosStatus::default(),
            sample_lost_status: SampleLostStatus::default(),
            sample_rejected_status: SampleRejectedStatus::default(),
            subscription_matched_status: SubscriptionMatchedStatus::default(),
            matched_publication_list: HashMap::new(),
            enabled: false,
            instance_reception_time: HashMap::new(),
            data_available_status_changed_flag: false,
            incompatible_writer_list: HashSet::new(),
        }
    }

    pub fn get_type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
    }

    pub fn get_liveliness_changed_status(&mut self) -> LivelinessChangedStatus {
        self.liveliness_changed_status.read_and_reset()
    }

    pub fn get_requested_deadline_missed_status(&mut self) -> RequestedDeadlineMissedStatus {
        self.requested_deadline_missed_status.read_and_reset()
    }

    pub fn get_requested_incompatible_qos_status(&mut self) -> RequestedIncompatibleQosStatus {
        self.requested_incompatible_qos_status.read_and_reset()
    }

    pub fn get_sample_lost_status(&mut self) -> SampleLostStatus {
        self.sample_lost_status.read_and_reset()
    }

    pub fn get_sample_rejected_status(&mut self) -> SampleRejectedStatus {
        self.sample_rejected_status.read_and_reset()
    }

    pub fn get_subscription_matched_status(&mut self) -> SubscriptionMatchedStatus {
        self.subscription_matched_status
            .read_and_reset(self.matched_publication_list.len() as i32)
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) -> DdsResult<()> {
        self.enabled = true;
        Ok(())
    }
}

impl DdsDataReader<RtpsStatefulReader> {
    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessageRead<'_>,
        message_receiver: &MessageReceiver,
        parent_subscriber_guid: Guid,
        parent_participant_guid: Guid,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) -> UserDefinedReaderDataSubmessageReceivedResult {
        let data_submessage_received_result = self
            .rtps_reader
            .on_data_submessage_received(data_submessage, message_receiver);
        match data_submessage_received_result {
            StatefulReaderDataReceivedResult::NoMatchedWriterProxy => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::NewSampleAdded(instance_handle) => {
                self.data_available_status_changed_flag = true;
                self.instance_reception_time
                    .insert(instance_handle, message_receiver.reception_timestamp());
                self.on_data_available(
                    parent_subscriber_guid,
                    parent_participant_guid,
                    listener_sender,
                );
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::NewSampleAddedAndSamplesLost(instance_handle) => {
                self.instance_reception_time
                    .insert(instance_handle, message_receiver.reception_timestamp());
                self.data_available_status_changed_flag = true;
                self.on_sample_lost(
                    parent_subscriber_guid,
                    parent_participant_guid,
                    listener_sender,
                );
                self.on_data_available(
                    parent_subscriber_guid,
                    parent_participant_guid,
                    listener_sender,
                );
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::SampleRejected(instance_handle, rejected_reason) => {
                self.on_sample_rejected(
                    instance_handle,
                    rejected_reason,
                    parent_subscriber_guid,
                    parent_participant_guid,
                    listener_sender,
                );
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::InvalidData(_) => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
        }
    }

    pub fn on_data_frag_submessage_received(
        &mut self,
        data_frag_submessage: &DataFragSubmessageRead<'_>,
        message_receiver: &MessageReceiver,
        parent_subscriber_guid: Guid,
        parent_participant_guid: Guid,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) -> UserDefinedReaderDataSubmessageReceivedResult {
        let data_submessage_received_result = self
            .rtps_reader
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
                    .insert(instance_handle, message_receiver.reception_timestamp());
                self.data_available_status_changed_flag = true;
                self.on_data_available(
                    parent_subscriber_guid,
                    parent_participant_guid,
                    listener_sender,
                );
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::NewSampleAddedAndSamplesLost(instance_handle) => {
                self.instance_reception_time
                    .insert(instance_handle, message_receiver.reception_timestamp());
                self.data_available_status_changed_flag = true;
                self.on_sample_lost(
                    parent_subscriber_guid,
                    parent_participant_guid,
                    listener_sender,
                );
                self.on_data_available(
                    parent_subscriber_guid,
                    parent_participant_guid,
                    listener_sender,
                );
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::SampleRejected(instance_handle, rejected_reason) => {
                self.on_sample_rejected(
                    instance_handle,
                    rejected_reason,
                    parent_subscriber_guid,
                    parent_participant_guid,
                    listener_sender,
                );
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::InvalidData(_) => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
        }
    }

    pub fn on_heartbeat_submessage_received(
        &mut self,
        heartbeat_submessage: &HeartbeatSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        self.rtps_reader
            .on_heartbeat_submessage_received(heartbeat_submessage, source_guid_prefix);
    }

    pub fn on_heartbeat_frag_submessage_received(
        &mut self,
        heartbeat_frag_submessage: &HeartbeatFragSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        self.rtps_reader
            .on_heartbeat_frag_submessage_received(heartbeat_frag_submessage, source_guid_prefix);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_matched_writer(
        &mut self,
        discovered_writer_data: &DiscoveredWriterData,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
        subscriber_qos: &SubscriberQos,
        parent_subscriber_guid: Guid,
        parent_participant_guid: Guid,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) {
        let publication_builtin_topic_data = discovered_writer_data.dds_publication_data();
        if publication_builtin_topic_data.topic_name() == self.topic_name
            && publication_builtin_topic_data.get_type_name() == self.type_name
        {
            let instance_handle = discovered_writer_data.get_serialized_key().into();
            let incompatible_qos_policy_list = self
                .get_discovered_writer_incompatible_qos_policy_list(
                    discovered_writer_data,
                    subscriber_qos,
                );
            if incompatible_qos_policy_list.is_empty() {
                let unicast_locator_list = if discovered_writer_data
                    .writer_proxy()
                    .unicast_locator_list()
                    .is_empty()
                {
                    default_unicast_locator_list
                } else {
                    discovered_writer_data.writer_proxy().unicast_locator_list()
                };

                let multicast_locator_list = if discovered_writer_data
                    .writer_proxy()
                    .multicast_locator_list()
                    .is_empty()
                {
                    default_multicast_locator_list
                } else {
                    discovered_writer_data
                        .writer_proxy()
                        .multicast_locator_list()
                };

                let writer_proxy = RtpsWriterProxy::new(
                    discovered_writer_data.writer_proxy().remote_writer_guid(),
                    unicast_locator_list,
                    multicast_locator_list,
                    discovered_writer_data
                        .writer_proxy()
                        .data_max_size_serialized(),
                    discovered_writer_data
                        .writer_proxy()
                        .remote_group_entity_id(),
                );

                self.rtps_reader.matched_writer_add(writer_proxy);
                let insert_matched_publication_result = self
                    .matched_publication_list
                    .insert(instance_handle, publication_builtin_topic_data.clone());
                match insert_matched_publication_result {
                    Some(value) if &value != publication_builtin_topic_data => self
                        .on_subscription_matched(
                            instance_handle,
                            parent_subscriber_guid,
                            parent_participant_guid,
                            listener_sender,
                        ),
                    None => self.on_subscription_matched(
                        instance_handle,
                        parent_subscriber_guid,
                        parent_participant_guid,
                        listener_sender,
                    ),
                    _ => (),
                }
            } else if self.incompatible_writer_list.insert(instance_handle) {
                self.on_requested_incompatible_qos(
                    incompatible_qos_policy_list,
                    parent_subscriber_guid,
                    parent_participant_guid,
                    listener_sender,
                );
            }
        }
    }

    fn get_discovered_writer_incompatible_qos_policy_list(
        &self,
        discovered_writer_data: &DiscoveredWriterData,
        subscriber_qos: &SubscriberQos,
    ) -> Vec<QosPolicyId> {
        let writer_info = discovered_writer_data.dds_publication_data();
        let reader_qos = self.rtps_reader.get_qos().clone();

        let mut incompatible_qos_policy_list = Vec::new();

        if subscriber_qos.presentation.access_scope > writer_info.presentation().access_scope
            || subscriber_qos.presentation.coherent_access
                != writer_info.presentation().coherent_access
            || subscriber_qos.presentation.ordered_access
                != writer_info.presentation().ordered_access
        {
            incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
        }
        if &reader_qos.durability > writer_info.durability() {
            incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
        }
        if &reader_qos.deadline > writer_info.deadline() {
            incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
        }
        if &reader_qos.latency_budget > writer_info.latency_budget() {
            incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
        }
        if &reader_qos.liveliness > writer_info.liveliness() {
            incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
        }
        if reader_qos.reliability.kind > writer_info.reliability().kind {
            incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
        }
        if &reader_qos.destination_order > writer_info.destination_order() {
            incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
        }

        incompatible_qos_policy_list
    }

    pub fn remove_matched_writer(
        &mut self,
        discovered_writer_handle: InstanceHandle,
        parent_subscriber_guid: Guid,
        parent_participant_guid: Guid,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) {
        let matched_publication = self
            .matched_publication_list
            .remove(&discovered_writer_handle);
        if let Some(w) = matched_publication {
            self.rtps_reader.matched_writer_remove(w.key().value.into());

            self.on_subscription_matched(
                discovered_writer_handle,
                parent_subscriber_guid,
                parent_participant_guid,
                listener_sender,
            )
        }
    }

    pub fn read<Foo>(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader.read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn take<Foo>(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader.take(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn read_next_instance<Foo>(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader.read_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn take_next_instance<Foo>(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_reader.take_next_instance(
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
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn lookup_instance<Foo>(&self, _instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }

    pub fn is_historical_data_received(&self) -> DdsResult<bool> {
        if !self.enabled {
            Err(DdsError::NotEnabled)
        } else {
            Ok(())
        }?;

        match self.rtps_reader.get_qos().durability.kind {
            DurabilityQosPolicyKind::Volatile => Err(DdsError::IllegalOperation),
            DurabilityQosPolicyKind::TransientLocal => Ok(()),
        }?;

        Ok(self.rtps_reader.is_historical_data_received())
    }

    pub fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.matched_publication_list
            .get(&publication_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    pub fn get_matched_publications(&self) -> Vec<InstanceHandle> {
        self.matched_publication_list
            .iter()
            .map(|(&key, _)| key)
            .collect()
    }

    pub fn set_qos(&mut self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        if self.is_enabled() {
            self.rtps_reader.get_qos().check_immutability(&qos)?;
        }

        self.rtps_reader.set_qos(qos)?;

        Ok(())
    }

    pub fn get_qos(&self) -> DataReaderQos {
        self.rtps_reader.get_qos().clone()
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_reader.guid().into()
    }

    pub fn as_discovered_reader_data(
        &self,
        topic_qos: &TopicQos,
        subscriber_qos: &SubscriberQos,
    ) -> DiscoveredReaderData {
        let guid = self.rtps_reader.guid();
        let reader_qos = self.rtps_reader.get_qos().clone();

        DiscoveredReaderData::new(
            ReaderProxy::new(guid, guid.entity_id(), vec![], vec![], false),
            SubscriptionBuiltinTopicData::new(
                BuiltInTopicKey { value: guid.into() },
                BuiltInTopicKey {
                    value: GUID_UNKNOWN.into(),
                },
                self.topic_name.clone(),
                self.type_name.to_string(),
                reader_qos.durability.clone(),
                reader_qos.deadline.clone(),
                reader_qos.latency_budget.clone(),
                reader_qos.liveliness.clone(),
                reader_qos.reliability.clone(),
                reader_qos.ownership.clone(),
                reader_qos.destination_order.clone(),
                reader_qos.user_data.clone(),
                reader_qos.time_based_filter,
                subscriber_qos.presentation.clone(),
                subscriber_qos.partition.clone(),
                topic_qos.topic_data.clone(),
                subscriber_qos.group_data.clone(),
            ),
        )
    }

    pub fn send_message(&mut self, header: RtpsMessageHeader, transport: &mut impl TransportWrite) {
        self.rtps_reader.send_message(header, transport);
    }

    pub fn update_communication_status(
        &mut self,
        now: Time,
        parent_participant_guid: Guid,
        parent_subcriber_guid: Guid,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) {
        let (missed_deadline_instances, instance_reception_time) = self
            .instance_reception_time
            .iter()
            .partition(|&(_, received_time)| {
                DurationKind::Finite(now - *received_time)
                    > self.rtps_reader.get_qos().deadline.period
            });

        self.instance_reception_time = instance_reception_time;

        for (missed_deadline_instance, _) in missed_deadline_instances {
            self.requested_deadline_missed_status
                .increment(missed_deadline_instance);

            listener_sender
                .try_send(ListenerTriggerKind::RequestedDeadlineMissed(
                    DataReaderNode::new(
                        self.guid(),
                        parent_subcriber_guid,
                        parent_participant_guid,
                    ),
                ))
                .ok();
        }
    }

    pub fn on_gap_submessage_received(
        &mut self,
        gap_submessage: &GapSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        self.rtps_reader
            .on_gap_submessage_received(gap_submessage, source_guid_prefix);
    }

    pub fn on_data_available(
        &self,
        parent_subcriber_guid: Guid,
        parent_participant_guid: Guid,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) {
        listener_sender
            .try_send(ListenerTriggerKind::OnDataAvailable(DataReaderNode::new(
                self.guid(),
                parent_subcriber_guid,
                parent_participant_guid,
            )))
            .ok();
    }

    fn on_sample_lost(
        &mut self,
        parent_subscriber_guid: Guid,
        parent_participant_guid: Guid,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) {
        self.sample_lost_status.increment();

        listener_sender
            .try_send(ListenerTriggerKind::OnSampleLost(DataReaderNode::new(
                self.guid(),
                parent_subscriber_guid,
                parent_participant_guid,
            )))
            .ok();
    }

    fn on_subscription_matched(
        &mut self,
        instance_handle: InstanceHandle,
        parent_subscriber_guid: Guid,
        parent_participant_guid: Guid,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) {
        self.subscription_matched_status.increment(instance_handle);

        listener_sender
            .try_send(ListenerTriggerKind::SubscriptionMatched(
                DataReaderNode::new(self.guid(), parent_subscriber_guid, parent_participant_guid),
            ))
            .ok();
    }

    fn on_sample_rejected(
        &mut self,
        instance_handle: InstanceHandle,
        rejected_reason: SampleRejectedStatusKind,
        parent_subscriber_guid: Guid,
        parent_participant_guid: Guid,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) {
        self.sample_rejected_status
            .increment(instance_handle, rejected_reason);

        listener_sender
            .try_send(ListenerTriggerKind::OnSampleRejected(DataReaderNode::new(
                self.guid(),
                parent_subscriber_guid,
                parent_participant_guid,
            )))
            .ok();
    }

    fn on_requested_incompatible_qos(
        &mut self,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
        parent_subscriber_guid: Guid,
        parent_participant_guid: Guid,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) {
        self.requested_incompatible_qos_status
            .increment(incompatible_qos_policy_list);

        listener_sender
            .try_send(ListenerTriggerKind::RequestedIncompatibleQos(
                DataReaderNode::new(self.guid(), parent_subscriber_guid, parent_participant_guid),
            ))
            .ok();
    }
}

impl DdsDataReader<RtpsStatefulReader> {
    pub fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy) {
        self.rtps_reader.matched_writer_add(a_writer_proxy)
    }

    pub fn _matched_writer_remove(&mut self, a_writer_guid: Guid) {
        self.rtps_reader.matched_writer_remove(a_writer_guid)
    }

    pub fn guid(&self) -> Guid {
        self.rtps_reader.guid()
    }
}

impl DdsDataReader<RtpsStatelessReader> {
    pub fn guid(&self) -> Guid {
        self.rtps_reader.guid()
    }

    pub fn _convert_data_to_cache_change(
        &self,
        data_submessage: &DataSubmessageRead,
        source_timestamp: Option<Time>,
        source_guid_prefix: GuidPrefix,
        reception_timestamp: Time,
    ) -> RtpsReaderResult<RtpsReaderCacheChange> {
        self.rtps_reader._convert_data_to_cache_change(
            data_submessage,
            source_timestamp,
            source_guid_prefix,
            reception_timestamp,
        )
    }

    pub fn _add_change(
        &mut self,
        change: RtpsReaderCacheChange,
    ) -> RtpsReaderResult<InstanceHandle> {
        self.rtps_reader._add_change(change)
    }

    pub fn get_qos(&self) -> DataReaderQos {
        self.rtps_reader.get_qos().clone()
    }

    pub fn _set_qos(&mut self, qos: DataReaderQos) -> DdsResult<()> {
        self.rtps_reader._set_qos(qos)
    }

    pub fn read<Foo>(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.rtps_reader.read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn _take<Foo>(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.rtps_reader._take(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn read_next_instance<Foo>(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.rtps_reader.read_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn _take_next_instance<Foo>(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.rtps_reader._take_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessageRead<'_>,
        message_receiver: &MessageReceiver,
    ) -> StatelessReaderDataReceivedResult {
        self.rtps_reader
            .on_data_submessage_received(data_submessage, message_receiver)
    }

    pub fn _get_instance_handle(&self) -> InstanceHandle {
        self.rtps_reader.guid().into()
    }
}
