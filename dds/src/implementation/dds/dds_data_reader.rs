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
            reader::{convert_data_frag_to_cache_change, RtpsReader, RtpsReaderError},
            types::{Guid, GuidPrefix, Locator, SequenceNumber, ENTITYID_UNKNOWN, GUID_UNKNOWN},
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
            DurabilityQosPolicyKind, QosPolicyId, ReliabilityQosPolicyKind, DEADLINE_QOS_POLICY_ID,
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

pub enum DataReceivedResult {
    NoMatchedWriterProxy,
    UnexpectedDataSequenceNumber,
    NewSampleAdded(InstanceHandle),
    NewSampleAddedAndSamplesLost(InstanceHandle),
    SampleRejected(InstanceHandle, SampleRejectedStatusKind),
    InvalidData(&'static str),
    NotForThisReader,
}

impl From<RtpsReaderError> for DataReceivedResult {
    fn from(e: RtpsReaderError) -> Self {
        match e {
            RtpsReaderError::InvalidData(s) => DataReceivedResult::InvalidData(s),
            RtpsReaderError::Rejected(instance_handle, reason) => {
                DataReceivedResult::SampleRejected(instance_handle, reason)
            }
        }
    }
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

pub struct DdsDataReader {
    rtps_reader: RtpsReader,
    matched_writers: Vec<RtpsWriterProxy>,
    type_name: String,
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

impl DdsDataReader {
    pub fn new(
        rtps_reader: RtpsReader,
        type_name: String,
        topic_name: String,
        listener: Option<Actor<DdsDataReaderListener>>,
        status_kind: Vec<StatusKind>,
    ) -> Self {
        DdsDataReader {
            rtps_reader,
            matched_writers: Vec::new(),
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

    pub fn get_type_name(&self) -> String {
        self.type_name.clone()
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

    pub fn process_rtps_message(
        &mut self,
        message: RtpsMessageRead,
        reception_timestamp: Time,
        data_reader_address: ActorAddress<DdsDataReader>,
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
        data_reader_address: &ActorAddress<DdsDataReader>,
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
        data_reader_address: &ActorAddress<DdsDataReader>,
        subscriber_address: &ActorAddress<DdsSubscriber>,
        participant_address: &ActorAddress<DdsDomainParticipant>,
    ) -> UserDefinedReaderDataSubmessageReceivedResult {
        let sequence_number = data_submessage.writer_sn();
        let writer_guid = Guid::new(source_guid_prefix, data_submessage.writer_id());

        let data_submessage_received_result = if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|wp| wp.remote_writer_guid() == writer_guid)
        {
            let expected_seq_num = writer_proxy.available_changes_max() + 1;
            match self.rtps_reader.get_qos().reliability.kind {
                ReliabilityQosPolicyKind::BestEffort => {
                    if sequence_number >= expected_seq_num {
                        let change_result = self.rtps_reader.convert_data_to_cache_change(
                            data_submessage,
                            source_timestamp,
                            source_guid_prefix,
                            reception_timestamp,
                        );
                        match change_result {
                            Ok(change) => {
                                let add_change_result = self.rtps_reader.add_change(change);

                                match add_change_result {
                                    Ok(instance_handle) => {
                                        writer_proxy.received_change_set(sequence_number);
                                        if sequence_number > expected_seq_num {
                                            writer_proxy.lost_changes_update(sequence_number);
                                            DataReceivedResult::NewSampleAddedAndSamplesLost(
                                                instance_handle,
                                            )
                                        } else {
                                            DataReceivedResult::NewSampleAdded(instance_handle)
                                        }
                                    }
                                    Err(err) => err.into(),
                                }
                            }
                            Err(_) => DataReceivedResult::InvalidData("Invalid data submessage"),
                        }
                    } else {
                        DataReceivedResult::UnexpectedDataSequenceNumber
                    }
                }
                ReliabilityQosPolicyKind::Reliable => {
                    if sequence_number == expected_seq_num {
                        let change_result = self.rtps_reader.convert_data_to_cache_change(
                            data_submessage,
                            source_timestamp,
                            source_guid_prefix,
                            reception_timestamp,
                        );
                        match change_result {
                            Ok(change) => {
                                let add_change_result = self.rtps_reader.add_change(change);

                                match add_change_result {
                                    Ok(instance_handle) => {
                                        writer_proxy
                                            .received_change_set(data_submessage.writer_sn());
                                        DataReceivedResult::NewSampleAdded(instance_handle)
                                    }
                                    Err(err) => err.into(),
                                }
                            }
                            Err(_) => DataReceivedResult::InvalidData("Invalid data submessage"),
                        }
                    } else {
                        DataReceivedResult::UnexpectedDataSequenceNumber
                    }
                }
            }
        } else if data_submessage.reader_id() == ENTITYID_UNKNOWN {
            let change_result = self.rtps_reader.convert_data_to_cache_change(
                data_submessage,
                source_timestamp,
                source_guid_prefix,
                reception_timestamp,
            );
            match change_result {
                Ok(change) => {
                    let add_change_result = self.rtps_reader.add_change(change);

                    match add_change_result {
                        Ok(h) => DataReceivedResult::NewSampleAdded(h),
                        Err(e) => match e {
                            RtpsReaderError::InvalidData(s) => DataReceivedResult::InvalidData(s),
                            RtpsReaderError::Rejected(h, k) => {
                                DataReceivedResult::SampleRejected(h, k)
                            }
                        },
                    }
                }
                Err(_) => DataReceivedResult::InvalidData("Invalid data submessage"), // Change is ignored,
            }
        } else {
            DataReceivedResult::NotForThisReader
        };

        match data_submessage_received_result {
            DataReceivedResult::NoMatchedWriterProxy => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            DataReceivedResult::UnexpectedDataSequenceNumber => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            DataReceivedResult::NewSampleAdded(instance_handle) => {
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
            DataReceivedResult::NewSampleAddedAndSamplesLost(instance_handle) => {
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
            DataReceivedResult::SampleRejected(instance_handle, rejected_reason) => {
                self.on_sample_rejected(
                    instance_handle,
                    rejected_reason,
                    data_reader_address,
                    subscriber_address,
                    participant_address,
                );
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            DataReceivedResult::InvalidData(_) => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            DataReceivedResult::NotForThisReader => {
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
        data_reader_address: &ActorAddress<DdsDataReader>,
        subscriber_address: &ActorAddress<DdsSubscriber>,
        participant_address: &ActorAddress<DdsDomainParticipant>,
    ) -> UserDefinedReaderDataSubmessageReceivedResult {
        let sequence_number = data_frag_submessage.writer_sn();
        let writer_guid = Guid::new(source_guid_prefix, data_frag_submessage.writer_id());

        let data_submessage_received_result = if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|wp| wp.remote_writer_guid() == writer_guid)
        {
            let expected_seq_num = writer_proxy.available_changes_max() + 1;
            match self.rtps_reader.get_qos().reliability.kind {
                ReliabilityQosPolicyKind::BestEffort => {
                    if sequence_number >= expected_seq_num {
                        writer_proxy.push_data_frag(data_frag_submessage);
                        if let Some(data) = writer_proxy.extract_frag(sequence_number) {
                            let change_results = convert_data_frag_to_cache_change(
                                data_frag_submessage,
                                data,
                                source_timestamp,
                                source_guid_prefix,
                                reception_timestamp,
                            );
                            match change_results {
                                Ok(change) => {
                                    let add_change_result = self.rtps_reader.add_change(change);
                                    match add_change_result {
                                        Ok(instance_handle) => {
                                            writer_proxy.received_change_set(sequence_number);
                                            if sequence_number > expected_seq_num {
                                                writer_proxy.lost_changes_update(sequence_number);
                                                DataReceivedResult::NewSampleAddedAndSamplesLost(
                                                    instance_handle,
                                                )
                                            } else {
                                                DataReceivedResult::NewSampleAdded(instance_handle)
                                            }
                                        }
                                        Err(err) => err.into(),
                                    }
                                }
                                Err(_) => {
                                    DataReceivedResult::InvalidData("Invalid data submessage")
                                }
                            }
                        } else {
                            DataReceivedResult::NoMatchedWriterProxy
                        }
                    } else {
                        DataReceivedResult::UnexpectedDataSequenceNumber
                    }
                }
                ReliabilityQosPolicyKind::Reliable => {
                    if sequence_number == expected_seq_num {
                        writer_proxy.push_data_frag(data_frag_submessage);
                        if let Some(data) = writer_proxy.extract_frag(sequence_number) {
                            let change_result = convert_data_frag_to_cache_change(
                                data_frag_submessage,
                                data,
                                source_timestamp,
                                source_guid_prefix,
                                reception_timestamp,
                            );

                            match change_result {
                                Ok(change) => {
                                    let add_change_result = self.rtps_reader.add_change(change);
                                    match add_change_result {
                                        Ok(instance_handle) => {
                                            writer_proxy.received_change_set(sequence_number);
                                            DataReceivedResult::NewSampleAdded(instance_handle)
                                        }
                                        Err(err) => err.into(),
                                    }
                                }
                                Err(_) => {
                                    DataReceivedResult::InvalidData("Invalid data submessage")
                                }
                            }
                        } else {
                            DataReceivedResult::NoMatchedWriterProxy
                        }
                    } else {
                        DataReceivedResult::UnexpectedDataSequenceNumber
                    }
                }
            }
        } else {
            DataReceivedResult::NoMatchedWriterProxy
        };

        match data_submessage_received_result {
            DataReceivedResult::NoMatchedWriterProxy => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            DataReceivedResult::UnexpectedDataSequenceNumber => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            DataReceivedResult::NewSampleAdded(instance_handle) => {
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
            DataReceivedResult::NewSampleAddedAndSamplesLost(instance_handle) => {
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
            DataReceivedResult::SampleRejected(instance_handle, rejected_reason) => {
                self.on_sample_rejected(
                    instance_handle,
                    rejected_reason,
                    data_reader_address,
                    subscriber_address,
                    participant_address,
                );
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
            DataReceivedResult::InvalidData(_) | DataReceivedResult::NotForThisReader => {
                UserDefinedReaderDataSubmessageReceivedResult::NoChange
            }
        }
    }

    pub fn on_heartbeat_submessage_received(
        &mut self,
        heartbeat_submessage: &HeartbeatSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.rtps_reader.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let writer_guid = Guid::new(source_guid_prefix, heartbeat_submessage.writer_id());

            if let Some(writer_proxy) = self
                .matched_writers
                .iter_mut()
                .find(|x| x.remote_writer_guid() == writer_guid)
            {
                if writer_proxy.last_received_heartbeat_count() < heartbeat_submessage.count() {
                    writer_proxy.set_last_received_heartbeat_count(heartbeat_submessage.count());

                    writer_proxy.set_must_send_acknacks(
                        !heartbeat_submessage.final_flag()
                            || (!heartbeat_submessage.liveliness_flag()
                                && !writer_proxy.missing_changes().is_empty()),
                    );

                    if !heartbeat_submessage.final_flag() {
                        writer_proxy.set_must_send_acknacks(true);
                    }
                    writer_proxy.missing_changes_update(heartbeat_submessage.last_sn());
                    writer_proxy.lost_changes_update(heartbeat_submessage.first_sn());
                }
            }
        }
    }

    pub fn on_heartbeat_frag_submessage_received(
        &mut self,
        heartbeat_frag_submessage: &HeartbeatFragSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.rtps_reader.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let writer_guid = Guid::new(source_guid_prefix, heartbeat_frag_submessage.writer_id());

            if let Some(writer_proxy) = self
                .matched_writers
                .iter_mut()
                .find(|x| x.remote_writer_guid() == writer_guid)
            {
                if writer_proxy.last_received_heartbeat_count() < heartbeat_frag_submessage.count()
                {
                    writer_proxy
                        .set_last_received_heartbeat_frag_count(heartbeat_frag_submessage.count());
                }

                // todo!()
            }
        }
    }

    pub fn add_matched_writer(
        &mut self,
        discovered_writer_data: DiscoveredWriterData,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        data_reader_address: ActorAddress<DdsDataReader>,
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
                    &subscriber_address.get_qos().unwrap(),
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

                self.matched_writer_add(writer_proxy);
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
        data_reader_address: ActorAddress<DdsDataReader>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        let matched_publication = self
            .matched_publication_list
            .remove(&discovered_writer_handle);
        if let Some(w) = matched_publication {
            self.matched_writer_remove(w.key().value.into());

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

        Ok(!self
            .matched_writers
            .iter()
            .any(|p| !p.is_historical_data_received()))
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
        for writer_proxy in self.matched_writers.iter_mut() {
            writer_proxy.send_message(&self.rtps_reader.guid(), header, &udp_transport_write)
        }
    }

    pub fn update_communication_status(
        &mut self,
        now: Time,
        data_reader_address: ActorAddress<DdsDataReader>,
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
        let writer_guid = Guid::new(source_guid_prefix, gap_submessage.writer_id());
        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == writer_guid)
        {
            for seq_num in
                i64::from(gap_submessage.gap_start())..i64::from(gap_submessage.gap_list().base())
            {
                writer_proxy.irrelevant_change_set(SequenceNumber::from(seq_num))
            }

            for seq_num in gap_submessage.gap_list().set() {
                writer_proxy.irrelevant_change_set(*seq_num)
            }
        }
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
        data_reader_address: ActorAddress<DdsDataReader>,
        subscriber_address: ActorAddress<DdsSubscriber>,
        participant_address: ActorAddress<DdsDomainParticipant>,
    ) {
        self.subscription_matched_status.increment(instance_handle);
        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::SubscriptionMatched);
        if self.listener.is_some() && self.status_kind.contains(&StatusKind::SubscriptionMatched) {
            let listener_address = self.listener.as_ref().unwrap().address().clone();
            let reader =
                DataReaderNode::new(data_reader_address, subscriber_address, participant_address);
            let status = self.get_subscription_matched_status();
            listener_address
                .trigger_on_subscription_matched(reader, status)
                .expect("Should not fail to send message");
        } else if subscriber_address.get_listener().unwrap().is_some()
            && subscriber_address
                .status_kind()
                .unwrap()
                .contains(&StatusKind::SubscriptionMatched)
        {
            let listener_address = subscriber_address.get_listener().unwrap().unwrap();
            let reader =
                DataReaderNode::new(data_reader_address, subscriber_address, participant_address);
            let status = self.get_subscription_matched_status();
            listener_address
                .trigger_on_subscription_matched(reader, status)
                .expect("Should not fail to send message");
        } else if participant_address.get_listener().unwrap().is_some()
            && participant_address
                .status_kind()
                .unwrap()
                .contains(&StatusKind::SubscriptionMatched)
        {
            let listener_address = participant_address.get_listener().unwrap().unwrap();
            let reader =
                DataReaderNode::new(data_reader_address, subscriber_address, participant_address);
            let status = self.get_subscription_matched_status();
            listener_address
                .trigger_on_subscription_matched(reader, status)
                .expect("Should not fail to send message");
        }
    }

    fn on_sample_rejected(
        &mut self,
        instance_handle: InstanceHandle,
        rejected_reason: SampleRejectedStatusKind,
        data_reader_address: &ActorAddress<DdsDataReader>,
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
            let listener_address = self.listener.as_ref().unwrap().address().clone();
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
        data_reader_address: &ActorAddress<DdsDataReader>,
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

    pub fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy) {
        if !self
            .matched_writers
            .iter()
            .any(|x| x.remote_writer_guid() == a_writer_proxy.remote_writer_guid())
        {
            self.matched_writers.push(a_writer_proxy);
        }
    }

    pub fn matched_writer_remove(&mut self, a_writer_guid: Guid) {
        self.matched_writers
            .retain(|x| x.remote_writer_guid() != a_writer_guid)
    }

    pub fn guid(&self) -> Guid {
        self.rtps_reader.guid()
    }
}
