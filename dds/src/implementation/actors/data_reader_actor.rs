use super::{
    any_data_reader_listener::{AnyDataReaderListener, DataReaderListenerOperation},
    domain_participant_actor::{ParticipantListenerMessage, ParticipantListenerOperation},
    message_sender_actor::MessageSenderActor,
    status_condition_actor::{self, AddCommunicationState, StatusConditionActor},
    subscriber_actor::{SubscriberListenerMessage, SubscriberListenerOperation},
    topic_actor::TopicActor,
};
use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    data_representation_builtin_endpoints::{
        discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
        discovered_writer_data::DiscoveredWriterData,
    },
    dds_async::{subscriber::SubscriberAsync, topic::TopicAsync},
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        data_representation_inline_qos::{
            parameter_id_values::{PID_KEY_HASH, PID_STATUS_INFO},
            types::{
                StatusInfo, STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED,
                STATUS_INFO_UNREGISTERED,
            },
        },
        payload_serializer_deserializer::{
            cdr_deserializer::ClassicCdrDeserializer, endianness::CdrEndianness,
        },
        runtime::{
            executor::{block_on, ExecutorHandle, TaskHandle},
            mpsc::{mpsc_channel, MpscSender},
            timer::TimerHandle,
        },
    },
    infrastructure::{
        self,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos},
        qos_policy::{
            DestinationOrderQosPolicyKind, DurabilityQosPolicyKind, HistoryQosPolicyKind,
            QosPolicyId, ReliabilityQosPolicyKind, TopicDataQosPolicy, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID,
        },
        status::{
            LivelinessChangedStatus, QosPolicyCount, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
            SampleRejectedStatusKind, StatusKind, SubscriptionMatchedStatus,
        },
        time::DurationKind,
    },
    rtps::{
        self,
        cache_change::RtpsCacheChange,
        messages::{
            submessage_elements::{Data, Parameter, ParameterList},
            submessages::{
                data::DataSubmessage, data_frag::DataFragSubmessage, gap::GapSubmessage,
                heartbeat::HeartbeatSubmessage, heartbeat_frag::HeartbeatFragSubmessage,
            },
        },
        reader::RtpsReaderKind,
        types::{ChangeKind, Guid, GuidPrefix, Locator, ENTITYID_UNKNOWN, GUID_UNKNOWN},
        writer_proxy::RtpsWriterProxy,
    },
    serialized_payload::cdr::deserialize::CdrDeserialize,
    subscription::sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
    topic_definition::type_support::{DdsKey, DynamicTypeInterface},
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    thread::JoinHandle,
};
use tracing::debug;

struct InstanceState {
    view_state: ViewStateKind,
    instance_state: InstanceStateKind,
    most_recent_disposed_generation_count: i32,
    most_recent_no_writers_generation_count: i32,
}

impl InstanceState {
    pub fn new() -> Self {
        Self {
            view_state: ViewStateKind::New,
            instance_state: InstanceStateKind::Alive,
            most_recent_disposed_generation_count: 0,
            most_recent_no_writers_generation_count: 0,
        }
    }

    pub fn update_state(&mut self, change_kind: ChangeKind) {
        match self.instance_state {
            InstanceStateKind::Alive => {
                if change_kind == ChangeKind::NotAliveDisposed
                    || change_kind == ChangeKind::NotAliveDisposedUnregistered
                {
                    self.instance_state = InstanceStateKind::NotAliveDisposed;
                } else if change_kind == ChangeKind::NotAliveUnregistered {
                    self.instance_state = InstanceStateKind::NotAliveNoWriters;
                }
            }
            InstanceStateKind::NotAliveDisposed => {
                if change_kind == ChangeKind::Alive {
                    self.instance_state = InstanceStateKind::Alive;
                    self.most_recent_disposed_generation_count += 1;
                }
            }
            InstanceStateKind::NotAliveNoWriters => {
                if change_kind == ChangeKind::Alive {
                    self.instance_state = InstanceStateKind::Alive;
                    self.most_recent_no_writers_generation_count += 1;
                }
            }
        }

        match self.view_state {
            ViewStateKind::New => (),
            ViewStateKind::NotNew => {
                if change_kind == ChangeKind::NotAliveDisposed
                    || change_kind == ChangeKind::NotAliveUnregistered
                {
                    self.view_state = ViewStateKind::New;
                }
            }
        }
    }

    pub fn mark_viewed(&mut self) {
        self.view_state = ViewStateKind::NotNew;
    }
}

#[derive(Debug)]
struct ReaderCacheChange {
    rtps_cache_change: RtpsCacheChange,
    sample_state: SampleStateKind,
    disposed_generation_count: i32,
    no_writers_generation_count: i32,
    reception_timestamp: rtps::messages::types::Time,
    source_timestamp: Option<rtps::messages::types::Time>,
}

impl ReaderCacheChange {
    fn instance_handle(&self) -> InstanceHandle {
        self.rtps_cache_change.instance_handle.into()
    }
}

