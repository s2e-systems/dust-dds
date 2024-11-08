use tracing::debug;

use super::{
    any_data_reader_listener::{AnyDataReaderListener, DataReaderListenerOperation},
    handle::DataReaderHandle,
    status_condition_actor::{self, StatusConditionActor},
};
use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    dds_async::{subscriber::SubscriberAsync, topic::TopicAsync},
    implementation::{
        actor::{Actor, ActorAddress},
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::DiscoveredWriterData,
        },
        data_representation_inline_qos::{
            parameter_id_values::{PID_KEY_HASH, PID_STATUS_INFO},
            types::{
                StatusInfo, STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED,
                STATUS_INFO_UNREGISTERED,
            },
        },
        runtime::{
            executor::{block_on, ExecutorHandle, TaskHandle},
            mpsc::{mpsc_channel, MpscSender},
        },
        xtypes_glue::key_and_instance_handle::{
            get_instance_handle_from_serialized_foo, get_instance_handle_from_serialized_key,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos, TopicQos},
        qos_policy::{
            DestinationOrderQosPolicyKind, DurabilityQosPolicyKind, HistoryQosPolicyKind,
            OwnershipQosPolicyKind, QosPolicyId, ReliabilityQosPolicyKind,
            DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
            RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
        status::{
            LivelinessChangedStatus, QosPolicyCount, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
            SampleRejectedStatusKind, StatusKind, SubscriptionMatchedStatus,
        },
        time::{DurationKind, Time},
    },
    rtps::{
        messages::submessage_elements::{Data, ParameterList},
        reader::{ReaderCacheChange, TransportReader},
        types::{ChangeKind, DurabilityKind, Guid, ReliabilityKind},
    },
    subscription::sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
    xtypes::{
        deserialize::XTypesDeserialize, dynamic_type::DynamicType,
        xcdr_deserializer::Xcdr1LeDeserializer,
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

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
struct ReaderSample {
    kind: ChangeKind,
    writer_guid: Guid,
    instance_handle: InstanceHandle,
    source_timestamp: Option<Time>,
    data_value: Data,
    _inline_qos: ParameterList,
    sample_state: SampleStateKind,
    disposed_generation_count: i32,
    no_writers_generation_count: i32,
    reception_timestamp: Time,
}

impl ReaderSample {
    fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle.into()
    }
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
    subscriber_async: SubscriberAsync,
}

impl DataReaderListenerThread {
    fn new(
        mut listener: Box<dyn AnyDataReaderListener + Send>,
        subscriber_async: SubscriberAsync,
    ) -> Self {
        let (sender, receiver) = mpsc_channel::<DataReaderListenerMessage>();
        let thread = std::thread::Builder::new()
            .name("Data reader listener".to_string())
            .spawn(move || {
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
            })
            .expect("failed to spawn thread");
        Self {
            thread,
            sender,
            subscriber_async,
        }
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

pub struct DataReaderActorListener {
    pub data_reader_listener: Box<dyn AnyDataReaderListener + Send>,
    pub subscriber_async: SubscriberAsync,
}

pub struct DataReaderActor {
    instance_handle: InstanceHandle,
    sample_list: Vec<ReaderSample>,
    qos: DataReaderQos,
    topic_name: String,
    type_name: String,
    type_support: Arc<dyn DynamicType + Send + Sync>,
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
    data_reader_status_kind: Vec<StatusKind>,
    instances: HashMap<InstanceHandle, InstanceState>,
    instance_deadline_missed_task: HashMap<InstanceHandle, TaskHandle>,
    instance_ownership: HashMap<InstanceHandle, Guid>,
    transport_reader: Box<dyn TransportReader>,
}

impl DataReaderActor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance_handle: InstanceHandle,
        topic_name: String,
        type_name: String,
        type_support: Arc<dyn DynamicType + Send + Sync>,
        qos: DataReaderQos,
        listener: Option<DataReaderActorListener>,
        data_reader_status_kind: Vec<StatusKind>,
        transport_reader: Box<dyn TransportReader>,
        executor_handle: &ExecutorHandle,
    ) -> Self {
        let status_condition = Actor::spawn(StatusConditionActor::default(), executor_handle);
        let data_reader_listener_thread = listener
            .map(|x| DataReaderListenerThread::new(x.data_reader_listener, x.subscriber_async));

        DataReaderActor {
            instance_handle,
            sample_list: Vec::new(),
            topic_name,
            type_name,
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
            data_reader_status_kind,
            data_reader_listener_thread,
            qos,
            instances: HashMap::new(),
            instance_deadline_missed_task: HashMap::new(),
            instance_ownership: HashMap::new(),
            transport_reader,
        }
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }

