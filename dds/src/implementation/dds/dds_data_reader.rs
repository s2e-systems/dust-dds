use std::collections::{HashMap, HashSet};

use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_writer_data::DiscoveredWriterData,
        },
        dds::nodes::DataReaderNode,
        rtps::{
            messages::{
                overall_structure::{RtpsMessageHeader, RtpsMessageRead, RtpsSubmessageReadKind},
                submessages::{
                    data::DataSubmessageRead, data_frag::DataFragSubmessageRead,
                    gap::GapSubmessageRead, heartbeat::HeartbeatSubmessageRead,
                    heartbeat_frag::HeartbeatFragSubmessageRead,
                },
            },
            reader::{RtpsReaderCacheChange, RtpsReaderResult},
            stateful_reader::{RtpsStatefulReader, StatefulReaderDataReceivedResult},
            stateless_reader::{RtpsStatelessReader, StatelessReaderDataReceivedResult},
            types::{Guid, GuidPrefix, Locator, GUID_UNKNOWN},
            writer_proxy::RtpsWriterProxy,
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::{
            actor::{Actor, ActorAddress},
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos, TopicQos},
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
        time::{DurationKind, Time},
    },
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    dds_data_reader_listener::DdsDataReaderListener, dds_domain_participant::DdsDomainParticipant,
    dds_subscriber::DdsSubscriber, message_receiver::MessageReceiver, nodes::SubscriberNode,
    status_condition_impl::StatusConditionImpl,
};

pub enum UserDefinedReaderDataSubmessageReceivedResult {
    NoChange,
    NewDataAvailable,
}

impl SampleLostStatus {
    fn _increment(&mut self) {
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
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    listener: Option<Actor<DdsDataReaderListener>>,
    status_kind: Vec<StatusKind>,
}

impl<T> DdsDataReader<T> {
    pub fn new(
        rtps_reader: T,
        type_name: &'static str,
        topic_name: String,
        listener: Option<Actor<DdsDataReaderListener>>,
        status_kind: Vec<StatusKind>,
    ) -> Self {
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
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            status_kind,
            listener,
        }
    }