fn build_instance_handle(
    type_support: &Arc<dyn DynamicTypeInterface + Send + Sync>,
    change_kind: ChangeKind,
    data: &[u8],
    inline_qos: &[Parameter],
) -> DdsResult<InstanceHandle> {
    Ok(match change_kind {
        ChangeKind::Alive | ChangeKind::AliveFiltered => {
            type_support.instance_handle_from_serialized_foo(data)?
        }
        ChangeKind::NotAliveDisposed
        | ChangeKind::NotAliveUnregistered
        | ChangeKind::NotAliveDisposedUnregistered => match inline_qos
            .iter()
            .find(|&x| x.parameter_id() == PID_KEY_HASH)
        {
            Some(p) => {
                if let Ok(key) = <[u8; 16]>::try_from(p.value()) {
                    InstanceHandle::new(key)
                } else {
                    type_support.instance_handle_from_serialized_key(data)?
                }
            }
            None => type_support.instance_handle_from_serialized_key(data)?,
        },
    })
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

#[derive(Default)]
struct ReaderRequestedDeadlineMissedStatus {
    total_count: i32,
    total_count_change: i32,
    last_instance_handle: InstanceHandle,
}

pub struct IncrementRequestedDeadlineMissedStatus {
    instance_handle: InstanceHandle,
}
impl Mail for IncrementRequestedDeadlineMissedStatus {
    type Result = ();
}
impl MailHandler<IncrementRequestedDeadlineMissedStatus> for DataReaderActor {
    fn handle(
        &mut self,
        message: IncrementRequestedDeadlineMissedStatus,
    ) -> <IncrementRequestedDeadlineMissedStatus as Mail>::Result {
        self.requested_deadline_missed_status.total_count += 1;
        self.requested_deadline_missed_status.total_count_change += 1;
        self.requested_deadline_missed_status.last_instance_handle = message.instance_handle;
    }
}

pub struct ReadRequestedDeadlineMissedStatus;
impl Mail for ReadRequestedDeadlineMissedStatus {
    type Result = RequestedDeadlineMissedStatus;
}
impl MailHandler<ReadRequestedDeadlineMissedStatus> for DataReaderActor {
    fn handle(
        &mut self,
        _: ReadRequestedDeadlineMissedStatus,
    ) -> <ReadRequestedDeadlineMissedStatus as Mail>::Result {
        self.read_requested_deadline_missed_status()
    }
}

impl LivelinessChangedStatus {
    fn _read_and_reset(&mut self) -> Self {
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

struct IndexedSample {
    index: usize,
    sample: (Option<Data>, SampleInfo),
}

struct DataReaderListenerMessage {
    listener_operation: DataReaderListenerOperation,
    reader_address: ActorAddress<DataReaderActor>,
    status_condition_address: ActorAddress<StatusConditionActor>,
    subscriber: SubscriberAsync,
    topic: TopicAsync,
}

struct DataReaderListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<DataReaderListenerMessage>,
}

impl DataReaderListenerThread {
    fn new(mut listener: Box<dyn AnyDataReaderListener + Send>) -> Self {
        let (sender, receiver) = mpsc_channel::<DataReaderListenerMessage>();
        let thread = std::thread::spawn(move || {
            block_on(async {
                while let Some(m) = receiver.recv().await {
                    listener
                        .call_listener_function(
                            m.listener_operation,
                            m.reader_address,
                            m.status_condition_address,
                            m.subscriber,
                            m.topic,
                        )
                        .await;
                }
            });
        });
        Self { thread, sender }
    }

    fn sender(&self) -> &MpscSender<DataReaderListenerMessage> {
        &self.sender
    }

    fn join(self) -> DdsResult<()> {
        self.sender.close();
        self.thread.join()?;
        Ok(())
    }
}

pub struct DataReaderActor {
    rtps_reader: RtpsReaderKind,
    changes: Vec<ReaderCacheChange>,
    qos: DataReaderQos,
    topic_address: ActorAddress<TopicActor>,
    topic_name: String,
    type_name: String,
    topic_status_condition: ActorAddress<StatusConditionActor>,
    type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
    _liveliness_changed_status: LivelinessChangedStatus,
    requested_deadline_missed_status: ReaderRequestedDeadlineMissedStatus,
    requested_incompatible_qos_status: RequestedIncompatibleQosStatus,
    sample_lost_status: SampleLostStatus,
    sample_rejected_status: SampleRejectedStatus,
    subscription_matched_status: SubscriptionMatchedStatus,
    matched_publication_list: HashMap<InstanceHandle, PublicationBuiltinTopicData>,
    enabled: bool,
    data_available_status_changed_flag: bool,
    incompatible_writer_list: HashSet<InstanceHandle>,
    status_condition: Actor<StatusConditionActor>,
    data_reader_listener_thread: Option<DataReaderListenerThread>,
    status_kind: Vec<StatusKind>,
    instances: HashMap<InstanceHandle, InstanceState>,
    instance_deadline_missed_task: HashMap<InstanceHandle, TaskHandle>,
}

impl DataReaderActor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rtps_reader: RtpsReaderKind,
        topic_address: ActorAddress<TopicActor>,
        topic_name: String,
        type_name: String,
        topic_status_condition: ActorAddress<StatusConditionActor>,
        type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
        qos: DataReaderQos,
        listener: Option<Box<dyn AnyDataReaderListener + Send>>,
        status_kind: Vec<StatusKind>,
        handle: &ExecutorHandle,
    ) -> Self {
        let status_condition = Actor::spawn(StatusConditionActor::default(), handle);
        let data_reader_listener_thread = listener.map(DataReaderListenerThread::new);

        DataReaderActor {
            rtps_reader,
            changes: Vec::new(),
            topic_address,
            topic_name,
            type_name,
            topic_status_condition,
            type_support,
            _liveliness_changed_status: LivelinessChangedStatus::default(),
            requested_deadline_missed_status: ReaderRequestedDeadlineMissedStatus::default(),
            requested_incompatible_qos_status: RequestedIncompatibleQosStatus::default(),
            sample_lost_status: SampleLostStatus::default(),
            sample_rejected_status: SampleRejectedStatus::default(),
            subscription_matched_status: SubscriptionMatchedStatus::default(),
            matched_publication_list: HashMap::new(),
            enabled: false,
            data_available_status_changed_flag: false,
            incompatible_writer_list: HashSet::new(),
            status_condition,
            status_kind,
            data_reader_listener_thread,
            qos,
            instances: HashMap::new(),
            instance_deadline_missed_task: HashMap::new(),
        }
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        InstanceHandle::new(self.rtps_reader.guid().into())
    }

    fn get_requested_incompatible_qos_status(&mut self) -> RequestedIncompatibleQosStatus {
        self.requested_incompatible_qos_status.read_and_reset()
    }

    fn get_sample_lost_status(&mut self) -> SampleLostStatus {
        self.sample_lost_status.read_and_reset()
    }

    fn get_sample_rejected_status(&mut self) -> SampleRejectedStatus {
        self.sample_rejected_status.read_and_reset()
    }

    fn read_requested_deadline_missed_status(&mut self) -> RequestedDeadlineMissedStatus {
        let status = RequestedDeadlineMissedStatus {
            total_count: self.requested_deadline_missed_status.total_count,
            total_count_change: self.requested_deadline_missed_status.total_count_change,
            last_instance_handle: self.requested_deadline_missed_status.last_instance_handle,
        };

        self.requested_deadline_missed_status.total_count_change = 0;

        status
    }

    fn read(
        &mut self,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<(Option<Data>, SampleInfo)>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.status_condition
            .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                state: StatusKind::DataAvailable,
            });

        let indexed_sample_list = self.create_indexed_sample_collection(
            max_samples,
            &sample_states,
            &view_states,
            &instance_states,
            specific_instance_handle,
        )?;

        let change_index_list: Vec<usize>;
        let samples;

        (change_index_list, samples) = indexed_sample_list
            .into_iter()
            .map(|IndexedSample { index, sample }| (index, sample))
            .unzip();

        for index in change_index_list {
            self.changes[index].sample_state = SampleStateKind::Read;
        }

        Ok(samples)
    }

    fn take(
        &mut self,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<(Option<Data>, SampleInfo)>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let indexed_sample_list = self.create_indexed_sample_collection(
            max_samples,
            &sample_states,
            &view_states,
            &instance_states,
            specific_instance_handle,
        )?;

        self.status_condition
            .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                state: StatusKind::DataAvailable,
            });

        let mut change_index_list: Vec<usize>;
        let samples;

        (change_index_list, samples) = indexed_sample_list
            .into_iter()
            .map(|IndexedSample { index, sample }| (index, sample))
            .unzip();

        while let Some(index) = change_index_list.pop() {
            self.changes.remove(index);
        }

        Ok(samples)
    }

    fn get_subscription_matched_status(&mut self) -> SubscriptionMatchedStatus {
        self.status_condition
            .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                state: StatusKind::SubscriptionMatched,
            });

        self.subscription_matched_status
            .read_and_reset(self.matched_publication_list.len() as i32)
    }

    fn on_data_available(
        &self,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber: &SubscriberAsync,
        (subscriber_listener, subscriber_listener_mask): &(
            Option<MpscSender<SubscriberListenerMessage>>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        subscriber
            .get_statuscondition()
            .address()
            .send_actor_mail(AddCommunicationState {
                state: StatusKind::DataOnReaders,
            })?;

        self.status_condition
            .address()
            .send_actor_mail(AddCommunicationState {
                state: StatusKind::DataAvailable,
            })?;

        if subscriber_listener_mask.contains(&StatusKind::DataOnReaders) {
            if let Some(listener) = subscriber_listener {
                listener.send(SubscriberListenerMessage {
                    listener_operation: SubscriberListenerOperation::DataOnReaders(
                        subscriber.clone(),
                    ),
                })?;
            }
        } else if self.status_kind.contains(&StatusKind::DataAvailable) {
            let topic_status_condition_address = self.topic_status_condition.clone();
            let type_name = self.type_name.clone();
            let topic_name = self.topic_name.clone();
            let reader_address = data_reader_address.clone();
            let status_condition_address = self.status_condition.address();
            let subscriber = subscriber.clone();
            let topic = TopicAsync::new(
                self.topic_address.clone(),
                topic_status_condition_address.clone(),
                type_name.clone(),
                topic_name.clone(),
                subscriber.get_participant(),
            );
            if let Some(listener) = &self.data_reader_listener_thread {
                listener.sender().send(DataReaderListenerMessage {
                    listener_operation: DataReaderListenerOperation::DataAvailable,
                    reader_address,
                    status_condition_address,
                    subscriber,
                    topic,
                })?;
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<rtps::messages::types::Time>,
        reception_timestamp: rtps::messages::types::Time,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber: &SubscriberAsync,
        subscriber_mask_listener: &(
            Option<MpscSender<SubscriberListenerMessage>>,
            Vec<StatusKind>,
        ),
        participant_mask_listener: &(
            Option<MpscSender<ParticipantListenerMessage>>,
            Vec<StatusKind>,
        ),
        executor_handle: &ExecutorHandle,
        timer_handle: &TimerHandle,
    ) -> DdsResult<()> {
        let writer_guid = Guid::new(source_guid_prefix, data_submessage.writer_id());
        let sequence_number = data_submessage.writer_sn();
        let message_reader_id = data_submessage.reader_id();
        match &mut self.rtps_reader {
            RtpsReaderKind::Stateful(r) => {
                if let Some(writer_proxy) = r.matched_writer_lookup(writer_guid) {
                    //Stateful reader behavior
                    match self.qos.reliability.kind {
                        ReliabilityQosPolicyKind::BestEffort => {
                            let expected_seq_num = writer_proxy.available_changes_max() + 1;
                            if sequence_number >= expected_seq_num {
                                writer_proxy.received_change_set(sequence_number);
                                if sequence_number > expected_seq_num {
                                    writer_proxy.lost_changes_update(sequence_number);
                                    self.on_sample_lost(
                                        data_reader_address,
                                        subscriber,
                                        subscriber_mask_listener,
                                        participant_mask_listener,
                                    )?;
                                }
                                match self.convert_received_data_to_cache_change(
                                                writer_guid,
                                                data_submessage.inline_qos().clone(),
                                                data_submessage.serialized_payload().clone(),
                                                source_timestamp,
                                                reception_timestamp,
                                            ) {
                                                Ok(change) => {
                                                    self.add_change(
                                                        change,
                                                        data_reader_address,
                                                        subscriber,
                                                        subscriber_mask_listener,
                                                        participant_mask_listener,
                                                        executor_handle,
                                                        timer_handle,
                                                    )?;
                                                }
                                                Err(e) => debug!(
                                                    "Received invalid data on reader with GUID {guid:?}. Error: {err:?}.
                                                     Message writer ID: {writer_id:?}
                                                     Message reader ID: {reader_id:?}
                                                     Data submessage payload: {payload:?}",
                                                    guid = self.rtps_reader.guid(),
                                                    err = e,
                                                    writer_id = data_submessage.writer_id(),
                                                    reader_id = data_submessage.reader_id(),
                                                    payload = data_submessage.serialized_payload(),
                                                ),
                                            }
                            }
                        }
                        ReliabilityQosPolicyKind::Reliable => {
                            let expected_seq_num = writer_proxy.available_changes_max() + 1;
                            if sequence_number == expected_seq_num {
                                writer_proxy.received_change_set(sequence_number);
                                match self.convert_received_data_to_cache_change(
                                                writer_guid,
                                                data_submessage.inline_qos().clone(),
                                                data_submessage.serialized_payload().clone(),
                                                source_timestamp,
                                                reception_timestamp,
                                            ) {
                                                Ok(change) => {
                                                    self.add_change(
                                                        change,
                                                        data_reader_address,
                                                        subscriber,
                                                        subscriber_mask_listener,
                                                        participant_mask_listener,
                                                        executor_handle,timer_handle,
                                                    )?;
                                                }
                                                Err(e) => debug!(
                                                    "Received invalid data on reader with GUID {guid:?}. Error: {err:?}.
                                                     Message writer ID: {writer_id:?}
                                                     Message reader ID: {reader_id:?}
                                                     Data submessage payload: {payload:?}",
                                                    guid = self.rtps_reader.guid(),
                                                    err = e,
                                                    writer_id = data_submessage.writer_id(),
                                                    reader_id = data_submessage.reader_id(),
                                                    payload = data_submessage.serialized_payload(),
                                                ),
                                            }
                            }
                        }
                    }
                }
            }
            RtpsReaderKind::Stateless(r) => {
                if message_reader_id == ENTITYID_UNKNOWN
                    || message_reader_id == r.guid().entity_id()
                {
                    // Stateless reader behavior. We add the change if the data is correct. No error is printed
                    // because all readers would get changes marked with ENTITYID_UNKNOWN
                    if let Ok(change) = self.convert_received_data_to_cache_change(
                        writer_guid,
                        data_submessage.inline_qos().clone(),
                        data_submessage.serialized_payload().clone(),
                        source_timestamp,
                        reception_timestamp,
                    ) {
                        self.add_change(
                            change,
                            data_reader_address,
                            subscriber,
                            subscriber_mask_listener,
                            participant_mask_listener,
                            executor_handle,
                            timer_handle,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn on_data_frag_submessage_received(
        &mut self,
        data_frag_submessage: &DataFragSubmessage,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<rtps::messages::types::Time>,
        reception_timestamp: rtps::messages::types::Time,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber: &SubscriberAsync,
        subscriber_mask_listener: &(
            Option<MpscSender<SubscriberListenerMessage>>,
            Vec<StatusKind>,
        ),
        participant_mask_listener: &(
            Option<MpscSender<ParticipantListenerMessage>>,
            Vec<StatusKind>,
        ),
        executor_handle: &ExecutorHandle,
        timer_handle: &TimerHandle,
    ) -> DdsResult<()> {
        let sequence_number = data_frag_submessage.writer_sn();
        let writer_guid = Guid::new(source_guid_prefix, data_frag_submessage.writer_id());

        match &mut self.rtps_reader {
            RtpsReaderKind::Stateful(r) => {
                if let Some(writer_proxy) = r.matched_writer_lookup(writer_guid) {
                    writer_proxy.push_data_frag(data_frag_submessage.clone());
                    if let Some(data_submessage) =
                        writer_proxy.reconstruct_data_from_frag(sequence_number)
                    {
                        self.on_data_submessage_received(
                            &data_submessage,
                            source_guid_prefix,
                            source_timestamp,
                            reception_timestamp,
                            data_reader_address,
                            subscriber,
                            subscriber_mask_listener,
                            participant_mask_listener,
                            executor_handle,
                            timer_handle,
                        )?;
                    }
                }
            }
            RtpsReaderKind::Stateless(_) => (),
        }

        Ok(())
    }

    fn on_heartbeat_submessage_received(
        &mut self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
        message_sender_actor: &ActorAddress<MessageSenderActor>,
    ) {
        if self.qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let writer_guid = Guid::new(source_guid_prefix, heartbeat_submessage.writer_id());

            match &mut self.rtps_reader {
                RtpsReaderKind::Stateful(r) => {
                    if let Some(writer_proxy) = r.matched_writer_lookup(writer_guid) {
                        if writer_proxy.last_received_heartbeat_count()
                            < heartbeat_submessage.count()
                        {
                            writer_proxy
                                .set_last_received_heartbeat_count(heartbeat_submessage.count());

                            writer_proxy.set_must_send_acknacks(
                                !heartbeat_submessage.final_flag()
                                    || (!heartbeat_submessage.liveliness_flag()
                                        && !writer_proxy.missing_changes().count() == 0),
                            );

                            if !heartbeat_submessage.final_flag() {
                                writer_proxy.set_must_send_acknacks(true);
                            }
                            writer_proxy.missing_changes_update(heartbeat_submessage.last_sn());
                            writer_proxy.lost_changes_update(heartbeat_submessage.first_sn());

                            self.send_message(message_sender_actor);
                        }
                    }
                }
                RtpsReaderKind::Stateless(_) => (),
            }
        }
    }

    fn on_heartbeat_frag_submessage_received(
        &mut self,
        heartbeat_frag_submessage: &HeartbeatFragSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let writer_guid = Guid::new(source_guid_prefix, heartbeat_frag_submessage.writer_id());

            match &mut self.rtps_reader {
                RtpsReaderKind::Stateful(r) => {
                    if let Some(writer_proxy) = r.matched_writer_lookup(writer_guid) {
                        if writer_proxy.last_received_heartbeat_count()
                            < heartbeat_frag_submessage.count()
                        {
                            writer_proxy.set_last_received_heartbeat_frag_count(
                                heartbeat_frag_submessage.count(),
                            );
                        }
                    }
                }
                RtpsReaderKind::Stateless(_) => (),
            }
        }
    }

    fn get_discovered_writer_incompatible_qos_policy_list(
        &self,
        discovered_writer_data: &DiscoveredWriterData,
        subscriber_qos: &SubscriberQos,
    ) -> Vec<QosPolicyId> {
        let writer_info = discovered_writer_data.dds_publication_data();

        let mut incompatible_qos_policy_list = Vec::new();

        if subscriber_qos.presentation.access_scope > writer_info.presentation().access_scope
            || subscriber_qos.presentation.coherent_access
                != writer_info.presentation().coherent_access
            || subscriber_qos.presentation.ordered_access
                != writer_info.presentation().ordered_access
        {
            incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
        }
        if &self.qos.durability > writer_info.durability() {
            incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
        }
        if &self.qos.deadline > writer_info.deadline() {
            incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
        }
        if &self.qos.latency_budget > writer_info.latency_budget() {
            incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
        }
        if &self.qos.liveliness > writer_info.liveliness() {
            incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
        }
        if self.qos.reliability.kind > writer_info.reliability().kind {
            incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
        }
        if &self.qos.destination_order > writer_info.destination_order() {
            incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
        }

        incompatible_qos_policy_list
    }

    fn on_gap_submessage_received(
        &mut self,
        gap_submessage: &GapSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, gap_submessage.writer_id());
        match &mut self.rtps_reader {
            RtpsReaderKind::Stateful(r) => {
                if let Some(writer_proxy) = r.matched_writer_lookup(writer_guid) {
                    for seq_num in gap_submessage.gap_start()..gap_submessage.gap_list().base() {
                        writer_proxy.irrelevant_change_set(seq_num)
                    }

                    for seq_num in gap_submessage.gap_list().set() {
                        writer_proxy.irrelevant_change_set(seq_num)
                    }
                }
            }
            RtpsReaderKind::Stateless(_) => (),
        }
    }

    fn on_sample_lost(
        &mut self,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber: &SubscriberAsync,
        (subscriber_listener, subscriber_listener_mask): &(
            Option<MpscSender<SubscriberListenerMessage>>,
            Vec<StatusKind>,
        ),
        (participant_listener, participant_listener_mask): &(
            Option<MpscSender<ParticipantListenerMessage>>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        self.sample_lost_status.increment();
        self.status_condition
            .send_actor_mail(AddCommunicationState {
                state: StatusKind::SampleLost,
            });
        let topic_status_condition_address = self.topic_status_condition.clone();
        if self.status_kind.contains(&StatusKind::SampleLost) {
            let type_name = self.type_name.clone();
            let topic_name = self.topic_name.clone();
            let status = self.get_sample_lost_status();
            let reader_address = data_reader_address.clone();
            let status_condition_address = self.status_condition.address();
            let subscriber = subscriber.clone();
            let topic = TopicAsync::new(
                self.topic_address.clone(),
                topic_status_condition_address.clone(),
                type_name.clone(),
                topic_name.clone(),
                subscriber.get_participant(),
            );
            if let Some(listener) = &self.data_reader_listener_thread {
                listener.sender().send(DataReaderListenerMessage {
                    listener_operation: DataReaderListenerOperation::SampleLost(status),
                    reader_address,
                    status_condition_address,
                    subscriber,
                    topic,
                })?;
            }
        } else if subscriber_listener_mask.contains(&StatusKind::SampleLost) {
            let status = self.get_sample_lost_status();
            if let Some(listener) = subscriber_listener {
                listener.send(SubscriberListenerMessage {
                    listener_operation: SubscriberListenerOperation::SampleLost(status),
                })?;
            }
        } else if participant_listener_mask.contains(&StatusKind::SampleLost) {
            let status = self.get_sample_lost_status();
            if let Some(listener) = participant_listener {
                listener.send(ParticipantListenerMessage {
                    listener_operation: ParticipantListenerOperation::SampleLost(status),
                })?;
            }
        }

        Ok(())
    }

    fn on_subscription_matched(
        &mut self,
        instance_handle: InstanceHandle,
        data_reader_address: ActorAddress<DataReaderActor>,
        subscriber: SubscriberAsync,
        (subscriber_listener, subscriber_listener_mask): &(
            Option<MpscSender<SubscriberListenerMessage>>,
            Vec<StatusKind>,
        ),
        (participant_listener, participant_listener_mask): &(
            Option<MpscSender<ParticipantListenerMessage>>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        self.subscription_matched_status.increment(instance_handle);
        self.status_condition
            .send_actor_mail(AddCommunicationState {
                state: StatusKind::SubscriptionMatched,
            });

        const SUBSCRIPTION_MATCHED_STATUS_KIND: &StatusKind = &StatusKind::SubscriptionMatched;
        if self.status_kind.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
            let type_name = self.type_name.clone();
            let topic_name = self.topic_name.clone();
            let status = self.get_subscription_matched_status();
            let reader_address = data_reader_address.clone();
            let status_condition_address = self.status_condition.address();
            let subscriber = subscriber.clone();

            let topic_status_condition_address = self.topic_status_condition.clone();
            let topic = TopicAsync::new(
                self.topic_address.clone(),
                topic_status_condition_address.clone(),
                type_name.clone(),
                topic_name.clone(),
                subscriber.get_participant(),
            );
            if let Some(listener) = &self.data_reader_listener_thread {
                listener.sender().send(DataReaderListenerMessage {
                    listener_operation: DataReaderListenerOperation::SubscriptionMatched(status),
                    reader_address,
                    status_condition_address,
                    subscriber,
                    topic,
                })?;
            }
        } else if subscriber_listener_mask.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
            let status = self.get_subscription_matched_status();
            if let Some(listener) = subscriber_listener {
                listener.send(SubscriberListenerMessage {
                    listener_operation: SubscriberListenerOperation::SubscriptionMatched(status),
                })?;
            }
        } else if participant_listener_mask.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
            let status = self.get_subscription_matched_status();
            if let Some(listener) = participant_listener {
                listener.send(ParticipantListenerMessage {
                    listener_operation: ParticipantListenerOperation::SubscriptionMatched(status),
                })?;
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn on_sample_rejected(
        &mut self,
        instance_handle: InstanceHandle,
        rejected_reason: SampleRejectedStatusKind,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber: &SubscriberAsync,
        (subscriber_listener, subscriber_listener_mask): &(
            Option<MpscSender<SubscriberListenerMessage>>,
            Vec<StatusKind>,
        ),
        (participant_listener, participant_listener_mask): &(
            Option<MpscSender<ParticipantListenerMessage>>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        self.sample_rejected_status
            .increment(instance_handle, rejected_reason);
        self.status_condition
            .send_actor_mail(AddCommunicationState {
                state: StatusKind::SampleRejected,
            });
        if self.status_kind.contains(&StatusKind::SampleRejected) {
            let type_name = self.type_name.clone();
            let topic_name = self.topic_name.clone();
            let status = self.get_sample_rejected_status();

            let topic_status_condition_address = self.topic_status_condition.clone();
            let reader_address = data_reader_address.clone();
            let status_condition_address = self.status_condition.address();
            let subscriber = subscriber.clone();
            let topic = TopicAsync::new(
                self.topic_address.clone(),
                topic_status_condition_address.clone(),
                type_name.clone(),
                topic_name.clone(),
                subscriber.get_participant(),
            );
            if let Some(listener) = &self.data_reader_listener_thread {
                listener.sender().send(DataReaderListenerMessage {
                    listener_operation: DataReaderListenerOperation::SampleRejected(status),
                    reader_address,
                    status_condition_address,
                    subscriber,
                    topic,
                })?;
            }
        } else if subscriber_listener_mask.contains(&StatusKind::SampleRejected) {
            let status = self.get_sample_rejected_status();
            if let Some(listener) = subscriber_listener {
                listener.send(SubscriberListenerMessage {
                    listener_operation: SubscriberListenerOperation::SampleRejected(status),
                })?;
            }
        } else if participant_listener_mask.contains(&StatusKind::SampleRejected) {
            let status = self.get_sample_rejected_status();
            if let Some(listener) = participant_listener {
                listener.send(ParticipantListenerMessage {
                    listener_operation: ParticipantListenerOperation::SampleRejected(status),
                })?;
            }
        }

        Ok(())
    }

    fn on_requested_incompatible_qos(
        &mut self,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber: &SubscriberAsync,
        (subscriber_listener, subscriber_listener_mask): &(
            Option<MpscSender<SubscriberListenerMessage>>,
            Vec<StatusKind>,
        ),
        (participant_listener, participant_listener_mask): &(
            Option<MpscSender<ParticipantListenerMessage>>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        self.requested_incompatible_qos_status
            .increment(incompatible_qos_policy_list);
        self.status_condition
            .send_actor_mail(AddCommunicationState {
                state: StatusKind::RequestedIncompatibleQos,
            });

        if self
            .status_kind
            .contains(&StatusKind::RequestedIncompatibleQos)
        {
            let type_name = self.type_name.clone();
            let topic_name = self.topic_name.clone();
            let status = self.get_requested_incompatible_qos_status();
            let topic_status_condition_address = self.topic_status_condition.clone();
            let reader_address = data_reader_address.clone();
            let status_condition_address = self.status_condition.address();
            let subscriber = subscriber.clone();
            let topic = TopicAsync::new(
                self.topic_address.clone(),
                topic_status_condition_address.clone(),
                type_name.clone(),
                topic_name.clone(),
                subscriber.get_participant(),
            );
            if let Some(listener) = &self.data_reader_listener_thread {
                listener.sender().send(DataReaderListenerMessage {
                    listener_operation: DataReaderListenerOperation::RequestedIncompatibleQos(
                        status,
                    ),
                    reader_address,
                    status_condition_address,
                    subscriber,
                    topic,
                })?;
            }
        } else if subscriber_listener_mask.contains(&StatusKind::RequestedIncompatibleQos) {
            let status = self.get_requested_incompatible_qos_status();
            if let Some(listener) = subscriber_listener {
                listener.send(SubscriberListenerMessage {
                    listener_operation: SubscriberListenerOperation::RequestedIncompatibleQos(
                        status,
                    ),
                })?;
            }
        } else if participant_listener_mask.contains(&StatusKind::RequestedIncompatibleQos) {
            let status = self.get_requested_incompatible_qos_status();
            if let Some(listener) = participant_listener {
                listener.send(ParticipantListenerMessage {
                    listener_operation: ParticipantListenerOperation::RequestedIncompatibleQos(
                        status,
                    ),
                })?;
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn convert_received_data_to_cache_change(
        &mut self,
        writer_guid: Guid,
        inline_qos: ParameterList,
        data: Data,
        source_timestamp: Option<rtps::messages::types::Time>,
        reception_timestamp: rtps::messages::types::Time,
    ) -> DdsResult<ReaderCacheChange> {
        let change_kind = if let Some(p) = inline_qos
            .parameter()
            .iter()
            .find(|&x| x.parameter_id() == PID_STATUS_INFO)
        {
            let mut deserializer =
                ClassicCdrDeserializer::new(p.value(), CdrEndianness::LittleEndian);
            let status_info: StatusInfo = CdrDeserialize::deserialize(&mut deserializer).unwrap();
            match status_info {
                STATUS_INFO_DISPOSED => Ok(ChangeKind::NotAliveDisposed),
                STATUS_INFO_UNREGISTERED => Ok(ChangeKind::NotAliveUnregistered),
                STATUS_INFO_DISPOSED_UNREGISTERED => Ok(ChangeKind::NotAliveDisposedUnregistered),
                _ => Err(DdsError::Error("Unknown status info value".to_string())),
            }
        } else {
            Ok(ChangeKind::Alive)
        }?;

        let instance_handle = build_instance_handle(
            &self.type_support,
            change_kind,
            data.as_ref(),
            inline_qos.parameter(),
        )?;

        match change_kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => {
                self.instances
                    .entry(instance_handle)
                    .or_insert_with(InstanceState::new)
                    .update_state(change_kind);
                Ok(())
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => {
                match self.instances.get_mut(&instance_handle) {
                    Some(instance) => {
                        instance.update_state(change_kind);
                        Ok(())
                    }
                    None => Err(DdsError::Error(
                        "Received message changing state of unknown instance".to_string(),
                    )),
                }
            }
        }?;

        Ok(ReaderCacheChange {
            rtps_cache_change: RtpsCacheChange {
                kind: change_kind,
                writer_guid,
                instance_handle: instance_handle.into(),
                data_value: data,
                inline_qos,
            },
            sample_state: SampleStateKind::NotRead,
            disposed_generation_count: self.instances[&instance_handle]
                .most_recent_disposed_generation_count,
            no_writers_generation_count: self.instances[&instance_handle]
                .most_recent_no_writers_generation_count,
            reception_timestamp,
            source_timestamp,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn add_change(
        &mut self,
        change: ReaderCacheChange,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber: &SubscriberAsync,
        subscriber_mask_listener: &(
            Option<MpscSender<SubscriberListenerMessage>>,
            Vec<StatusKind>,
        ),
        participant_mask_listener: &(
            Option<MpscSender<ParticipantListenerMessage>>,
            Vec<StatusKind>,
        ),
        executor_handle: &ExecutorHandle,
        timer_handle: &TimerHandle,
    ) -> DdsResult<()> {
        if self.is_sample_of_interest_based_on_time(&change) {
            if self.is_max_samples_limit_reached(&change) {
                self.on_sample_rejected(
                    change.instance_handle(),
                    SampleRejectedStatusKind::RejectedBySamplesLimit,
                    data_reader_address,
                    subscriber,
                    subscriber_mask_listener,
                    participant_mask_listener,
                )?;
            } else if self.is_max_instances_limit_reached(&change) {
                self.on_sample_rejected(
                    change.instance_handle(),
                    SampleRejectedStatusKind::RejectedByInstancesLimit,
                    data_reader_address,
                    subscriber,
                    subscriber_mask_listener,
                    participant_mask_listener,
                )?;
            } else if self.is_max_samples_per_instance_limit_reached(&change) {
                self.on_sample_rejected(
                    change.instance_handle(),
                    SampleRejectedStatusKind::RejectedBySamplesPerInstanceLimit,
                    data_reader_address,
                    subscriber,
                    subscriber_mask_listener,
                    participant_mask_listener,
                )?;
            } else {
                let num_alive_samples_of_instance = self
                    .changes
                    .iter()
                    .filter(|cc| {
                        cc.instance_handle() == change.instance_handle()
                            && cc.rtps_cache_change.kind == ChangeKind::Alive
                    })
                    .count() as i32;

                if let HistoryQosPolicyKind::KeepLast(depth) = self.qos.history.kind {
                    if depth == num_alive_samples_of_instance {
                        let index_sample_to_remove = self
                            .changes
                            .iter()
                            .position(|cc| {
                                cc.instance_handle() == change.instance_handle()
                                    && cc.rtps_cache_change.kind == ChangeKind::Alive
                            })
                            .expect("Samples must exist");
                        self.changes.remove(index_sample_to_remove);
                    }
                }

                self.start_deadline_missed_task(
                    change.instance_handle(),
                    data_reader_address.clone(),
                    subscriber.clone(),
                    subscriber_mask_listener,
                    participant_mask_listener,
                    executor_handle,
                    timer_handle,
                )?;

                tracing::debug!(cache_change = ?change, "Adding change to data reader history cache");
                self.changes.push(change);
                self.data_available_status_changed_flag = true;

                match self.qos.destination_order.kind {
                    DestinationOrderQosPolicyKind::BySourceTimestamp => {
                        self.changes.sort_by(|a, b| {
                            a.source_timestamp
                                .as_ref()
                                .expect("Missing source timestamp")
                                .cmp(
                                    b.source_timestamp
                                        .as_ref()
                                        .expect("Missing source timestamp"),
                                )
                        });
                    }
                    DestinationOrderQosPolicyKind::ByReceptionTimestamp => self
                        .changes
                        .sort_by(|a, b| a.reception_timestamp.cmp(&b.reception_timestamp)),
                }

                self.on_data_available(data_reader_address, subscriber, subscriber_mask_listener)?;
            }
        }

        Ok(())
    }

    fn is_sample_of_interest_based_on_time(&self, change: &ReaderCacheChange) -> bool {
        let closest_timestamp_before_received_sample = self
            .changes
            .iter()
            .filter(|cc| cc.instance_handle() == change.instance_handle())
            .filter(|cc| cc.source_timestamp <= change.source_timestamp)
            .map(|cc| cc.source_timestamp)
            .max();

        if let Some(Some(t)) = closest_timestamp_before_received_sample {
            if let Some(sample_source_time) = change.source_timestamp {
                let sample_separation = infrastructure::time::Duration::from(sample_source_time)
                    - infrastructure::time::Duration::from(t);
                DurationKind::Finite(sample_separation)
                    >= self.qos.time_based_filter.minimum_separation
            } else {
                true
            }
        } else {
            true
        }
    }

    fn is_max_samples_limit_reached(&self, _change: &ReaderCacheChange) -> bool {
        let total_samples = self
            .changes
            .iter()
            .filter(|cc| cc.rtps_cache_change.kind == ChangeKind::Alive)
            .count();

        total_samples == self.qos.resource_limits.max_samples
    }

    fn is_max_instances_limit_reached(&self, change: &ReaderCacheChange) -> bool {
        let instance_handle_list: HashSet<_> =
            self.changes.iter().map(|cc| cc.instance_handle()).collect();

        if instance_handle_list.contains(&change.instance_handle()) {
            false
        } else {
            instance_handle_list.len() == self.qos.resource_limits.max_instances
        }
    }

    fn is_max_samples_per_instance_limit_reached(&self, change: &ReaderCacheChange) -> bool {
        let total_samples_of_instance = self
            .changes
            .iter()
            .filter(|cc| cc.instance_handle() == change.instance_handle())
            .count();

        total_samples_of_instance == self.qos.resource_limits.max_samples_per_instance
    }

    fn create_indexed_sample_collection(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<IndexedSample>> {
        if let Some(h) = specific_instance_handle {
            if !self.instances.contains_key(&h) {
                return Err(DdsError::BadParameter);
            }
        };

        let mut indexed_samples = Vec::new();

        let instances = &self.instances;
        let mut instances_in_collection = HashMap::new();
        for (index, cache_change) in self
            .changes
            .iter()
            .enumerate()
            .filter(|(_, cc)| {
                sample_states.contains(&cc.sample_state)
                    && view_states.contains(&instances[&cc.instance_handle()].view_state)
                    && instance_states.contains(&instances[&cc.instance_handle()].instance_state)
                    && if let Some(h) = specific_instance_handle {
                        h == cc.instance_handle()
                    } else {
                        true
                    }
            })
            .take(max_samples as usize)
        {
            instances_in_collection
                .entry(cache_change.instance_handle())
                .or_insert_with(InstanceState::new);

            instances_in_collection
                .get_mut(&cache_change.instance_handle())
                .unwrap()
                .update_state(cache_change.rtps_cache_change.kind);
            let sample_state = cache_change.sample_state;
            let view_state = self.instances[&cache_change.instance_handle()].view_state;
            let instance_state = self.instances[&cache_change.instance_handle()].instance_state;

            let absolute_generation_rank = (self.instances[&cache_change.instance_handle()]
                .most_recent_disposed_generation_count
                + self.instances[&cache_change.instance_handle()]
                    .most_recent_no_writers_generation_count)
                - (instances_in_collection[&cache_change.instance_handle()]
                    .most_recent_disposed_generation_count
                    + instances_in_collection[&cache_change.instance_handle()]
                        .most_recent_no_writers_generation_count);

            let (data, valid_data) = match cache_change.rtps_cache_change.kind {
                ChangeKind::Alive | ChangeKind::AliveFiltered => (
                    Some(cache_change.rtps_cache_change.data_value.clone()),
                    true,
                ),
                ChangeKind::NotAliveDisposed
                | ChangeKind::NotAliveUnregistered
                | ChangeKind::NotAliveDisposedUnregistered => (None, false),
            };

            let sample_info = SampleInfo {
                sample_state,
                view_state,
                instance_state,
                disposed_generation_count: cache_change.disposed_generation_count,
                no_writers_generation_count: cache_change.no_writers_generation_count,
                sample_rank: 0,     // To be filled up after collection is created
                generation_rank: 0, // To be filled up after collection is created
                absolute_generation_rank,
                source_timestamp: cache_change.source_timestamp.map(Into::into),
                instance_handle: cache_change.instance_handle(),
                publication_handle: InstanceHandle::new(
                    cache_change.rtps_cache_change.writer_guid.into(),
                ),
                valid_data,
            };

            let sample = (data, sample_info);

            indexed_samples.push(IndexedSample { index, sample })
        }

        // After the collection is created, update the relative generation rank values and mark the read instances as viewed
        for handle in instances_in_collection.into_keys() {
            let most_recent_sample_absolute_generation_rank = indexed_samples
                .iter()
                .filter(
                    |IndexedSample {
                         sample: (_, sample_info),
                         ..
                     }| sample_info.instance_handle == handle,
                )
                .map(
                    |IndexedSample {
                         sample: (_, sample_info),
                         ..
                     }| sample_info.absolute_generation_rank,
                )
                .last()
                .expect("Instance handle must exist on collection");

            let mut total_instance_samples_in_collection = indexed_samples
                .iter()
                .filter(
                    |IndexedSample {
                         sample: (_, sample_info),
                         ..
                     }| sample_info.instance_handle == handle,
                )
                .count();

            for IndexedSample {
                sample: (_, sample_info),
                ..
            } in indexed_samples.iter_mut().filter(
                |IndexedSample {
                     sample: (_, sample_info),
                     ..
                 }| sample_info.instance_handle == handle,
            ) {
                sample_info.generation_rank = sample_info.absolute_generation_rank
                    - most_recent_sample_absolute_generation_rank;

                total_instance_samples_in_collection -= 1;
                sample_info.sample_rank = total_instance_samples_in_collection as i32;
            }

            self.instances
                .get_mut(&handle)
                .expect("Sample must exist on hash map")
                .mark_viewed()
        }

        if indexed_samples.is_empty() {
            Err(DdsError::NoData)
        } else {
            Ok(indexed_samples)
        }
    }

    fn next_instance(&self, previous_handle: Option<InstanceHandle>) -> Option<InstanceHandle> {
        match previous_handle {
            Some(p) => self.instances.keys().filter(|&h| h > &p).min().cloned(),
            None => self.instances.keys().min().cloned(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn start_deadline_missed_task(
        &mut self,
        change_instance_handle: InstanceHandle,
        data_reader_address: ActorAddress<DataReaderActor>,
        subscriber: SubscriberAsync,
        subscriber_mask_listener: &(
            Option<MpscSender<SubscriberListenerMessage>>,
            Vec<StatusKind>,
        ),
        participant_mask_listener: &(
            Option<MpscSender<ParticipantListenerMessage>>,
            Vec<StatusKind>,
        ),
        executor_handle: &ExecutorHandle,
        timer_handle: &TimerHandle,
    ) -> DdsResult<()> {
        if let Some(t) = self
            .instance_deadline_missed_task
            .remove(&change_instance_handle)
        {
            t.abort();
        }

        if let DurationKind::Finite(deadline_missed_period) = self.qos.deadline.period {
            let deadline_missed_interval = std::time::Duration::new(
                deadline_missed_period.sec() as u64,
                deadline_missed_period.nanosec(),
            );
            let reader_status_condition = self.status_condition.address();
            let data_reader_listener_sender = self
                .data_reader_listener_thread
                .as_ref()
                .map(|l| l.sender().clone());
            let reader_listener_mask = self.status_kind.clone();
            let subscriber_listener = subscriber_mask_listener.0.clone();
            let subscriber_listener_mask = subscriber_mask_listener.1.clone();
            let participant_listener = participant_mask_listener.0.clone();
            let participant_listener_mask = participant_mask_listener.1.clone();
            let topic_address = self.topic_address.clone();
            let type_name = self.type_name.clone();
            let topic_name = self.topic_name.clone();
            let status_condition_address = self.status_condition.address();
            let topic_status_condition_address = self.topic_status_condition.clone();
            let timer_handle = timer_handle.clone();
            let deadline_missed_task = executor_handle.spawn(async move {
                loop {
                    timer_handle.sleep(deadline_missed_interval).await;
                    let subscriber_listener = subscriber_listener.clone();
                    let participant_listener = participant_listener.clone();
                    let r: DdsResult<()> =
                        async {
                            data_reader_address.send_actor_mail(
                                IncrementRequestedDeadlineMissedStatus {
                                    instance_handle: change_instance_handle,
                                },
                            )?;

                            reader_status_condition
                                .send_actor_mail(AddCommunicationState {
                                    state: StatusKind::RequestedDeadlineMissed,
                                })?
                                .receive_reply()
                                .await;
                            if reader_listener_mask.contains(&StatusKind::RequestedDeadlineMissed) {
                                let status = data_reader_address
                                    .send_actor_mail(ReadRequestedDeadlineMissedStatus)?
                                    .receive_reply()
                                    .await;
                                let reader_address = data_reader_address.clone();
                                let subscriber = subscriber.clone();
                                let topic = TopicAsync::new(
                                    topic_address.clone(),
                                    topic_status_condition_address.clone(),
                                    type_name.clone(),
                                    topic_name.clone(),
                                    subscriber.get_participant(),
                                );
                                let status_condition_address = status_condition_address.clone();

                                if let Some(listener) = &data_reader_listener_sender {
                                    listener
                                    .send(DataReaderListenerMessage {
                                        listener_operation:
                                            DataReaderListenerOperation::RequestedDeadlineMissed(
                                                status,
                                            ),
                                        reader_address,
                                        status_condition_address,
                                        subscriber,
                                        topic,
                                    })
                                    .ok();
                                }
                            } else if subscriber_listener_mask
                                .contains(&StatusKind::RequestedDeadlineMissed)
                            {
                                let status = data_reader_address
                                    .send_actor_mail(ReadRequestedDeadlineMissedStatus)?
                                    .receive_reply()
                                    .await;
                                if let Some(listener) = subscriber_listener {
                                    listener
                                    .send(SubscriberListenerMessage {
                                        listener_operation:
                                            SubscriberListenerOperation::RequestedDeadlineMissed(
                                                status,
                                            ),
                                    })
                                    .ok();
                                }
                            } else if participant_listener_mask
                                .contains(&StatusKind::RequestedDeadlineMissed)
                            {
                                let status = data_reader_address
                                    .send_actor_mail(ReadRequestedDeadlineMissedStatus)?
                                    .receive_reply()
                                    .await;
                                if let Some(listener) = participant_listener {
                                    listener.send(ParticipantListenerMessage {
                                    listener_operation:
                                        ParticipantListenerOperation::RequestedDeadlineMissed(
                                            status,
                                        ),
                                }).ok();
                                }
                            }
                            Ok(())
                        }
                        .await;

                    if r.is_err() {
                        break;
                    }
                }
            });

            self.instance_deadline_missed_task
                .insert(change_instance_handle, deadline_missed_task);
        }
        Ok(())
    }

    fn send_message(&mut self, message_sender_actor: &ActorAddress<MessageSenderActor>) {
        match &mut self.rtps_reader {
            RtpsReaderKind::Stateful(r) => r.send_message(message_sender_actor),
            RtpsReaderKind::Stateless(_) => (),
        }
    }
}

pub struct Read {
    pub max_samples: i32,
    pub sample_states: Vec<SampleStateKind>,
    pub view_states: Vec<ViewStateKind>,
    pub instance_states: Vec<InstanceStateKind>,
    pub specific_instance_handle: Option<InstanceHandle>,
}
impl Mail for Read {
    type Result = DdsResult<Vec<(Option<Data>, SampleInfo)>>;
}
impl MailHandler<Read> for DataReaderActor {
    fn handle(&mut self, message: Read) -> <Read as Mail>::Result {
        self.read(
            message.max_samples,
            message.sample_states,
            message.view_states,
            message.instance_states,
            message.specific_instance_handle,
        )
    }
}

pub struct Take {
    pub max_samples: i32,
    pub sample_states: Vec<SampleStateKind>,
    pub view_states: Vec<ViewStateKind>,
    pub instance_states: Vec<InstanceStateKind>,
    pub specific_instance_handle: Option<InstanceHandle>,
}
impl Mail for Take {
    type Result = DdsResult<Vec<(Option<Data>, SampleInfo)>>;
}
impl MailHandler<Take> for DataReaderActor {
    fn handle(&mut self, message: Take) -> <Take as Mail>::Result {
        self.take(
            message.max_samples,
            message.sample_states,
            message.view_states,
            message.instance_states,
            message.specific_instance_handle,
        )
    }
}

pub struct IsHistoricalDataReceived;
impl Mail for IsHistoricalDataReceived {
    type Result = DdsResult<bool>;
}
impl MailHandler<IsHistoricalDataReceived> for DataReaderActor {
    fn handle(
        &mut self,
        _: IsHistoricalDataReceived,
    ) -> <IsHistoricalDataReceived as Mail>::Result {
        if !self.enabled {
            Err(DdsError::NotEnabled)
        } else {
            Ok(())
        }?;

        match self.qos.durability.kind {
            DurabilityQosPolicyKind::Volatile => Err(DdsError::IllegalOperation),
            DurabilityQosPolicyKind::TransientLocal => Ok(()),
        }?;

        match &self.rtps_reader {
            RtpsReaderKind::Stateful(r) => Ok(r.is_historical_data_received()),
            RtpsReaderKind::Stateless(_) => Ok(true),
        }
    }
}

pub struct AsDiscoveredReaderData {
    pub subscriber_qos: SubscriberQos,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub topic_data: TopicDataQosPolicy,
    pub xml_type: String,
}
impl Mail for AsDiscoveredReaderData {
    type Result = DdsResult<DiscoveredReaderData>;
}
impl MailHandler<AsDiscoveredReaderData> for DataReaderActor {
    fn handle(
        &mut self,
        message: AsDiscoveredReaderData,
    ) -> <AsDiscoveredReaderData as Mail>::Result {
        let guid = self.rtps_reader.guid();
        let type_name = self.type_name.clone();
        let topic_name = self.topic_name.clone();

        let unicast_locator_list = if self.rtps_reader.unicast_locator_list().is_empty() {
            message.default_unicast_locator_list
        } else {
            self.rtps_reader.unicast_locator_list().to_vec()
        };

        let multicast_locator_list = if self.rtps_reader.unicast_locator_list().is_empty() {
            message.default_multicast_locator_list
        } else {
            self.rtps_reader.multicast_locator_list().to_vec()
        };

        Ok(DiscoveredReaderData::new(
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
                topic_name,
                type_name,
                self.qos.clone(),
                message.subscriber_qos.clone(),
                message.topic_data,
                message.xml_type,
            ),
        ))
    }
}

pub struct GetInstanceHandle;
impl Mail for GetInstanceHandle {
    type Result = InstanceHandle;
}
impl MailHandler<GetInstanceHandle> for DataReaderActor {
    fn handle(&mut self, _: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
        self.get_instance_handle()
    }
}

pub struct SetQos {
    pub qos: DataReaderQos,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for DataReaderActor {
    fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        message.qos.is_consistent()?;
        if self.enabled {
            message.qos.check_immutability(&self.qos)?;
        }
        self.qos = message.qos;
        Ok(())
    }
}

pub struct GetQos;
impl Mail for GetQos {
    type Result = DataReaderQos;
}
impl MailHandler<GetQos> for DataReaderActor {
    fn handle(&mut self, _: GetQos) -> <GetQos as Mail>::Result {
        self.qos.clone()
    }
}

pub struct GetMatchedPublicationData {
    pub publication_handle: InstanceHandle,
}
impl Mail for GetMatchedPublicationData {
    type Result = DdsResult<PublicationBuiltinTopicData>;
}
impl MailHandler<GetMatchedPublicationData> for DataReaderActor {
    fn handle(
        &mut self,
        message: GetMatchedPublicationData,
    ) -> <GetMatchedPublicationData as Mail>::Result {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.matched_publication_list
            .get(&message.publication_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }
}

pub struct IsEnabled;
impl Mail for IsEnabled {
    type Result = bool;
}
impl MailHandler<IsEnabled> for DataReaderActor {
    fn handle(&mut self, _: IsEnabled) -> <IsEnabled as Mail>::Result {
        self.enabled
    }
}

pub struct Enable;
impl Mail for Enable {
    type Result = ();
}
impl MailHandler<Enable> for DataReaderActor {
    fn handle(&mut self, _: Enable) -> <Enable as Mail>::Result {
        self.enabled = true;
    }
}

pub struct GetStatuscondition;
impl Mail for GetStatuscondition {
    type Result = ActorAddress<StatusConditionActor>;
}
impl MailHandler<GetStatuscondition> for DataReaderActor {
    fn handle(&mut self, _: GetStatuscondition) -> <GetStatuscondition as Mail>::Result {
        self.status_condition.address()
    }
}

pub struct GetMatchedPublications;
impl Mail for GetMatchedPublications {
    type Result = Vec<InstanceHandle>;
}
impl MailHandler<GetMatchedPublications> for DataReaderActor {
    fn handle(&mut self, _: GetMatchedPublications) -> <GetMatchedPublications as Mail>::Result {
        self.matched_publication_list
            .iter()
            .map(|(&key, _)| key)
            .collect()
    }
}

pub struct TakeNextInstance {
    pub max_samples: i32,
    pub previous_handle: Option<InstanceHandle>,
    pub sample_states: Vec<SampleStateKind>,
    pub view_states: Vec<ViewStateKind>,
    pub instance_states: Vec<InstanceStateKind>,
}
impl Mail for TakeNextInstance {
    type Result = DdsResult<Vec<(Option<Data>, SampleInfo)>>;
}
impl MailHandler<TakeNextInstance> for DataReaderActor {
    fn handle(&mut self, message: TakeNextInstance) -> <TakeNextInstance as Mail>::Result {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(message.previous_handle) {
            Some(next_handle) => self.take(
                message.max_samples,
                message.sample_states,
                message.view_states,
                message.instance_states,
                Some(next_handle),
            ),
            None => Err(DdsError::NoData),
        }
    }
}

pub struct GetSubscriptionMatchedStatus;
impl Mail for GetSubscriptionMatchedStatus {
    type Result = SubscriptionMatchedStatus;
}
impl MailHandler<GetSubscriptionMatchedStatus> for DataReaderActor {
    fn handle(
        &mut self,
        _: GetSubscriptionMatchedStatus,
    ) -> <GetSubscriptionMatchedStatus as Mail>::Result {
        self.get_subscription_matched_status()
    }
}

pub struct ReadNextInstance {
    pub max_samples: i32,
    pub previous_handle: Option<InstanceHandle>,
    pub sample_states: Vec<SampleStateKind>,
    pub view_states: Vec<ViewStateKind>,
    pub instance_states: Vec<InstanceStateKind>,
}
impl Mail for ReadNextInstance {
    type Result = DdsResult<Vec<(Option<Data>, SampleInfo)>>;
}
impl MailHandler<ReadNextInstance> for DataReaderActor {
    fn handle(&mut self, message: ReadNextInstance) -> <ReadNextInstance as Mail>::Result {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(message.previous_handle) {
            Some(next_handle) => self.read(
                message.max_samples,
                message.sample_states,
                message.view_states,
                message.instance_states,
                Some(next_handle),
            ),
            None => Err(DdsError::NoData),
        }
    }
}

pub struct AddMatchedWriter {
    pub discovered_writer_data: DiscoveredWriterData,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub data_reader_address: ActorAddress<DataReaderActor>,
    pub subscriber: SubscriberAsync,
    pub subscriber_qos: SubscriberQos,
    pub subscriber_mask_listener: (
        Option<MpscSender<SubscriberListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
}
impl Mail for AddMatchedWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<AddMatchedWriter> for DataReaderActor {
    fn handle(&mut self, message: AddMatchedWriter) -> <AddMatchedWriter as Mail>::Result {
        let type_name = self.type_name.clone();
        let topic_name = self.topic_name.clone();
        let publication_builtin_topic_data = message.discovered_writer_data.dds_publication_data();
        if publication_builtin_topic_data.topic_name() == topic_name
            && publication_builtin_topic_data.get_type_name() == type_name
        {
            tracing::trace!(
                topic_name = topic_name,
                type_name = type_name,
                "Writer with matched topic and type found",
            );
            let instance_handle =
                InstanceHandle::try_from_key(&message.discovered_writer_data.get_key().unwrap())
                    .unwrap();
            let incompatible_qos_policy_list = self
                .get_discovered_writer_incompatible_qos_policy_list(
                    &message.discovered_writer_data,
                    &message.subscriber_qos,
                );
            if incompatible_qos_policy_list.is_empty() {
                let unicast_locator_list = if message
                    .discovered_writer_data
                    .writer_proxy()
                    .unicast_locator_list()
                    .is_empty()
                {
                    message.default_unicast_locator_list
                } else {
                    message
                        .discovered_writer_data
                        .writer_proxy()
                        .unicast_locator_list()
                        .to_vec()
                };

                let multicast_locator_list = if message
                    .discovered_writer_data
                    .writer_proxy()
                    .multicast_locator_list()
                    .is_empty()
                {
                    message.default_multicast_locator_list
                } else {
                    message
                        .discovered_writer_data
                        .writer_proxy()
                        .multicast_locator_list()
                        .to_vec()
                };

                let writer_proxy = RtpsWriterProxy::new(
                    message
                        .discovered_writer_data
                        .writer_proxy()
                        .remote_writer_guid(),
                    &unicast_locator_list,
                    &multicast_locator_list,
                    message
                        .discovered_writer_data
                        .writer_proxy()
                        .data_max_size_serialized(),
                    message
                        .discovered_writer_data
                        .writer_proxy()
                        .remote_group_entity_id(),
                );

                match &mut self.rtps_reader {
                    RtpsReaderKind::Stateful(r) => r.matched_writer_add(writer_proxy),
                    RtpsReaderKind::Stateless(_) => (),
                }

                let insert_matched_publication_result = self
                    .matched_publication_list
                    .insert(instance_handle, publication_builtin_topic_data.clone());
                match insert_matched_publication_result {
                    Some(value) if &value != publication_builtin_topic_data => {
                        self.on_subscription_matched(
                            instance_handle,
                            message.data_reader_address,
                            message.subscriber,
                            &message.subscriber_mask_listener,
                            &message.participant_mask_listener,
                        )?;
                    }
                    None => {
                        self.on_subscription_matched(
                            instance_handle,
                            message.data_reader_address,
                            message.subscriber,
                            &message.subscriber_mask_listener,
                            &message.participant_mask_listener,
                        )?;
                    }
                    _ => (),
                }
            } else if !self.incompatible_writer_list.contains(&instance_handle) {
                self.incompatible_writer_list.insert(instance_handle);
                self.on_requested_incompatible_qos(
                    incompatible_qos_policy_list,
                    &message.data_reader_address,
                    &message.subscriber,
                    &message.subscriber_mask_listener,
                    &message.participant_mask_listener,
                )?;
            }
        }

        Ok(())
    }
}

pub struct RemoveMatchedWriter {
    pub discovered_writer_handle: InstanceHandle,
    pub data_reader_address: ActorAddress<DataReaderActor>,
    pub subscriber: SubscriberAsync,
    pub subscriber_mask_listener: (
        Option<MpscSender<SubscriberListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
}
impl Mail for RemoveMatchedWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<RemoveMatchedWriter> for DataReaderActor {
    fn handle(&mut self, message: RemoveMatchedWriter) -> <RemoveMatchedWriter as Mail>::Result {
        let matched_publication = self
            .matched_publication_list
            .remove(&message.discovered_writer_handle);
        if let Some(w) = matched_publication {
            match &mut self.rtps_reader {
                RtpsReaderKind::Stateful(r) => r.matched_writer_remove(w.key().value.into()),
                RtpsReaderKind::Stateless(_) => (),
            }

            self.on_subscription_matched(
                message.discovered_writer_handle,
                message.data_reader_address,
                message.subscriber,
                &message.subscriber_mask_listener,
                &message.participant_mask_listener,
            )?;
        }
        Ok(())
    }
}

pub struct GetTopicName;
impl Mail for GetTopicName {
    type Result = DdsResult<String>;
}
impl MailHandler<GetTopicName> for DataReaderActor {
    fn handle(&mut self, _: GetTopicName) -> <GetTopicName as Mail>::Result {
        Ok(self.topic_name.clone())
    }
}

pub struct GetTypeName;
impl Mail for GetTypeName {
    type Result = DdsResult<String>;
}
impl MailHandler<GetTypeName> for DataReaderActor {
    fn handle(&mut self, _: GetTypeName) -> <GetTypeName as Mail>::Result {
        Ok(self.type_name.clone())
    }
}

pub struct ProcessDataSubmessage {
    pub data_submessage: DataSubmessage,
    pub source_guid_prefix: GuidPrefix,
    pub source_timestamp: Option<rtps::messages::types::Time>,
    pub reception_timestamp: rtps::messages::types::Time,
    pub data_reader_address: ActorAddress<DataReaderActor>,
    pub subscriber: SubscriberAsync,
    pub subscriber_mask_listener: (
        Option<MpscSender<SubscriberListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub executor_handle: ExecutorHandle,
    pub timer_handle: TimerHandle,
}
impl Mail for ProcessDataSubmessage {
    type Result = ();
}
impl MailHandler<ProcessDataSubmessage> for DataReaderActor {
    fn handle(
        &mut self,
        message: ProcessDataSubmessage,
    ) -> <ProcessDataSubmessage as Mail>::Result {
        self.on_data_submessage_received(
            &message.data_submessage,
            message.source_guid_prefix,
            message.source_timestamp,
            message.reception_timestamp,
            &message.data_reader_address,
            &message.subscriber,
            &message.subscriber_mask_listener,
            &message.participant_mask_listener,
            &message.executor_handle,
            &message.timer_handle,
        )
        .ok();
    }
}

pub struct ProcessDataFragSubmessage {
    pub data_frag_submessage: DataFragSubmessage,
    pub source_guid_prefix: GuidPrefix,
    pub source_timestamp: Option<rtps::messages::types::Time>,
    pub reception_timestamp: rtps::messages::types::Time,
    pub data_reader_address: ActorAddress<DataReaderActor>,
    pub subscriber: SubscriberAsync,
    pub subscriber_mask_listener: (
        Option<MpscSender<SubscriberListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub executor_handle: ExecutorHandle,
    pub timer_handle: TimerHandle,
}
impl Mail for ProcessDataFragSubmessage {
    type Result = ();
}
impl MailHandler<ProcessDataFragSubmessage> for DataReaderActor {
    fn handle(
        &mut self,
        message: ProcessDataFragSubmessage,
    ) -> <ProcessDataFragSubmessage as Mail>::Result {
        self.on_data_frag_submessage_received(
            &message.data_frag_submessage,
            message.source_guid_prefix,
            message.source_timestamp,
            message.reception_timestamp,
            &message.data_reader_address,
            &message.subscriber,
            &message.subscriber_mask_listener,
            &message.participant_mask_listener,
            &message.executor_handle,
            &message.timer_handle,
        )
        .ok();
    }
}

pub struct ProcessGapSubmessage {
    pub gap_submessage: GapSubmessage,
    pub source_guid_prefix: GuidPrefix,
}
impl Mail for ProcessGapSubmessage {
    type Result = ();
}
impl MailHandler<ProcessGapSubmessage> for DataReaderActor {
    fn handle(&mut self, message: ProcessGapSubmessage) -> <ProcessGapSubmessage as Mail>::Result {
        self.on_gap_submessage_received(&message.gap_submessage, message.source_guid_prefix);
    }
}

pub struct ProcessHeartbeatSubmessage {
    pub heartbeat_submessage: HeartbeatSubmessage,
    pub source_guid_prefix: GuidPrefix,
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
}
impl Mail for ProcessHeartbeatSubmessage {
    type Result = ();
}
impl MailHandler<ProcessHeartbeatSubmessage> for DataReaderActor {
    fn handle(
        &mut self,
        message: ProcessHeartbeatSubmessage,
    ) -> <ProcessHeartbeatSubmessage as Mail>::Result {
        self.on_heartbeat_submessage_received(
            &message.heartbeat_submessage,
            message.source_guid_prefix,
            &message.message_sender_actor,
        );
    }
}

pub struct ProcessHeartbeatFragSubmessage {
    pub heartbeat_frag_submessage: HeartbeatFragSubmessage,
    pub source_guid_prefix: GuidPrefix,
}
impl Mail for ProcessHeartbeatFragSubmessage {
    type Result = ();
}
impl MailHandler<ProcessHeartbeatFragSubmessage> for DataReaderActor {
    fn handle(
        &mut self,
        message: ProcessHeartbeatFragSubmessage,
    ) -> <ProcessHeartbeatFragSubmessage as Mail>::Result {
        self.on_heartbeat_frag_submessage_received(
            &message.heartbeat_frag_submessage,
            message.source_guid_prefix,
        )
    }
}

pub struct SendMessage {
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
}
impl Mail for SendMessage {
    type Result = ();
}
impl MailHandler<SendMessage> for DataReaderActor {
    fn handle(&mut self, message: SendMessage) -> <SendMessage as Mail>::Result {
        self.send_message(&message.message_sender_actor)
    }
}

pub struct SetListener {
    pub listener: Option<Box<dyn AnyDataReaderListener + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for SetListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetListener> for DataReaderActor {
    fn handle(&mut self, message: SetListener) -> <SetListener as Mail>::Result {
        if let Some(listener) = self.data_reader_listener_thread.take() {
            listener.join()?;
        }
        self.data_reader_listener_thread = message.listener.map(DataReaderListenerThread::new);
        self.status_kind = message.status_kind;
        Ok(())
    }
}

pub struct GetRequestedDeadlineMissedStatus;
impl Mail for GetRequestedDeadlineMissedStatus {
    type Result = RequestedDeadlineMissedStatus;
}
impl MailHandler<GetRequestedDeadlineMissedStatus> for DataReaderActor {
    fn handle(
        &mut self,
        _: GetRequestedDeadlineMissedStatus,
    ) -> <GetRequestedDeadlineMissedStatus as Mail>::Result {
        self.read_requested_deadline_missed_status()
    }
}

pub struct GetTopicAddress;
impl Mail for GetTopicAddress {
    type Result = ActorAddress<TopicActor>;
}
impl MailHandler<GetTopicAddress> for DataReaderActor {
    fn handle(&mut self, _: GetTopicAddress) -> <GetTopicAddress as Mail>::Result {
        self.topic_address.clone()
    }
}