    pub fn get_statuscondition(&self) -> ActorAddress<StatusConditionActor> {
        self.status_condition.address()
    }

    pub fn get_requested_deadline_missed_status(&mut self) -> RequestedDeadlineMissedStatus {
        let status = RequestedDeadlineMissedStatus {
            total_count: self.requested_deadline_missed_status.total_count,
            total_count_change: self.requested_deadline_missed_status.total_count_change,
            last_instance_handle: self.requested_deadline_missed_status.last_instance_handle,
        };

        self.requested_deadline_missed_status.total_count_change = 0;

        status
    }

    pub fn read(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
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
            self.sample_list[index].sample_state = SampleStateKind::Read;
        }

        Ok(samples)
    }

    pub fn take(
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
            self.sample_list.remove(index);
        }

        Ok(samples)
    }

    fn on_data_available(
        &mut self,
        // data_reader_address: &ActorAddress<DataReaderActor>,
        // subscriber: &SubscriberAsync,
        // (subscriber_listener, subscriber_listener_mask): &(
        //     Option<MpscSender<SubscriberListenerMessage>>,
        //     Vec<StatusKind>,
        // ),
    ) -> DdsResult<()> {
        // let topic_status_condition_address = self.topic_status_condition.clone();
        // let type_name = self.type_name.clone();
        // let topic_name = self.topic_name.clone();
        // let reader_address = data_reader_address.clone();
        // let status_condition_address = self.status_condition.address();
        // let subscriber = subscriber.clone();
        // let topic = TopicAsync::new(
        //     self.topic_address.clone(),
        //     topic_status_condition_address.clone(),
        //     type_name.clone(),
        //     topic_name.clone(),
        //     subscriber.get_participant(),
        // );
        // if subscriber_listener_mask.contains(&StatusKind::DataOnReaders) {
        //     if let Some(listener) = subscriber_listener {
        //         listener.send(SubscriberListenerMessage {
        //             listener_operation: SubscriberListenerOperation::DataOnReaders(
        //                 subscriber.clone(),
        //             ),
        //             reader_address,
        //             status_condition_address,
        //             subscriber: subscriber.clone(),
        //             topic,
        //         })?;
        //     }
        // } else if self
        //     .data_reader_status_kind
        //     .contains(&StatusKind::DataAvailable)
        // {
        //     if let Some(listener) = &self.data_reader_listener_thread {
        //         listener.sender().send(DataReaderListenerMessage {
        //             listener_operation: DataReaderListenerOperation::DataAvailable,
        //             reader_address,
        //             status_condition_address,
        //             subscriber: subscriber.clone(),
        //             topic,
        //         })?;
        //     }
        // }

        self.status_condition
            .send_actor_mail(status_condition_actor::AddCommunicationState {
                state: StatusKind::DataAvailable,
            });
        Ok(())
    }

    fn get_discovered_writer_incompatible_qos_policy_list(
        &self,
        discovered_writer_data: &DiscoveredWriterData,
        subscriber_qos: &SubscriberQos,
    ) -> Vec<QosPolicyId> {
        let writer_info = &discovered_writer_data.dds_publication_data;

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
        if &self.qos.deadline < writer_info.deadline() {
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
        if self.qos.ownership.kind != writer_info.ownership().kind {
            incompatible_qos_policy_list.push(OWNERSHIP_QOS_POLICY_ID);
        }

        let writer_offered_representation = writer_info
            .representation()
            .value
            .first()
            .unwrap_or(&XCDR_DATA_REPRESENTATION);
        if !self
            .qos
            .representation
            .value
            .contains(writer_offered_representation)
        {
            // Empty list is interpreted as containing XCDR_DATA_REPRESENTATION
            if !(writer_offered_representation == &XCDR_DATA_REPRESENTATION
                && self.qos.representation.value.is_empty())
            {
                incompatible_qos_policy_list.push(DATA_REPRESENTATION_QOS_POLICY_ID)
            }
        }

        incompatible_qos_policy_list
    }

    fn on_sample_lost(&mut self) -> DdsResult<()> {
        self.sample_lost_status.increment();

        // let topic_status_condition_address = self.topic_status_condition.clone();
        // let type_name = self.type_name.clone();
        // let topic_name = self.topic_name.clone();
        // let reader_address = data_reader_address.clone();
        // let status_condition_address = self.status_condition.address();
        // let subscriber = subscriber.clone();
        // let topic = TopicAsync::new(
        //     self.topic_address.clone(),
        //     topic_status_condition_address.clone(),
        //     type_name.clone(),
        //     topic_name.clone(),
        //     subscriber.get_participant(),
        // );
        // if self
        //     .data_reader_status_kind
        //     .contains(&StatusKind::SampleLost)
        // {
        //     let status = self.sample_lost_status.read_and_reset();
        //     if let Some(listener) = &self.data_reader_listener_thread {
        //         listener.sender().send(DataReaderListenerMessage {
        //             listener_operation: DataReaderListenerOperation::SampleLost(status),
        //             reader_address,
        //             status_condition_address,
        //             subscriber,
        //             topic,
        //         })?;
        //     }
        // } else if subscriber_listener_mask.contains(&StatusKind::SampleLost) {
        //     let status = self.sample_lost_status.read_and_reset();
        //     if let Some(listener) = subscriber_listener {
        //         listener.send(SubscriberListenerMessage {
        //             listener_operation: SubscriberListenerOperation::SampleLost(status),
        //             reader_address,
        //             status_condition_address,
        //             subscriber,
        //             topic,
        //         })?;
        //     }
        // } else if participant_listener_mask.contains(&StatusKind::SampleLost) {
        //     let status = self.sample_lost_status.read_and_reset();
        //     if let Some(listener) = participant_listener {
        //         listener.send(ParticipantListenerMessage {
        //             listener_operation: ParticipantListenerOperation::SampleLost(status),
        //             listener_kind: ListenerKind::Reader {
        //                 reader_address,
        //                 status_condition_address,
        //                 subscriber,
        //                 topic,
        //             },
        //         })?;
        //     }
        // }
        self.status_condition
            .send_actor_mail(status_condition_actor::AddCommunicationState {
                state: StatusKind::SampleLost,
            });

        Ok(())
    }

    fn on_subscription_matched(
        &mut self,
        instance_handle: InstanceHandle,
        // data_reader_address: ActorAddress<DataReaderActor>,
        // subscriber: SubscriberAsync,
        // (subscriber_listener, subscriber_listener_mask): &(
        //     Option<MpscSender<SubscriberListenerMessage>>,
        //     Vec<StatusKind>,
        // ),
        // (participant_listener, participant_listener_mask): &(
        //     Option<MpscSender<ParticipantListenerMessage>>,
        //     Vec<StatusKind>,
        // ),
    ) {
        self.subscription_matched_status.increment(instance_handle);

        // const SUBSCRIPTION_MATCHED_STATUS_KIND: &StatusKind = &StatusKind::SubscriptionMatched;
        // let type_name = self.type_name.clone();
        // let topic_name = self.topic_name.clone();
        // let reader_address = data_reader_address.clone();
        // let status_condition_address = self.status_condition.address();
        // let subscriber = subscriber.clone();

        // let topic_status_condition_address = self.topic_status_condition.clone();
        // let topic = TopicAsync::new(
        //     self.topic_address.clone(),
        //     topic_status_condition_address.clone(),
        //     type_name.clone(),
        //     topic_name.clone(),
        //     subscriber.get_participant(),
        // );
        // if self
        //     .data_reader_status_kind
        //     .contains(SUBSCRIPTION_MATCHED_STATUS_KIND)
        // {
        //     let status = self
        //         .subscription_matched_status
        //         .read_and_reset(self.matched_publication_list.len() as i32);
        //     if let Some(listener) = &self.data_reader_listener_thread {
        //         listener.sender().send(DataReaderListenerMessage {
        //             listener_operation: DataReaderListenerOperation::SubscriptionMatched(status),
        //             reader_address,
        //             status_condition_address,
        //             subscriber,
        //             topic,
        //         })?;
        //     }
        // } else if subscriber_listener_mask.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
        //     let status = self
        //         .subscription_matched_status
        //         .read_and_reset(self.matched_publication_list.len() as i32);
        //     if let Some(listener) = subscriber_listener {
        //         listener.send(SubscriberListenerMessage {
        //             listener_operation: SubscriberListenerOperation::SubscriptionMatched(status),
        //             reader_address,
        //             status_condition_address,
        //             subscriber,
        //             topic,
        //         })?;
        //     }
        // } else if participant_listener_mask.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
        //     let status = self
        //         .subscription_matched_status
        //         .read_and_reset(self.matched_publication_list.len() as i32);
        //     if let Some(listener) = participant_listener {
        //         listener.send(ParticipantListenerMessage {
        //             listener_operation: ParticipantListenerOperation::SubscriptionMatched(status),
        //             listener_kind: ListenerKind::Reader {
        //                 reader_address,
        //                 status_condition_address,
        //                 subscriber,
        //                 topic,
        //             },
        //         })?;
        //     }
        // }
        self.status_condition
            .send_actor_mail(status_condition_actor::AddCommunicationState {
                state: StatusKind::SubscriptionMatched,
            });
    }

    #[allow(clippy::too_many_arguments)]
    fn on_sample_rejected(
        &mut self,
        instance_handle: InstanceHandle,
        rejected_reason: SampleRejectedStatusKind,
        // data_reader_address: &ActorAddress<DataReaderActor>,
        // subscriber: &SubscriberAsync,
        // (subscriber_listener, subscriber_listener_mask): &(
        //     Option<MpscSender<SubscriberListenerMessage>>,
        //     Vec<StatusKind>,
        // ),
        // (participant_listener, participant_listener_mask): &(
        //     Option<MpscSender<ParticipantListenerMessage>>,
        //     Vec<StatusKind>,
        // ),
    ) {
        self.sample_rejected_status
            .increment(instance_handle, rejected_reason);

        // let type_name = self.type_name.clone();
        // let topic_name = self.topic_name.clone();

        // let topic_status_condition_address = self.topic_status_condition.clone();
        // let reader_address = data_reader_address.clone();
        // let status_condition_address = self.status_condition.address();
        // let subscriber = subscriber.clone();
        // let topic = TopicAsync::new(
        //     self.topic_address.clone(),
        //     topic_status_condition_address.clone(),
        //     type_name.clone(),
        //     topic_name.clone(),
        //     subscriber.get_participant(),
        // );
        // if self
        //     .data_reader_status_kind
        //     .contains(&StatusKind::SampleRejected)
        // {
        //     let status = self.sample_rejected_status.read_and_reset();
        //     if let Some(listener) = &self.data_reader_listener_thread {
        //         listener.sender().send(DataReaderListenerMessage {
        //             listener_operation: DataReaderListenerOperation::SampleRejected(status),
        //             reader_address,
        //             status_condition_address,
        //             subscriber,
        //             topic,
        //         })?;
        //     }
        // } else if subscriber_listener_mask.contains(&StatusKind::SampleRejected) {
        //     let status = self.sample_rejected_status.read_and_reset();
        //     if let Some(listener) = subscriber_listener {
        //         listener.send(SubscriberListenerMessage {
        //             listener_operation: SubscriberListenerOperation::SampleRejected(status),
        //             reader_address,
        //             status_condition_address,
        //             subscriber,
        //             topic,
        //         })?;
        //     }
        // } else if participant_listener_mask.contains(&StatusKind::SampleRejected) {
        //     let status = self.sample_rejected_status.read_and_reset();
        //     if let Some(listener) = participant_listener {
        //         listener.send(ParticipantListenerMessage {
        //             listener_operation: ParticipantListenerOperation::SampleRejected(status),
        //             listener_kind: ListenerKind::Reader {
        //                 reader_address,
        //                 status_condition_address,
        //                 subscriber,
        //                 topic,
        //             },
        //         })?;
        //     }
        // }
        self.status_condition
            .send_actor_mail(status_condition_actor::AddCommunicationState {
                state: StatusKind::SampleRejected,
            });
    }

    fn on_requested_incompatible_qos(
        &mut self,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
        // data_reader_address: &ActorAddress<DataReaderActor>,
        // subscriber: &SubscriberAsync,
        // (subscriber_listener, subscriber_listener_mask): &(
        //     Option<MpscSender<SubscriberListenerMessage>>,
        //     Vec<StatusKind>,
        // ),
        // (participant_listener, participant_listener_mask): &(
        //     Option<MpscSender<ParticipantListenerMessage>>,
        //     Vec<StatusKind>,
        // ),
    ) {
        self.requested_incompatible_qos_status
            .increment(incompatible_qos_policy_list);

        // let type_name = self.type_name.clone();
        // let topic_name = self.topic_name.clone();
        // let topic_status_condition_address = self.topic_status_condition.clone();
        // let reader_address = data_reader_address.clone();
        // let status_condition_address = self.status_condition.address();
        // let subscriber = subscriber.clone();
        // let topic = TopicAsync::new(
        //     self.topic_address.clone(),
        //     topic_status_condition_address.clone(),
        //     type_name.clone(),
        //     topic_name.clone(),
        //     subscriber.get_participant(),
        // );
        // if self
        //     .data_reader_status_kind
        //     .contains(&StatusKind::RequestedIncompatibleQos)
        // {
        //     let status = self.requested_incompatible_qos_status.read_and_reset();
        //     if let Some(listener) = &self.data_reader_listener_thread {
        //         listener.sender().send(DataReaderListenerMessage {
        //             listener_operation: DataReaderListenerOperation::RequestedIncompatibleQos(
        //                 status,
        //             ),
        //             reader_address,
        //             status_condition_address,
        //             subscriber,
        //             topic,
        //         })?;
        //     }
        // } else if subscriber_listener_mask.contains(&StatusKind::RequestedIncompatibleQos) {
        //     let status = self.requested_incompatible_qos_status.read_and_reset();
        //     if let Some(listener) = subscriber_listener {
        //         listener.send(SubscriberListenerMessage {
        //             listener_operation: SubscriberListenerOperation::RequestedIncompatibleQos(
        //                 status,
        //             ),
        //             reader_address,
        //             status_condition_address,
        //             subscriber,
        //             topic,
        //         })?;
        //     }
        // } else if participant_listener_mask.contains(&StatusKind::RequestedIncompatibleQos) {
        //     let status = self.requested_incompatible_qos_status.read_and_reset();
        //     if let Some(listener) = participant_listener {
        //         listener.send(ParticipantListenerMessage {
        //             listener_operation: ParticipantListenerOperation::RequestedIncompatibleQos(
        //                 status,
        //             ),
        //             listener_kind: ListenerKind::Reader {
        //                 reader_address,
        //                 status_condition_address,
        //                 subscriber,
        //                 topic,
        //             },
        //         })?;
        //     }
        // }
        self.status_condition
            .send_actor_mail(status_condition_actor::AddCommunicationState {
                state: StatusKind::RequestedIncompatibleQos,
            });
    }

    fn convert_cache_change_to_sample(
        &mut self,
        cache_change: ReaderCacheChange,
        reception_timestamp: Time,
    ) -> DdsResult<ReaderSample> {
        let instance_handle = {
            match cache_change.kind {
                ChangeKind::Alive | ChangeKind::AliveFiltered => {
                    get_instance_handle_from_serialized_foo(
                        cache_change.data_value.as_ref(),
                        self.type_support.as_ref(),
                    )?
                }
                ChangeKind::NotAliveDisposed
                | ChangeKind::NotAliveUnregistered
                | ChangeKind::NotAliveDisposedUnregistered => match &cache_change
                    .inline_qos
                    .parameter()
                    .iter()
                    .find(|&x| x.parameter_id() == PID_KEY_HASH)
                {
                    Some(p) => {
                        if let Ok(key) = <[u8; 16]>::try_from(p.value()) {
                            InstanceHandle::new(key)
                        } else {
                            get_instance_handle_from_serialized_key(
                                cache_change.data_value.as_ref(),
                                self.type_support.as_ref(),
                            )?
                        }
                    }
                    None => get_instance_handle_from_serialized_key(
                        cache_change.data_value.as_ref(),
                        self.type_support.as_ref(),
                    )?,
                },
            }
        };

        // Update the state of the instance before creating since this has direct impact on
        // the information that is store on the sample
        match cache_change.kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => {
                self.instances
                    .entry(instance_handle)
                    .or_insert_with(InstanceState::new)
                    .update_state(cache_change.kind);
                Ok(())
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => {
                match self.instances.get_mut(&instance_handle) {
                    Some(instance) => {
                        instance.update_state(cache_change.kind);
                        Ok(())
                    }
                    None => Err(DdsError::Error(
                        "Received message changing state of unknown instance".to_string(),
                    )),
                }
            }
        }?;

        Ok(ReaderSample {
            kind: cache_change.kind,
            writer_guid: cache_change.writer_guid,
            instance_handle: instance_handle.into(),
            source_timestamp: cache_change.source_timestamp.map(Into::into),
            data_value: cache_change.data_value,
            _inline_qos: cache_change.inline_qos,
            sample_state: SampleStateKind::NotRead,
            disposed_generation_count: self.instances[&instance_handle]
                .most_recent_disposed_generation_count,
            no_writers_generation_count: self.instances[&instance_handle]
                .most_recent_no_writers_generation_count,
            reception_timestamp,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn add_sample(
        &mut self,
        sample: ReaderSample,
        // data_reader_address: &ActorAddress<DataReaderActor>,
        // subscriber: &SubscriberAsync,
        // subscriber_mask_listener: &(
        //     Option<MpscSender<SubscriberListenerMessage>>,
        //     Vec<StatusKind>,
        // ),
        // participant_mask_listener: &(
        //     Option<MpscSender<ParticipantListenerMessage>>,
        //     Vec<StatusKind>,
        // ),
    ) -> DdsResult<()> {
        // For exclusive access if the writer is not the allowed to write the sample do an early return
        if self.qos.ownership.kind == OwnershipQosPolicyKind::Exclusive {
            // Get the InstanceHandle of the data writer owning this instance
            if let Some(&instance_owner_handle) =
                self.instance_ownership.get(&sample.instance_handle())
            {
                let instance_owner = InstanceHandle::new(instance_owner_handle.into());
                let instance_writer = InstanceHandle::new(sample.writer_guid.into());
                if instance_owner_handle != sample.writer_guid
                    && self.matched_publication_list[&instance_writer]
                        .ownership_strength()
                        .value
                        <= self.matched_publication_list[&instance_owner]
                            .ownership_strength()
                            .value
                {
                    return Ok(());
                }
            }

            self.instance_ownership
                .insert(sample.instance_handle(), sample.writer_guid);
        }

        if matches!(
            sample.kind,
            ChangeKind::NotAliveDisposed
                | ChangeKind::NotAliveUnregistered
                | ChangeKind::NotAliveDisposedUnregistered
        ) {
            // Drop the ownership and stop the deadline task
            self.instance_ownership.remove(&sample.instance_handle());
            if let Some(t) = self
                .instance_deadline_missed_task
                .remove(&sample.instance_handle())
            {
                t.abort();
            }
        }

        if self.is_sample_of_interest_based_on_time(&sample) {
            if self.is_max_samples_limit_reached(&sample) {
                self.on_sample_rejected(
                    sample.instance_handle(),
                    SampleRejectedStatusKind::RejectedBySamplesLimit,
                    // data_reader_address,
                    // subscriber,
                    // subscriber_mask_listener,
                    // participant_mask_listener,
                );
            } else if self.is_max_instances_limit_reached(&sample) {
                self.on_sample_rejected(
                    sample.instance_handle(),
                    SampleRejectedStatusKind::RejectedByInstancesLimit,
                    // data_reader_address,
                    // subscriber,
                    // subscriber_mask_listener,
                    // participant_mask_listener,
                );
            } else if self.is_max_samples_per_instance_limit_reached(&sample) {
                self.on_sample_rejected(
                    sample.instance_handle(),
                    SampleRejectedStatusKind::RejectedBySamplesPerInstanceLimit,
                    // data_reader_address,
                    // subscriber,
                    // subscriber_mask_listener,
                    // participant_mask_listener,
                );
            } else {
                let num_alive_samples_of_instance = self
                    .sample_list
                    .iter()
                    .filter(|cc| {
                        cc.instance_handle() == sample.instance_handle()
                            && cc.kind == ChangeKind::Alive
                    })
                    .count() as u32;

                if let HistoryQosPolicyKind::KeepLast(depth) = self.qos.history.kind {
                    if depth == num_alive_samples_of_instance {
                        let index_sample_to_remove = self
                            .sample_list
                            .iter()
                            .position(|cc| {
                                cc.instance_handle() == sample.instance_handle()
                                    && cc.kind == ChangeKind::Alive
                            })
                            .expect("Samples must exist");
                        self.sample_list.remove(index_sample_to_remove);
                    }
                }

                match sample.kind {
                    ChangeKind::Alive | ChangeKind::AliveFiltered => {
                        self.instances
                            .entry(sample.instance_handle)
                            .or_insert_with(InstanceState::new)
                            .update_state(sample.kind);
                        Ok(())
                    }
                    ChangeKind::NotAliveDisposed
                    | ChangeKind::NotAliveUnregistered
                    | ChangeKind::NotAliveDisposedUnregistered => {
                        match self.instances.get_mut(&sample.instance_handle) {
                            Some(instance) => {
                                instance.update_state(sample.kind);
                                Ok(())
                            }
                            None => Err(DdsError::Error(
                                "Received message changing state of unknown instance".to_string(),
                            )),
                        }
                    }
                }?;

                tracing::debug!(cache_change = ?sample, "Adding change to data reader history cache");
                self.sample_list.push(sample);
                self.data_available_status_changed_flag = true;

                match self.qos.destination_order.kind {
                    DestinationOrderQosPolicyKind::BySourceTimestamp => {
                        self.sample_list.sort_by(|a, b| {
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
                        .sample_list
                        .sort_by(|a, b| a.reception_timestamp.cmp(&b.reception_timestamp)),
                }

                self.on_data_available(
                    // data_reader_address, subscriber, subscriber_mask_listener
                )?;
            }
        }

        Ok(())
    }

    fn is_sample_of_interest_based_on_time(&self, sample: &ReaderSample) -> bool {
        let closest_timestamp_before_received_sample = self
            .sample_list
            .iter()
            .filter(|cc| cc.instance_handle() == sample.instance_handle())
            .filter(|cc| cc.source_timestamp <= sample.source_timestamp)
            .map(|cc| cc.source_timestamp)
            .max();

        if let Some(Some(t)) = closest_timestamp_before_received_sample {
            if let Some(sample_source_time) = sample.source_timestamp {
                let sample_separation = sample_source_time - t;
                DurationKind::Finite(sample_separation)
                    >= self.qos.time_based_filter.minimum_separation
            } else {
                true
            }
        } else {
            true
        }
    }

    fn is_max_samples_limit_reached(&self, _change: &ReaderSample) -> bool {
        let total_samples = self
            .sample_list
            .iter()
            .filter(|cc| cc.kind == ChangeKind::Alive)
            .count();

        total_samples == self.qos.resource_limits.max_samples
    }

    fn is_max_instances_limit_reached(&self, change: &ReaderSample) -> bool {
        let instance_handle_list: HashSet<_> = self
            .sample_list
            .iter()
            .map(|cc| cc.instance_handle())
            .collect();

        if instance_handle_list.contains(&change.instance_handle()) {
            false
        } else {
            instance_handle_list.len() == self.qos.resource_limits.max_instances
        }
    }

    fn is_max_samples_per_instance_limit_reached(&self, change: &ReaderSample) -> bool {
        let total_samples_of_instance = self
            .sample_list
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
            .sample_list
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
                .update_state(cache_change.kind);
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

            let (data, valid_data) = match cache_change.kind {
                ChangeKind::Alive | ChangeKind::AliveFiltered => {
                    (Some(cache_change.data_value.clone()), true)
                }
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
                publication_handle: InstanceHandle::new(cache_change.writer_guid.into()),
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

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn add_deadline_missed_task(
        &mut self,
        change_instance_handle: InstanceHandle,
        deadline_missed_task: TaskHandle,
    ) {
        if let Some(t) = self
            .instance_deadline_missed_task
            .remove(&change_instance_handle)
        {
            t.abort();
        }

        self.instance_deadline_missed_task
            .insert(change_instance_handle, deadline_missed_task);
    }

    pub fn on_deadline_missed(&mut self, change_instance_handle: InstanceHandle) {
        self.requested_deadline_missed_status.total_count += 1;
        self.requested_deadline_missed_status.total_count_change += 1;
        self.requested_deadline_missed_status.last_instance_handle = change_instance_handle;
        self.remove_instance_ownership(change_instance_handle);
        self.status_condition
            .send_actor_mail(status_condition_actor::AddCommunicationState {
                state: StatusKind::RequestedDeadlineMissed,
            });
    }

    pub fn as_subscription_builtin_topic_data(
        &self,
        subscriber_qos: &SubscriberQos,
        topic_qos: &TopicQos,
    ) -> SubscriptionBuiltinTopicData {
        SubscriptionBuiltinTopicData {
            key: BuiltInTopicKey {
                value: self.transport_reader.guid(),
            },
            participant_key: BuiltInTopicKey { value: [0; 16] },
            topic_name: self.topic_name.clone(),
            type_name: self.type_name.clone(),
            durability: self.qos.durability.clone(),
            deadline: self.qos.deadline.clone(),
            latency_budget: self.qos.latency_budget.clone(),
            liveliness: self.qos.liveliness.clone(),
            reliability: self.qos.reliability.clone(),
            ownership: self.qos.ownership.clone(),
            destination_order: self.qos.destination_order.clone(),
            user_data: self.qos.user_data.clone(),
            time_based_filter: self.qos.time_based_filter.clone(),
            presentation: subscriber_qos.presentation.clone(),
            partition: subscriber_qos.partition.clone(),
            topic_data: topic_qos.topic_data.clone(),
            group_data: subscriber_qos.group_data.clone(),
            representation: self.qos.representation.clone(),
        }
    }

    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
    }

    pub fn add_matched_writer(
        &mut self,
        discovered_writer_data: &DiscoveredWriterData,
        subscriber_qos: &SubscriberQos,
    ) {
        let type_name = self.type_name.clone();
        let topic_name = self.topic_name.clone();
        let publication_builtin_topic_data = &discovered_writer_data.dds_publication_data;
        if publication_builtin_topic_data.topic_name() == topic_name
            && publication_builtin_topic_data.get_type_name() == type_name
        {
            tracing::trace!(
                topic_name = topic_name,
                type_name = type_name,
                "Writer with matched topic and type found",
            );
            let instance_handle =
                InstanceHandle::new(discovered_writer_data.dds_publication_data.key.value);
            let incompatible_qos_policy_list = self
                .get_discovered_writer_incompatible_qos_policy_list(
                    &discovered_writer_data,
                    &subscriber_qos,
                );
            if incompatible_qos_policy_list.is_empty() {
                let insert_matched_publication_result = self
                    .matched_publication_list
                    .insert(instance_handle, publication_builtin_topic_data.clone());
                match insert_matched_publication_result {
                    Some(value) if &value != publication_builtin_topic_data => {
                        self.on_subscription_matched(
                            instance_handle,
                            // message.data_reader_address,
                            // message.subscriber,
                            // &message.subscriber_mask_listener,
                            // &message.participant_mask_listener,
                        );
                    }
                    None => {
                        self.on_subscription_matched(
                            instance_handle,
                            // message.data_reader_address,
                            // message.subscriber,
                            // &message.subscriber_mask_listener,
                            // &message.participant_mask_listener,
                        );
                    }
                    _ => (),
                }
            } else if !self.incompatible_writer_list.contains(&instance_handle) {
                self.incompatible_writer_list.insert(instance_handle);
                self.on_requested_incompatible_qos(
                    incompatible_qos_policy_list,
                    // &message.data_reader_address,
                    // &message.subscriber,
                    // &message.subscriber_mask_listener,
                    // &message.participant_mask_listener,
                );
            }
        }
    }

    pub fn remove_matched_writer(&mut self, discovered_writer_handle: InstanceHandle) {
        let matched_publication = self
            .matched_publication_list
            .remove(&discovered_writer_handle);
        if let Some(w) = matched_publication {
            self.on_subscription_matched(
                discovered_writer_handle,
                // message.data_reader_address,
                // message.subscriber,
                // &message.subscriber_mask_listener,
                // &message.participant_mask_listener,
            );
        }
    }

    pub fn is_historical_data_received(&self) -> DdsResult<bool> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        };

        match self.qos.durability.kind {
            DurabilityQosPolicyKind::Volatile => Err(DdsError::IllegalOperation),
            DurabilityQosPolicyKind::TransientLocal
            | DurabilityQosPolicyKind::Transient
            | DurabilityQosPolicyKind::Persistent => Ok(()),
        }?;

        Ok(self.transport_reader.is_historical_data_received())
    }

    pub fn set_qos(&mut self, qos: DataReaderQos) -> DdsResult<()> {
        qos.is_consistent()?;
        if self.enabled {
            qos.check_immutability(&self.qos)?;
        }
        self.qos = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> &DataReaderQos {
        &self.qos
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

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_matched_publications(&self) -> Vec<InstanceHandle> {
        self.matched_publication_list
            .iter()
            .map(|(&key, _)| key)
            .collect()
    }

    pub fn take_next_instance(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> DdsResult<Vec<(Option<Data>, SampleInfo)>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(previous_handle) {
            Some(next_handle) => self.take(
                max_samples,
                sample_states,
                view_states,
                instance_states,
                Some(next_handle),
            ),
            None => Err(DdsError::NoData),
        }
    }

    pub fn read_next_instance(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<(Option<Data>, SampleInfo)>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(previous_handle) {
            Some(next_handle) => self.read(
                max_samples,
                sample_states,
                view_states,
                instance_states,
                Some(next_handle),
            ),
            None => Err(DdsError::NoData),
        }
    }

    pub fn get_subscription_matched_status(&mut self) -> SubscriptionMatchedStatus {
        self.status_condition
            .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                state: StatusKind::SubscriptionMatched,
            });

        self.subscription_matched_status
            .read_and_reset(self.matched_publication_list.len() as i32)
    }

    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }

    pub fn set_listener(
        &mut self,
        listener: Option<DataReaderActorListener>,
        status_kind: Vec<StatusKind>,
    ) -> DdsResult<()> {
        if let Some(listener) = self.data_reader_listener_thread.take() {
            listener.join()?;
        }
        self.data_reader_listener_thread = listener
            .map(|x| DataReaderListenerThread::new(x.data_reader_listener, x.subscriber_async));
        self.data_reader_status_kind = status_kind;
        Ok(())
    }

    pub fn remove_instance_ownership(&mut self, instance: InstanceHandle) {
        self.instance_ownership.remove(&instance);
    }

    pub fn add_change(&mut self, cache_change: ReaderCacheChange) -> DdsResult<InstanceHandle> {
        let sample = self.convert_cache_change_to_sample(cache_change, Time::now())?;
        let change_instance_handle = sample.instance_handle;
        self.add_sample(sample)?;
        Ok(change_instance_handle)
    }
}