    pub fn get_type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn get_topic_name(&self) -> String {
        self.topic_name.clone()
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
        self.status_condition
            .write_lock()
            .remove_communication_state(StatusKind::SubscriptionMatched);
        self.subscription_matched_status
            .read_and_reset(self.matched_publication_list.len() as i32)
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_matched_publications(&self) -> Vec<InstanceHandle> {
        self.matched_publication_list
            .iter()
            .map(|(&key, _)| key)
            .collect()
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
}

impl DdsDataReader<RtpsStatefulReader> {
    pub fn process_rtps_message(
        &mut self,
        message: RtpsMessageRead,
        reception_timestamp: Time,
        data_reader_address: ActorAddress<DdsDataReader<RtpsStatefulReader>>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        let mut message_receiver = MessageReceiver::new(&message);
        while let Some(submessage) = message_receiver.next() {
            match submessage {
                RtpsSubmessageReadKind::Data(data_submessage) => {
                    self.on_data_submessage_received(
                        &data_submessage,
                        message_receiver.source_guid_prefix(),
                        message_receiver.source_timestamp(),
                        reception_timestamp,
                        &data_reader_address,
                        &subscriber_address,
                        &participant_address,
                    );
                }
                RtpsSubmessageReadKind::DataFrag(data_frag_submessage) => {
                    self.on_data_frag_submessage_received(
                        &data_frag_submessage,
                        message_receiver.source_guid_prefix(),
                        message_receiver.source_timestamp(),
                        reception_timestamp,
                        &data_reader_address,
                        &subscriber_address,
                        &participant_address,
                    );
                }
                RtpsSubmessageReadKind::Gap(gap_submessage) => {
                    self.on_gap_submessage_received(
                        &gap_submessage,
                        message_receiver.source_guid_prefix(),
                    );
                }
                RtpsSubmessageReadKind::Heartbeat(heartbeat_submessage) => self
                    .on_heartbeat_submessage_received(
                        &heartbeat_submessage,
                        message_receiver.source_guid_prefix(),
                    ),
                RtpsSubmessageReadKind::HeartbeatFrag(heartbeat_frag_submessage) => self
                    .on_heartbeat_frag_submessage_received(
                        &heartbeat_frag_submessage,
                        message_receiver.source_guid_prefix(),
                    ),
                _ => (),
            }
        }
    }

    pub fn on_data_available(
        &self,
        data_reader_address: &ActorAddress<DdsDataReader<RtpsStatefulReader>>,
        subscriber_address: &ActorAddress<DdsSubscriber>,
        participant_address: &ActorAddress<DdsDomainParticipant>,
    ) {
        subscriber_address
            .get_statuscondition()
            .unwrap()
            .write_lock()
            .add_communication_state(StatusKind::DataOnReaders);
        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::DataAvailable);
        if subscriber_address.get_listener().unwrap().is_some()
            && subscriber_address
                .status_kind()
                .unwrap()
                .contains(&StatusKind::DataOnReaders)
        {
            let listener = subscriber_address
                .get_listener()
                .unwrap()
                .expect("Already checked for some");
            listener
                .trigger_on_data_on_readers(SubscriberNode::new(
                    subscriber_address.clone(),
                    participant_address.clone(),
                ))
                .expect("Should not fail to send message");
        } else if self.listener.is_some() && self.status_kind.contains(&StatusKind::DataAvailable) {
            let listener_address = self.listener.as_ref().unwrap().address();
            let reader = DataReaderNode::new(
                data_reader_address.clone(),
                subscriber_address.clone(),
                participant_address.clone(),
            );
            listener_address
                .trigger_on_data_available(reader)
                .expect("Should not fail to send message");
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessageRead<'_>,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<Time>,
        reception_timestamp: Time,
        data_reader_address: &ActorAddress<DdsDataReader<RtpsStatefulReader>>,
        subscriber_address: &ActorAddress<DdsSubscriber>,
        participant_address: &ActorAddress<DdsDomainParticipant>,
    ) -> UserDefinedReaderDataSubmessageReceivedResult {
        let data_submessage_received_result = self.rtps_reader.on_data_submessage_received(
            data_submessage,
            source_guid_prefix,
            source_timestamp,
            reception_timestamp,
        );
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
                    .insert(instance_handle, reception_timestamp);
                self.on_data_available(
                    data_reader_address,
                    subscriber_address,
                    participant_address,
                );
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::NewSampleAddedAndSamplesLost(instance_handle) => {
                self.instance_reception_time
                    .insert(instance_handle, reception_timestamp);
                self.data_available_status_changed_flag = true;
                // self.on_sample_lost(
                //     parent_subscriber_guid,
                //     parent_participant_guid,
                //     listener_sender,
                // );
                self.on_data_available(
                    data_reader_address,
                    subscriber_address,
                    participant_address,
                );
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::SampleRejected(instance_handle, rejected_reason) => {
                self.on_sample_rejected(
                    instance_handle,
                    rejected_reason,
                    data_reader_address,
                    subscriber_address,
                    participant_address,
                );
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::InvalidData(_) => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_data_frag_submessage_received(
        &mut self,
        data_frag_submessage: &DataFragSubmessageRead<'_>,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<Time>,
        reception_timestamp: Time,
        data_reader_address: &ActorAddress<DdsDataReader<RtpsStatefulReader>>,
        subscriber_address: &ActorAddress<DdsSubscriber>,
        participant_address: &ActorAddress<DdsDomainParticipant>,
    ) -> UserDefinedReaderDataSubmessageReceivedResult {
        let data_submessage_received_result = self.rtps_reader.on_data_frag_submessage_received(
            data_frag_submessage,
            source_guid_prefix,
            source_timestamp,
            reception_timestamp,
        );

        match data_submessage_received_result {
            StatefulReaderDataReceivedResult::NoMatchedWriterProxy => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            StatefulReaderDataReceivedResult::NewSampleAdded(instance_handle) => {
                self.instance_reception_time
                    .insert(instance_handle, reception_timestamp);
                self.data_available_status_changed_flag = true;
                self.on_data_available(
                    data_reader_address,
                    subscriber_address,
                    participant_address,
                );
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::NewSampleAddedAndSamplesLost(instance_handle) => {
                self.instance_reception_time
                    .insert(instance_handle, reception_timestamp);
                self.data_available_status_changed_flag = true;
                // self.on_sample_lost(
                //     parent_subscriber_guid,
                //     parent_participant_guid,
                //     listener_sender,
                // );
                self.on_data_available(
                    data_reader_address,
                    subscriber_address,
                    participant_address,
                );
                UserDefinedReaderDataSubmessageReceivedResult::NewDataAvailable
            }
            StatefulReaderDataReceivedResult::SampleRejected(instance_handle, rejected_reason) => {
                self.on_sample_rejected(
                    instance_handle,
                    rejected_reason,
                    data_reader_address,
                    subscriber_address,
                    participant_address,
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
        heartbeat_frag_submessage: &HeartbeatFragSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        self.rtps_reader
            .on_heartbeat_frag_submessage_received(heartbeat_frag_submessage, source_guid_prefix);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_matched_writer(
        &mut self,
        discovered_writer_data: DiscoveredWriterData,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        subscriber_qos: SubscriberQos,
        data_reader_address: ActorAddress<DdsDataReader<RtpsStatefulReader>>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        let publication_builtin_topic_data = discovered_writer_data.dds_publication_data();
        if publication_builtin_topic_data.topic_name() == self.topic_name
            && publication_builtin_topic_data.get_type_name() == self.type_name
        {
            let instance_handle = discovered_writer_data.get_serialized_key().into();
            let incompatible_qos_policy_list = self
                .get_discovered_writer_incompatible_qos_policy_list(
                    &discovered_writer_data,
                    &subscriber_qos,
                );
            if incompatible_qos_policy_list.is_empty() {
                let unicast_locator_list = if discovered_writer_data
                    .writer_proxy()
                    .unicast_locator_list()
                    .is_empty()
                {
                    default_unicast_locator_list
                } else {
                    discovered_writer_data
                        .writer_proxy()
                        .unicast_locator_list()
                        .to_vec()
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
                        .to_vec()
                };

                let writer_proxy = RtpsWriterProxy::new(
                    discovered_writer_data.writer_proxy().remote_writer_guid(),
                    &unicast_locator_list,
                    &multicast_locator_list,
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
                            data_reader_address,
                            subscriber_address,
                            participant_address,
                        ),
                    None => self.on_subscription_matched(
                        instance_handle,
                        data_reader_address,
                        subscriber_address,
                        participant_address,
                    ),
                    _ => (),
                }
            } else if self.incompatible_writer_list.insert(instance_handle) {
                self.on_requested_incompatible_qos(
                    incompatible_qos_policy_list,
                    &data_reader_address,
                    &subscriber_address,
                    &participant_address,
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
        data_reader_address: ActorAddress<DdsDataReader<RtpsStatefulReader>>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        let matched_publication = self
            .matched_publication_list
            .remove(&discovered_writer_handle);
        if let Some(w) = matched_publication {
            self.rtps_reader.matched_writer_remove(w.key().value.into());

            self.on_subscription_matched(
                discovered_writer_handle,
                data_reader_address,
                subscriber_address,
                participant_address,
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

    pub fn set_qos(&mut self, qos: DataReaderQos) -> DdsResult<()> {
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
        topic_qos: TopicQos,
        subscriber_qos: SubscriberQos,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> DiscoveredReaderData {
        let guid = self.rtps_reader.guid();
        let reader_qos = self.rtps_reader.get_qos().clone();

        let unicast_locator_list = if self.rtps_reader.unicast_locator_list().is_empty() {
            default_unicast_locator_list
        } else {
            self.rtps_reader.unicast_locator_list().to_vec()
        };

        let multicast_locator_list = if self.rtps_reader.unicast_locator_list().is_empty() {
            default_multicast_locator_list
        } else {
            self.rtps_reader.multicast_locator_list().to_vec()
        };

        DiscoveredReaderData::new(
            ReaderProxy::new(
                guid,
                guid.entity_id(),
                unicast_locator_list,
                multicast_locator_list,
                false,
            ),
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
                topic_qos.topic_data,
                subscriber_qos.group_data,
            ),
        )
    }

    pub fn send_message(
        &mut self,
        header: RtpsMessageHeader,
        udp_transport_write: ActorAddress<UdpTransportWrite>,
    ) {
        self.rtps_reader.send_message(header, &udp_transport_write);
    }

    pub fn update_communication_status(
        &mut self,
        now: Time,
        data_reader_address: ActorAddress<DdsDataReader<RtpsStatefulReader>>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_address: ActorAddress<DdsDomainParticipant>,
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

            self.status_condition
                .write_lock()
                .add_communication_state(StatusKind::RequestedDeadlineMissed);
            if self.listener.is_some()
                && self
                    .status_kind
                    .contains(&StatusKind::RequestedDeadlineMissed)
            {
                let status = self.get_requested_deadline_missed_status();
                let listener_address = self.listener.as_ref().unwrap().address();
                let reader = DataReaderNode::new(
                    data_reader_address.clone(),
                    subscriber_address.clone(),
                    participant_address.clone(),
                );
                listener_address
                    .trigger_on_requested_deadline_missed(reader, status)
                    .expect("Should not fail to send message");
            } else if subscriber_address.get_listener().unwrap().is_some()
                && subscriber_address
                    .status_kind()
                    .unwrap()
                    .contains(&StatusKind::RequestedDeadlineMissed)
            {
                let status = self.get_requested_deadline_missed_status();
                let listener_address = subscriber_address.get_listener().unwrap().unwrap();
                let reader = DataReaderNode::new(
                    data_reader_address.clone(),
                    subscriber_address.clone(),
                    participant_address.clone(),
                );
                listener_address
                    .trigger_on_requested_deadline_missed(reader, status)
                    .expect("Should not fail to send message");
            } else if participant_address.get_listener().unwrap().is_some()
                && participant_address
                    .status_kind()
                    .unwrap()
                    .contains(&StatusKind::RequestedDeadlineMissed)
            {
                let status = self.get_requested_deadline_missed_status();
                let listener_address = participant_address.get_listener().unwrap().unwrap();
                let reader = DataReaderNode::new(
                    data_reader_address.clone(),
                    subscriber_address.clone(),
                    participant_address.clone(),
                );
                listener_address
                    .trigger_on_requested_deadline_missed(reader, status)
                    .expect("Should not fail to send message");
            }
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

    fn _on_sample_lost(&mut self, _parent_subscriber_guid: Guid, _parent_participant_guid: Guid) {
        todo!()
        // self.sample_lost_status.increment();
        // listener_sender
        //     .try_send(ListenerTriggerKind::OnSampleLost(DataReaderNode::new(
        //         self.guid(),
        //         parent_subscriber_guid,
        //         parent_participant_guid,
        //     )))
        //     .ok();
    }

    fn on_subscription_matched(
        &mut self,
        instance_handle: InstanceHandle,
        data_reader_address: ActorAddress<DdsDataReader<RtpsStatefulReader>>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        self.subscription_matched_status.increment(instance_handle);
        if self.listener.is_some() && self.status_kind.contains(&StatusKind::SubscriptionMatched) {
            let listener_address = self.listener.as_ref().unwrap().address();
            let reader =
                DataReaderNode::new(data_reader_address, subscriber_address, participant_address);
            let status = self.get_subscription_matched_status();
            listener_address
                .trigger_on_subscription_matched(reader, status)
                .expect("Should not fail to send message");
        }
        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::SubscriptionMatched);
    }

    fn on_sample_rejected(
        &mut self,
        instance_handle: InstanceHandle,
        rejected_reason: SampleRejectedStatusKind,
        data_reader_address: &ActorAddress<DdsDataReader<RtpsStatefulReader>>,
        subscriber_address: &ActorAddress<DdsSubscriber>,
        participant_address: &ActorAddress<DdsDomainParticipant>,
    ) {
        self.sample_rejected_status
            .increment(instance_handle, rejected_reason);
        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::SampleRejected);
        if self.listener.is_some() && self.status_kind.contains(&StatusKind::SampleRejected) {
            let status = self.get_sample_rejected_status();
            let listener_address = self.listener.as_ref().unwrap().address();
            let reader = DataReaderNode::new(
                data_reader_address.clone(),
                subscriber_address.clone(),
                participant_address.clone(),
            );
            listener_address
                .trigger_on_sample_rejected(reader, status)
                .expect("Should not fail to send message");
        } else if subscriber_address.get_listener().unwrap().is_some()
            && subscriber_address
                .status_kind()
                .unwrap()
                .contains(&StatusKind::SampleRejected)
        {
            let status = self.get_sample_rejected_status();
            let listener_address = subscriber_address.get_listener().unwrap().unwrap();
            let reader = DataReaderNode::new(
                data_reader_address.clone(),
                subscriber_address.clone(),
                participant_address.clone(),
            );
            listener_address
                .trigger_on_sample_rejected(reader, status)
                .expect("Should not fail to send message");
        } else if participant_address.get_listener().unwrap().is_some()
            && participant_address
                .status_kind()
                .unwrap()
                .contains(&StatusKind::SampleRejected)
        {
            let status = self.get_sample_rejected_status();
            let listener_address = participant_address.get_listener().unwrap().unwrap();
            let reader = DataReaderNode::new(
                data_reader_address.clone(),
                subscriber_address.clone(),
                participant_address.clone(),
            );
            listener_address
                .trigger_on_sample_rejected(reader, status)
                .expect("Should not fail to send message");
        }
    }

    fn on_requested_incompatible_qos(
        &mut self,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
        data_reader_address: &ActorAddress<DdsDataReader<RtpsStatefulReader>>,
        subscriber_address: &ActorAddress<DdsSubscriber>,
        participant_address: &ActorAddress<DdsDomainParticipant>,
    ) {
        self.requested_incompatible_qos_status
            .increment(incompatible_qos_policy_list);
        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::RequestedIncompatibleQos);

        if self.listener.is_some()
            && self
                .status_kind
                .contains(&StatusKind::RequestedIncompatibleQos)
        {
            let status = self.get_requested_incompatible_qos_status();
            let listener_address = self.listener.as_ref().unwrap().address();
            let reader = DataReaderNode::new(
                data_reader_address.clone(),
                subscriber_address.clone(),
                participant_address.clone(),
            );
            listener_address
                .trigger_on_requested_incompatible_qos(reader, status)
                .expect("Should not fail to send message");
        } else if subscriber_address.get_listener().unwrap().is_some()
            && subscriber_address
                .status_kind()
                .unwrap()
                .contains(&StatusKind::RequestedIncompatibleQos)
        {
            let status = self.get_requested_incompatible_qos_status();
            let listener_address = subscriber_address.get_listener().unwrap().unwrap();
            let reader = DataReaderNode::new(
                data_reader_address.clone(),
                subscriber_address.clone(),
                participant_address.clone(),
            );
            listener_address
                .trigger_on_requested_incompatible_qos(reader, status)
                .expect("Should not fail to send message");
        } else if participant_address.get_listener().unwrap().is_some()
            && participant_address
                .status_kind()
                .unwrap()
                .contains(&StatusKind::RequestedIncompatibleQos)
        {
            let status = self.get_requested_incompatible_qos_status();
            let listener_address = participant_address.get_listener().unwrap().unwrap();
            let reader = DataReaderNode::new(
                data_reader_address.clone(),
                subscriber_address.clone(),
                participant_address.clone(),
            );
            listener_address
                .trigger_on_requested_incompatible_qos(reader, status)
                .expect("Should not fail to send message");
        }
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
    pub fn process_rtps_message(&mut self, message: RtpsMessageRead, reception_timestamp: Time) {
        let mut message_receiver = MessageReceiver::new(&message);
        while let Some(submessage) = message_receiver.next() {
            match &submessage {
                RtpsSubmessageReadKind::Data(data_submessage) => {
                    self.on_data_submessage_received(
                        data_submessage,
                        message_receiver.source_timestamp(),
                        message_receiver.source_guid_prefix(),
                        reception_timestamp,
                    );
                }
                RtpsSubmessageReadKind::DataFrag(_) => todo!(),
                RtpsSubmessageReadKind::Gap(_) => todo!(),
                RtpsSubmessageReadKind::Heartbeat(_) => todo!(),
                RtpsSubmessageReadKind::HeartbeatFrag(_) => todo!(),
                _ => (),
            }
        }
    }

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
        source_timestamp: Option<Time>,
        source_guid_prefix: GuidPrefix,
        reception_timestamp: Time,
    ) -> StatelessReaderDataReceivedResult {
        self.rtps_reader.on_data_submessage_received(
            data_submessage,
            source_timestamp,
            source_guid_prefix,
            reception_timestamp,
        )
    }

    pub fn _get_instance_handle(&self) -> InstanceHandle {
        self.rtps_reader.guid().into()
    }
}
