use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    data_representation_builtin_endpoints::{
        discovered_reader_data::DiscoveredReaderData,
        discovered_writer_data::{DiscoveredWriterData, WriterProxy},
    },
    dds_async::{publisher::PublisherAsync, topic::TopicAsync},
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        data_representation_inline_qos::{
            parameter_id_values::{PID_KEY_HASH, PID_STATUS_INFO},
            types::{
                STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED, STATUS_INFO_UNREGISTERED,
            },
        },
        payload_serializer_deserializer::{
            cdr_serializer::ClassicCdrSerializer, endianness::CdrEndianness,
        },
        runtime::{
            executor::{block_on, ExecutorHandle},
            mpsc::{mpsc_channel, MpscSender},
            timer::TimerHandle,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::{InstanceHandle, HANDLE_NIL},
        qos::{DataWriterQos, PublisherQos},
        qos_policy::{
            DurabilityQosPolicyKind, HistoryQosPolicyKind, QosPolicyId, ReliabilityQosPolicyKind,
            TopicDataQosPolicy, DEADLINE_QOS_POLICY_ID, DESTINATIONORDER_QOS_POLICY_ID,
            DURABILITY_QOS_POLICY_ID, INVALID_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID,
        },
        status::{
            OfferedIncompatibleQosStatus, PublicationMatchedStatus, QosPolicyCount, StatusKind,
        },
        time::{Duration, DurationKind, Time},
    },
    rtps::{
        messages::{
            submessage_elements::{
                Data, Parameter, ParameterList, SequenceNumberSet, SerializedDataFragment,
            },
            submessages::{
                ack_nack::AckNackSubmessage, data_frag::DataFragSubmessage, gap::GapSubmessage,
                info_destination::InfoDestinationSubmessage,
                info_timestamp::InfoTimestampSubmessage, nack_frag::NackFragSubmessage,
            },
        },
        reader_locator::RtpsReaderLocator,
        reader_proxy::RtpsReaderProxy,
        types::{
            ChangeKind, EntityId, Guid, GuidPrefix, Locator, ReliabilityKind, SequenceNumber,
            ENTITYID_UNKNOWN, GUID_UNKNOWN, USER_DEFINED_UNKNOWN,
        },
        writer::RtpsWriter,
        writer_history_cache::{RtpsWriterCacheChange, WriterHistoryCache},
    },
    serialized_payload::cdr::serialize::CdrSerialize,
    topic_definition::type_support::DdsKey,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    thread::JoinHandle,
};

use super::{
    any_data_writer_listener::{AnyDataWriterListener, DataWriterListenerOperation},
    domain_participant_actor::{ParticipantListenerMessage, ParticipantListenerOperation},
    message_sender_actor::{self, MessageSenderActor},
    publisher_actor::{PublisherListenerMessage, PublisherListenerOperation},
    status_condition_actor::{self, AddCommunicationState, StatusConditionActor},
    topic_actor::TopicActor,
};

struct MatchedSubscriptions {
    matched_subscription_list: HashMap<InstanceHandle, SubscriptionBuiltinTopicData>,
    total_count: i32,
    total_count_last_read: i32,
    current_count_last_read: i32,
    last_subscription_handle: InstanceHandle,
}

impl MatchedSubscriptions {
    fn new() -> Self {
        Self {
            matched_subscription_list: HashMap::new(),
            total_count: 0,
            total_count_last_read: 0,
            current_count_last_read: 0,
            last_subscription_handle: HANDLE_NIL,
        }
    }

    fn add_matched_subscription(
        &mut self,
        handle: InstanceHandle,
        subscription_data: SubscriptionBuiltinTopicData,
    ) {
        self.matched_subscription_list
            .insert(handle, subscription_data);
        self.total_count += 1;
        self.last_subscription_handle = handle;
    }

    fn remove_matched_subscription(&mut self, handle: InstanceHandle) {
        self.matched_subscription_list.remove(&handle);
    }

    fn get_matched_subscriptions(&self) -> Vec<InstanceHandle> {
        self.matched_subscription_list
            .iter()
            .map(|(&h, _)| h)
            .collect()
    }

    pub fn get_matched_subscription_data(
        &self,
        handle: InstanceHandle,
    ) -> Option<&SubscriptionBuiltinTopicData> {
        self.matched_subscription_list.get(&handle)
    }

    fn get_publication_matched_status(&mut self) -> PublicationMatchedStatus {
        let current_count = self.matched_subscription_list.len() as i32;
        let status = PublicationMatchedStatus {
            total_count: self.total_count,
            total_count_change: self.total_count - self.total_count_last_read,
            last_subscription_handle: self.last_subscription_handle,
            current_count,
            current_count_change: current_count - self.current_count_last_read,
        };

        self.total_count_last_read = self.total_count;
        self.current_count_last_read = current_count;

        status
    }
}

struct IncompatibleSubscriptions {
    incompatible_subscription_list: HashSet<InstanceHandle>,
    total_count: i32,
    total_count_last_read: i32,
    last_policy_id: QosPolicyId,
    policies: Vec<QosPolicyCount>,
}

impl IncompatibleSubscriptions {
    fn new() -> Self {
        Self {
            incompatible_subscription_list: HashSet::new(),
            total_count: 0,
            total_count_last_read: 0,
            last_policy_id: INVALID_QOS_POLICY_ID,
            policies: Vec::new(),
        }
    }

    fn add_offered_incompatible_qos(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        self.total_count += 1;
        self.last_policy_id = incompatible_qos_policy_list[0];

        self.incompatible_subscription_list.insert(handle);
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

    fn get_incompatible_subscriptions(&self) -> Vec<InstanceHandle> {
        self.incompatible_subscription_list
            .iter()
            .cloned()
            .collect()
    }

    fn get_offered_incompatible_qos_status(&mut self) -> OfferedIncompatibleQosStatus {
        let status = OfferedIncompatibleQosStatus {
            total_count: self.total_count,
            total_count_change: self.total_count - self.total_count_last_read,
            last_policy_id: self.last_policy_id,
            policies: self.policies.clone(),
        };

        self.total_count_last_read = self.total_count;

        status
    }

    fn contains(&self, handle: &InstanceHandle) -> bool {
        self.incompatible_subscription_list.contains(handle)
    }
}

struct DataWriterListenerMessage {
    listener_operation: DataWriterListenerOperation,
    writer_address: ActorAddress<DataWriterActor>,
    status_condition_address: ActorAddress<StatusConditionActor>,
    publisher: PublisherAsync,
    topic: TopicAsync,
}

struct DataWriterListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<DataWriterListenerMessage>,
}

impl DataWriterListenerThread {
    fn new(mut listener: Box<dyn AnyDataWriterListener + Send>) -> Self {
        let (sender, receiver) = mpsc_channel::<DataWriterListenerMessage>();
        let thread = std::thread::spawn(move || {
            block_on(async {
                while let Some(m) = receiver.recv().await {
                    listener
                        .call_listener_function(
                            m.listener_operation,
                            m.writer_address,
                            m.status_condition_address,
                            m.publisher,
                            m.topic,
                        )
                        .await;
                }
            });
        });
        Self { thread, sender }
    }

    fn sender(&self) -> &MpscSender<DataWriterListenerMessage> {
        &self.sender
    }

    fn join(self) -> DdsResult<()> {
        self.sender.close();
        self.thread.join()?;
        Ok(())
    }
}

pub struct DataWriterActor {
    rtps_writer: RtpsWriter,
    reader_locators: Vec<RtpsReaderLocator>,
    matched_readers: Vec<RtpsReaderProxy>,
    topic_address: ActorAddress<TopicActor>,
    topic_name: String,
    type_name: String,
    topic_status_condition: ActorAddress<StatusConditionActor>,
    matched_subscriptions: MatchedSubscriptions,
    incompatible_subscriptions: IncompatibleSubscriptions,
    enabled: bool,
    status_condition: Actor<StatusConditionActor>,
    data_writer_listener_thread: Option<DataWriterListenerThread>,
    status_kind: Vec<StatusKind>,
    writer_cache: WriterHistoryCache,
    qos: DataWriterQos,
    registered_instance_list: HashSet<InstanceHandle>,
}

impl DataWriterActor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rtps_writer: RtpsWriter,
        topic_address: ActorAddress<TopicActor>,
        topic_name: String,
        type_name: String,
        topic_status_condition: ActorAddress<StatusConditionActor>,
        listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        status_kind: Vec<StatusKind>,
        qos: DataWriterQos,
        handle: &ExecutorHandle,
    ) -> Self {
        let status_condition = Actor::spawn(StatusConditionActor::default(), handle);
        let data_writer_listener_thread = listener.map(DataWriterListenerThread::new);

        let max_changes = match qos.history.kind {
            HistoryQosPolicyKind::KeepLast(keep_last) => Some(keep_last),
            HistoryQosPolicyKind::KeepAll => None,
        };
        DataWriterActor {
            rtps_writer,
            reader_locators: Vec::new(),
            matched_readers: Vec::new(),
            topic_address,
            topic_name,
            type_name,
            topic_status_condition,
            matched_subscriptions: MatchedSubscriptions::new(),
            incompatible_subscriptions: IncompatibleSubscriptions::new(),
            enabled: false,
            status_condition,
            data_writer_listener_thread,
            status_kind,
            writer_cache: WriterHistoryCache::new(max_changes),
            qos,
            registered_instance_list: HashSet::new(),
        }
    }

    pub fn reader_locator_add(&mut self, a_locator: RtpsReaderLocator) {
        let mut locator = a_locator;
        if let Some(highest_available_change_sn) = self.writer_cache.get_seq_num_max() {
            locator.set_highest_sent_change_sn(highest_available_change_sn)
        }

        self.reader_locators.push(locator);
    }

    fn add_change(
        &mut self,
        change: RtpsWriterCacheChange,
        message_sender_actor: ActorAddress<MessageSenderActor>,
        now: Time,
        writer_address: ActorAddress<DataWriterActor>,
        executor_handle: ExecutorHandle,
        timer_handle: TimerHandle,
    ) {
        let seq_num = change.sequence_number();

        if let DurationKind::Finite(lifespan) = self.qos.lifespan.duration {
            let change_lifespan =
                (crate::infrastructure::time::Time::from(change.timestamp()) - now) + lifespan;
            if change_lifespan > Duration::new(0, 0) {
                self.writer_cache.add_change(change);

                executor_handle.spawn(async move {
                    timer_handle.sleep(change_lifespan.into()).await;

                    writer_address
                        .send_actor_mail(RemoveChange { seq_num })
                        .ok();
                });
            }
        } else {
            self.writer_cache.add_change(change);
        }

        self.send_message(message_sender_actor);
    }

    fn remove_change(&mut self, seq_num: SequenceNumber) {
        self.writer_cache
            .remove_change(|cc| cc.sequence_number() == seq_num)
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        InstanceHandle::new(self.rtps_writer.guid().into())
    }

    fn send_message(&mut self, message_sender_actor: ActorAddress<MessageSenderActor>) {
        self.send_message_to_reader_locators(&message_sender_actor);
        self.send_message_to_reader_proxies(&message_sender_actor);
    }

    fn matched_reader_remove(&mut self, a_reader_guid: Guid) {
        self.matched_readers
            .retain(|x| x.remote_reader_guid() != a_reader_guid)
    }

    fn register_instance_w_timestamp(
        &mut self,
        instance_handle: InstanceHandle,
        _timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        if !self.registered_instance_list.contains(&instance_handle) {
            if self.registered_instance_list.len() < self.qos.resource_limits.max_instances {
                self.registered_instance_list.insert(instance_handle);
            } else {
                return Err(DdsError::OutOfResources);
            }
        }
        Ok(Some(instance_handle))
    }

    fn get_publication_matched_status(&mut self) -> PublicationMatchedStatus {
        self.status_condition
            .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                state: StatusKind::PublicationMatched,
            });

        self.matched_subscriptions.get_publication_matched_status()
    }

    fn on_acknack_submessage_received(
        &mut self,
        acknack_submessage: &AckNackSubmessage,
        source_guid_prefix: GuidPrefix,
        message_sender_actor: ActorAddress<MessageSenderActor>,
    ) {
        if self.qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let reader_guid = Guid::new(source_guid_prefix, *acknack_submessage.reader_id());

            if let Some(reader_proxy) = self
                .matched_readers
                .iter_mut()
                .find(|x| x.remote_reader_guid() == reader_guid)
            {
                match reader_proxy.reliability() {
                    ReliabilityKind::BestEffort => (),
                    ReliabilityKind::Reliable => {
                        if acknack_submessage.count() > reader_proxy.last_received_acknack_count() {
                            reader_proxy
                                .acked_changes_set(acknack_submessage.reader_sn_state().base() - 1);
                            reader_proxy
                                .requested_changes_set(acknack_submessage.reader_sn_state().set());

                            reader_proxy
                                .set_last_received_acknack_count(acknack_submessage.count());

                            self.send_message(message_sender_actor);
                        }
                    }
                }
            }
        }
    }

    fn send_message_to_reader_locators(
        &mut self,
        message_sender_actor: &ActorAddress<MessageSenderActor>,
    ) {
        for reader_locator in &mut self.reader_locators {
            match &self.qos.reliability.kind {
                ReliabilityQosPolicyKind::BestEffort => {
                    while let Some(unsent_change_seq_num) =
                        reader_locator.next_unsent_change(&self.writer_cache)
                    {
                        // The post-condition:
                        // "( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
                        // should be full-filled by next_unsent_change()

                        if let Some(cache_change) = self
                            .writer_cache
                            .change_list()
                            .find(|cc| cc.sequence_number() == unsent_change_seq_num)
                        {
                            let info_ts_submessage = Box::new(InfoTimestampSubmessage::new(
                                false,
                                cache_change.timestamp(),
                            ));
                            let data_submessage =
                                Box::new(cache_change.as_data_submessage(ENTITYID_UNKNOWN));

                            message_sender_actor
                                .send_actor_mail(message_sender_actor::WriteMessage {
                                    submessages: vec![info_ts_submessage, data_submessage],
                                    destination_locator_list: vec![reader_locator.locator()],
                                })
                                .ok();
                        } else {
                            let gap_submessage = Box::new(GapSubmessage::new(
                                ENTITYID_UNKNOWN,
                                self.rtps_writer.guid().entity_id(),
                                unsent_change_seq_num,
                                SequenceNumberSet::new(unsent_change_seq_num + 1, []),
                            ));

                            message_sender_actor
                                .send_actor_mail(message_sender_actor::WriteMessage {
                                    submessages: vec![gap_submessage],
                                    destination_locator_list: vec![reader_locator.locator()],
                                })
                                .ok();
                        }
                        reader_locator.set_highest_sent_change_sn(unsent_change_seq_num);
                    }
                }
                ReliabilityQosPolicyKind::Reliable => {
                    unimplemented!("Reliable messages to reader locators not implemented")
                }
            }
        }
    }

    fn send_message_to_reader_proxies(
        &mut self,
        message_sender_actor: &ActorAddress<MessageSenderActor>,
    ) {
        for reader_proxy in &mut self.matched_readers {
            match (&self.qos.reliability.kind, reader_proxy.reliability()) {
                (ReliabilityQosPolicyKind::BestEffort, ReliabilityKind::BestEffort)
                | (ReliabilityQosPolicyKind::Reliable, ReliabilityKind::BestEffort) => {
                    send_message_to_reader_proxy_best_effort(
                        reader_proxy,
                        self.rtps_writer.guid().entity_id(),
                        &self.writer_cache,
                        self.rtps_writer.data_max_size_serialized(),
                        message_sender_actor,
                    )
                }
                (ReliabilityQosPolicyKind::Reliable, ReliabilityKind::Reliable) => {
                    send_message_to_reader_proxy_reliable(
                        reader_proxy,
                        self.rtps_writer.guid().entity_id(),
                        &self.writer_cache,
                        self.rtps_writer.data_max_size_serialized(),
                        self.rtps_writer.heartbeat_period().into(),
                        message_sender_actor,
                    )
                }
                (ReliabilityQosPolicyKind::BestEffort, ReliabilityKind::Reliable) => {
                    panic!("Impossible combination. Should not be matched")
                }
            }
        }
    }

    fn on_nack_frag_submessage_received(
        &mut self,
        nackfrag_submessage: &NackFragSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let reader_guid = Guid::new(source_guid_prefix, nackfrag_submessage.reader_id());

            if let Some(reader_proxy) = self
                .matched_readers
                .iter_mut()
                .find(|x| x.remote_reader_guid() == reader_guid)
            {
                match reader_proxy.reliability() {
                    ReliabilityKind::BestEffort => (),
                    ReliabilityKind::Reliable => {
                        if nackfrag_submessage.count()
                            > reader_proxy.last_received_nack_frag_count()
                        {
                            reader_proxy.requested_changes_set(std::iter::once(
                                nackfrag_submessage.writer_sn(),
                            ));
                            reader_proxy
                                .set_last_received_nack_frag_count(nackfrag_submessage.count());
                        }
                    }
                }
            }
        }
    }

    fn on_publication_matched(
        &mut self,
        data_writer_address: ActorAddress<DataWriterActor>,
        publisher: PublisherAsync,
        (publisher_listener, publisher_listener_mask): (
            Option<MpscSender<PublisherListenerMessage>>,
            Vec<StatusKind>,
        ),
        (participant_listener, participant_listener_mask): (
            Option<MpscSender<ParticipantListenerMessage>>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        self.status_condition
            .send_actor_mail(AddCommunicationState {
                state: StatusKind::PublicationMatched,
            });

        if self.status_kind.contains(&StatusKind::PublicationMatched) {
            let type_name = self.type_name.clone();
            let topic_name = self.topic_name.clone();
            let status = self.get_publication_matched_status();
            let participant = publisher.get_participant();
            let status_condition_address = self.status_condition.address();
            let topic_status_condition_address = self.topic_status_condition.clone();
            let topic = TopicAsync::new(
                self.topic_address.clone(),
                topic_status_condition_address,
                type_name,
                topic_name,
                participant,
            );
            if let Some(listener) = &self.data_writer_listener_thread {
                listener.sender().send(DataWriterListenerMessage {
                    listener_operation: DataWriterListenerOperation::PublicationMatched(status),
                    writer_address: data_writer_address,
                    status_condition_address,
                    publisher,
                    topic,
                })?;
            }
        } else if publisher_listener_mask.contains(&StatusKind::PublicationMatched) {
            let status = self.get_publication_matched_status();
            if let Some(listener) = publisher_listener {
                listener.send(PublisherListenerMessage {
                    listener_operation: PublisherListenerOperation::PublicationMatched(status),
                })?;
            }
        } else if participant_listener_mask.contains(&StatusKind::PublicationMatched) {
            let status = self.get_publication_matched_status();
            if let Some(listener) = participant_listener {
                listener.send(ParticipantListenerMessage {
                    listener_operation: ParticipantListenerOperation::PublicationMatched(status),
                })?;
            }
        }
        Ok(())
    }

    fn on_offered_incompatible_qos(
        &mut self,
        data_writer_address: ActorAddress<DataWriterActor>,
        publisher: PublisherAsync,
        (publisher_listener, publisher_listener_mask): (
            Option<MpscSender<PublisherListenerMessage>>,
            Vec<StatusKind>,
        ),
        (participant_listener, participant_listener_mask): (
            Option<MpscSender<ParticipantListenerMessage>>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        self.status_condition
            .send_actor_mail(AddCommunicationState {
                state: StatusKind::OfferedIncompatibleQos,
            });

        if self
            .status_kind
            .contains(&StatusKind::OfferedIncompatibleQos)
        {
            let type_name = self.type_name.clone();
            let topic_name = self.topic_name.clone();
            let status = self
                .incompatible_subscriptions
                .get_offered_incompatible_qos_status();
            let participant = publisher.get_participant();
            let status_condition_address = self.status_condition.address();
            let topic_status_condition_address = self.topic_status_condition.clone();
            let topic = TopicAsync::new(
                self.topic_address.clone(),
                topic_status_condition_address,
                type_name,
                topic_name,
                participant,
            );

            if let Some(listener) = &self.data_writer_listener_thread {
                listener.sender().send(DataWriterListenerMessage {
                    listener_operation: DataWriterListenerOperation::OfferedIncompatibleQos(status),
                    writer_address: data_writer_address,
                    status_condition_address,
                    publisher,
                    topic,
                })?;
            }
        } else if publisher_listener_mask.contains(&StatusKind::OfferedIncompatibleQos) {
            let status = self
                .incompatible_subscriptions
                .get_offered_incompatible_qos_status();
            if let Some(listener) = publisher_listener {
                listener.send(PublisherListenerMessage {
                    listener_operation: PublisherListenerOperation::OfferedIncompatibleQos(status),
                })?;
            }
        } else if participant_listener_mask.contains(&StatusKind::OfferedIncompatibleQos) {
            let status = self
                .incompatible_subscriptions
                .get_offered_incompatible_qos_status();
            if let Some(listener) = participant_listener {
                listener.send(ParticipantListenerMessage {
                    listener_operation: ParticipantListenerOperation::OfferedIncompatibleQos(
                        status,
                    ),
                })?;
            }
        }
        Ok(())
    }
}

pub struct GetInstanceHandle;
impl Mail for GetInstanceHandle {
    type Result = InstanceHandle;
}
impl MailHandler<GetInstanceHandle> for DataWriterActor {
    fn handle(&mut self, _: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
        self.get_instance_handle()
    }
}

pub struct AddMatchedPublication {
    pub handle: InstanceHandle,
    pub subscription_data: SubscriptionBuiltinTopicData,
}
impl Mail for AddMatchedPublication {
    type Result = ();
}
impl MailHandler<AddMatchedPublication> for DataWriterActor {
    fn handle(
        &mut self,
        message: AddMatchedPublication,
    ) -> <AddMatchedPublication as Mail>::Result {
        self.matched_subscriptions
            .add_matched_subscription(message.handle, message.subscription_data);
    }
}

pub struct RemoveMatchedSubscription {
    pub handle: InstanceHandle,
}
impl Mail for RemoveMatchedSubscription {
    type Result = ();
}
impl MailHandler<RemoveMatchedSubscription> for DataWriterActor {
    fn handle(
        &mut self,
        message: RemoveMatchedSubscription,
    ) -> <RemoveMatchedSubscription as Mail>::Result {
        self.matched_subscriptions
            .remove_matched_subscription(message.handle)
    }
}

pub struct GetMatchedSubscriptions;
impl Mail for GetMatchedSubscriptions {
    type Result = Vec<InstanceHandle>;
}
impl MailHandler<GetMatchedSubscriptions> for DataWriterActor {
    fn handle(&mut self, _: GetMatchedSubscriptions) -> <GetMatchedSubscriptions as Mail>::Result {
        self.matched_subscriptions.get_matched_subscriptions()
    }
}

pub struct GetMatchedSubscriptionData {
    pub handle: InstanceHandle,
}
impl Mail for GetMatchedSubscriptionData {
    type Result = Option<SubscriptionBuiltinTopicData>;
}
impl MailHandler<GetMatchedSubscriptionData> for DataWriterActor {
    fn handle(
        &mut self,
        message: GetMatchedSubscriptionData,
    ) -> <GetMatchedSubscriptionData as Mail>::Result {
        self.matched_subscriptions
            .get_matched_subscription_data(message.handle)
            .cloned()
    }
}

pub struct GetOfferedIncompatibleQosStatus;
impl Mail for GetOfferedIncompatibleQosStatus {
    type Result = OfferedIncompatibleQosStatus;
}
impl MailHandler<GetOfferedIncompatibleQosStatus> for DataWriterActor {
    fn handle(
        &mut self,
        _: GetOfferedIncompatibleQosStatus,
    ) -> <GetOfferedIncompatibleQosStatus as Mail>::Result {
        self.incompatible_subscriptions
            .get_offered_incompatible_qos_status()
    }
}

pub struct GetIncompatibleSubscriptions;
impl Mail for GetIncompatibleSubscriptions {
    type Result = Vec<InstanceHandle>;
}
impl MailHandler<GetIncompatibleSubscriptions> for DataWriterActor {
    fn handle(
        &mut self,
        _: GetIncompatibleSubscriptions,
    ) -> <GetIncompatibleSubscriptions as Mail>::Result {
        self.incompatible_subscriptions
            .get_incompatible_subscriptions()
    }
}

pub struct Enable {
    pub data_writer_address: ActorAddress<DataWriterActor>,
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
    pub executor_handle: ExecutorHandle,
    pub timer_handle: TimerHandle,
}
impl Mail for Enable {
    type Result = ();
}
impl MailHandler<Enable> for DataWriterActor {
    fn handle(&mut self, message: Enable) -> <Enable as Mail>::Result {
        self.enabled = true;

        if self.qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let half_heartbeat_period =
                std::time::Duration::from(Duration::from(self.rtps_writer.heartbeat_period())) / 2;
            let message_sender_actor = message.message_sender_actor;
            let data_writer_address = message.data_writer_address;
            let timer_handle = message.timer_handle;
            message.executor_handle.spawn(async move {
                loop {
                    timer_handle.sleep(half_heartbeat_period).await;

                    let r = data_writer_address.send_actor_mail(SendMessage {
                        message_sender_actor: message_sender_actor.clone(),
                    });
                    if r.is_err() {
                        break;
                    }
                }
            });
        }
    }
}

pub struct IsEnabled;
impl Mail for IsEnabled {
    type Result = bool;
}
impl MailHandler<IsEnabled> for DataWriterActor {
    fn handle(&mut self, _: IsEnabled) -> <IsEnabled as Mail>::Result {
        self.enabled
    }
}

pub struct GetStatuscondition;
impl Mail for GetStatuscondition {
    type Result = ActorAddress<StatusConditionActor>;
}
impl MailHandler<GetStatuscondition> for DataWriterActor {
    fn handle(&mut self, _: GetStatuscondition) -> <GetStatuscondition as Mail>::Result {
        self.status_condition.address()
    }
}

pub struct GetGuid;
impl Mail for GetGuid {
    type Result = Guid;
}
impl MailHandler<GetGuid> for DataWriterActor {
    fn handle(&mut self, _: GetGuid) -> <GetGuid as Mail>::Result {
        self.rtps_writer.guid()
    }
}

pub struct GetQos;
impl Mail for GetQos {
    type Result = DataWriterQos;
}
impl MailHandler<GetQos> for DataWriterActor {
    fn handle(&mut self, _: GetQos) -> <GetQos as Mail>::Result {
        self.qos.clone()
    }
}

pub struct SetQos {
    pub qos: DataWriterQos,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for DataWriterActor {
    fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        message.qos.is_consistent()?;
        if self.enabled {
            message.qos.check_immutability(&self.qos)?;
        }
        self.qos = message.qos;
        Ok(())
    }
}

pub struct RegisterInstanceWTimestamp {
    pub instance_handle: InstanceHandle,
    pub timestamp: Time,
}
impl Mail for RegisterInstanceWTimestamp {
    type Result = DdsResult<Option<InstanceHandle>>;
}
impl MailHandler<RegisterInstanceWTimestamp> for DataWriterActor {
    fn handle(
        &mut self,
        message: RegisterInstanceWTimestamp,
    ) -> <RegisterInstanceWTimestamp as Mail>::Result {
        self.register_instance_w_timestamp(message.instance_handle, message.timestamp)
    }
}

pub struct UnregisterInstanceWTimestamp {
    pub instance_serialized_key: Data,
    pub handle: InstanceHandle,
    pub timestamp: Time,
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
    pub now: Time,
    pub data_writer_address: ActorAddress<DataWriterActor>,
    pub executor_handle: ExecutorHandle,
    pub timer_handle: TimerHandle,
}
impl Mail for UnregisterInstanceWTimestamp {
    type Result = DdsResult<()>;
}
impl MailHandler<UnregisterInstanceWTimestamp> for DataWriterActor {
    fn handle(
        &mut self,
        message: UnregisterInstanceWTimestamp,
    ) -> <UnregisterInstanceWTimestamp as Mail>::Result {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let mut serialized_status_info = Vec::new();
        let mut serializer =
            ClassicCdrSerializer::new(&mut serialized_status_info, CdrEndianness::LittleEndian);
        if self
            .qos
            .writer_data_lifecycle
            .autodispose_unregistered_instances
        {
            STATUS_INFO_DISPOSED_UNREGISTERED
                .serialize(&mut serializer)
                .unwrap();
        } else {
            STATUS_INFO_UNREGISTERED.serialize(&mut serializer).unwrap();
        }
        let pid_status_info = Parameter::new(PID_STATUS_INFO, Arc::from(serialized_status_info));
        let pid_key_hash = Parameter::new(PID_KEY_HASH, Arc::from(*message.handle.as_ref()));
        let inline_qos = ParameterList::new(vec![pid_status_info, pid_key_hash]);

        let change: RtpsWriterCacheChange = self.rtps_writer.new_change(
            ChangeKind::NotAliveUnregistered,
            message.instance_serialized_key,
            inline_qos,
            message.handle.into(),
            message.timestamp.into(),
        );

        self.add_change(
            change,
            message.message_sender_actor,
            message.now,
            message.data_writer_address,
            message.executor_handle,
            message.timer_handle,
        );
        Ok(())
    }
}

pub struct LookupInstance {
    pub instance_handle: InstanceHandle,
}
impl Mail for LookupInstance {
    type Result = DdsResult<Option<InstanceHandle>>;
}
impl MailHandler<LookupInstance> for DataWriterActor {
    fn handle(&mut self, message: LookupInstance) -> <LookupInstance as Mail>::Result {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        Ok(
            if self
                .registered_instance_list
                .contains(&message.instance_handle)
            {
                Some(message.instance_handle)
            } else {
                None
            },
        )
    }
}

pub struct DisposeWTimestamp {
    pub instance_serialized_key: Data,
    pub handle: InstanceHandle,
    pub timestamp: Time,
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
    pub now: Time,
    pub data_writer_address: ActorAddress<DataWriterActor>,
    pub executor_handle: ExecutorHandle,
    pub timer_handle: TimerHandle,
}
impl Mail for DisposeWTimestamp {
    type Result = DdsResult<()>;
}
impl MailHandler<DisposeWTimestamp> for DataWriterActor {
    fn handle(&mut self, message: DisposeWTimestamp) -> <DisposeWTimestamp as Mail>::Result {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let mut serialized_status_info = Vec::new();
        let mut serializer =
            ClassicCdrSerializer::new(&mut serialized_status_info, CdrEndianness::LittleEndian);
        STATUS_INFO_DISPOSED.serialize(&mut serializer).unwrap();

        let pid_status_info = Parameter::new(PID_STATUS_INFO, Arc::from(serialized_status_info));
        let pid_key_hash = Parameter::new(PID_KEY_HASH, Arc::from(*message.handle.as_ref()));
        let inline_qos = ParameterList::new(vec![pid_status_info, pid_key_hash]);

        let change: RtpsWriterCacheChange = self.rtps_writer.new_change(
            ChangeKind::NotAliveDisposed,
            message.instance_serialized_key,
            inline_qos,
            message.handle.into(),
            message.timestamp.into(),
        );

        self.add_change(
            change,
            message.message_sender_actor,
            message.now,
            message.data_writer_address,
            message.executor_handle,
            message.timer_handle,
        );

        Ok(())
    }
}

pub struct AreAllChangesAcknowledge;
impl Mail for AreAllChangesAcknowledge {
    type Result = bool;
}
impl MailHandler<AreAllChangesAcknowledge> for DataWriterActor {
    fn handle(
        &mut self,
        _: AreAllChangesAcknowledge,
    ) -> <AreAllChangesAcknowledge as Mail>::Result {
        !self
            .matched_readers
            .iter()
            .any(|rp| rp.unacked_changes(&self.writer_cache))
    }
}

pub struct AsDiscoveredWriterData {
    pub publisher_qos: PublisherQos,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub topic_data: TopicDataQosPolicy,
    pub xml_type: String,
}
impl Mail for AsDiscoveredWriterData {
    type Result = DdsResult<DiscoveredWriterData>;
}
impl MailHandler<AsDiscoveredWriterData> for DataWriterActor {
    fn handle(
        &mut self,
        message: AsDiscoveredWriterData,
    ) -> <AsDiscoveredWriterData as Mail>::Result {
        let type_name = self.type_name.clone();
        let topic_name = self.topic_name.clone();
        let writer_qos = &self.qos;

        let unicast_locator_list = if self.rtps_writer.unicast_locator_list().is_empty() {
            message.default_unicast_locator_list
        } else {
            self.rtps_writer.unicast_locator_list().to_vec()
        };

        let multicast_locator_list = if self.rtps_writer.unicast_locator_list().is_empty() {
            message.default_multicast_locator_list
        } else {
            self.rtps_writer.multicast_locator_list().to_vec()
        };

        Ok(DiscoveredWriterData::new(
            PublicationBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: self.rtps_writer.guid().into(),
                },
                BuiltInTopicKey {
                    value: GUID_UNKNOWN.into(),
                },
                topic_name,
                type_name,
                writer_qos.clone(),
                message.publisher_qos.clone(),
                message.topic_data,
                message.xml_type,
            ),
            WriterProxy::new(
                self.rtps_writer.guid(),
                EntityId::new([0; 3], USER_DEFINED_UNKNOWN),
                unicast_locator_list,
                multicast_locator_list,
                None,
            ),
        ))
    }
}

pub struct GetPublicationMatchedStatus;
impl Mail for GetPublicationMatchedStatus {
    type Result = PublicationMatchedStatus;
}
impl MailHandler<GetPublicationMatchedStatus> for DataWriterActor {
    fn handle(
        &mut self,
        _: GetPublicationMatchedStatus,
    ) -> <GetPublicationMatchedStatus as Mail>::Result {
        self.get_publication_matched_status()
    }
}

pub struct GetTopicName;
impl Mail for GetTopicName {
    type Result = DdsResult<String>;
}
impl MailHandler<GetTopicName> for DataWriterActor {
    fn handle(&mut self, _: GetTopicName) -> <GetTopicName as Mail>::Result {
        Ok(self.topic_name.clone())
    }
}

pub struct WriteWTimestamp {
    pub serialized_data: Data,
    pub instance_handle: InstanceHandle,
    pub _handle: Option<InstanceHandle>,
    pub timestamp: Time,
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
    pub now: Time,
    pub data_writer_address: ActorAddress<DataWriterActor>,
    pub executor_handle: ExecutorHandle,
    pub timer_handle: TimerHandle,
}
impl Mail for WriteWTimestamp {
    type Result = DdsResult<()>;
}
impl MailHandler<WriteWTimestamp> for DataWriterActor {
    fn handle(&mut self, message: WriteWTimestamp) -> <WriteWTimestamp as Mail>::Result {
        let handle = self
            .register_instance_w_timestamp(message.instance_handle, message.timestamp)?
            .unwrap_or(HANDLE_NIL);

        let pid_key_hash = Parameter::new(PID_KEY_HASH, Arc::from(*handle.as_ref()));
        let parameter_list = ParameterList::new(vec![pid_key_hash]);
        let change = self.rtps_writer.new_change(
            ChangeKind::Alive,
            message.serialized_data,
            parameter_list,
            handle.into(),
            message.timestamp.into(),
        );

        self.add_change(
            change,
            message.message_sender_actor,
            message.now,
            message.data_writer_address,
            message.executor_handle,
            message.timer_handle,
        );

        Ok(())
    }
}

pub struct GetTypeName;
impl Mail for GetTypeName {
    type Result = DdsResult<String>;
}
impl MailHandler<GetTypeName> for DataWriterActor {
    fn handle(&mut self, _: GetTypeName) -> <GetTypeName as Mail>::Result {
        Ok(self.type_name.clone())
    }
}

pub struct AddMatchedReader {
    pub discovered_reader_data: DiscoveredReaderData,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub data_writer_address: ActorAddress<DataWriterActor>,
    pub publisher: PublisherAsync,
    pub publisher_qos: PublisherQos,
    pub publisher_mask_listener: (
        Option<MpscSender<PublisherListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
}
impl Mail for AddMatchedReader {
    type Result = DdsResult<()>;
}
impl MailHandler<AddMatchedReader> for DataWriterActor {
    fn handle(&mut self, message: AddMatchedReader) -> <AddMatchedReader as Mail>::Result {
        let type_name = self.type_name.clone();
        let topic_name = self.topic_name.clone();
        let is_matched_topic_name = message
            .discovered_reader_data
            .subscription_builtin_topic_data()
            .topic_name()
            == topic_name;
        let is_matched_type_name = message
            .discovered_reader_data
            .subscription_builtin_topic_data()
            .get_type_name()
            == type_name;

        if is_matched_topic_name && is_matched_type_name {
            tracing::trace!(
                topic_name = topic_name,
                type_name = type_name,
                "Reader with matched topic and type found",
            );
            let incompatible_qos_policy_list = get_discovered_reader_incompatible_qos_policy_list(
                &self.qos,
                message
                    .discovered_reader_data
                    .subscription_builtin_topic_data(),
                &message.publisher_qos,
            );
            let instance_handle =
                InstanceHandle::try_from_key(&message.discovered_reader_data.get_key().unwrap())
                    .unwrap();

            if incompatible_qos_policy_list.is_empty() {
                let unicast_locator_list = if message
                    .discovered_reader_data
                    .reader_proxy()
                    .unicast_locator_list()
                    .is_empty()
                {
                    message.default_unicast_locator_list
                } else {
                    message
                        .discovered_reader_data
                        .reader_proxy()
                        .unicast_locator_list()
                        .to_vec()
                };

                let multicast_locator_list = if message
                    .discovered_reader_data
                    .reader_proxy()
                    .multicast_locator_list()
                    .is_empty()
                {
                    message.default_multicast_locator_list
                } else {
                    message
                        .discovered_reader_data
                        .reader_proxy()
                        .multicast_locator_list()
                        .to_vec()
                };

                let proxy_reliability = match message
                    .discovered_reader_data
                    .subscription_builtin_topic_data()
                    .reliability()
                    .kind
                {
                    ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                    ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                };

                let first_relevant_sample_seq_num = match message
                    .discovered_reader_data
                    .subscription_builtin_topic_data()
                    .durability()
                    .kind
                {
                    DurabilityQosPolicyKind::Volatile => {
                        self.writer_cache.get_seq_num_max().unwrap_or(0)
                    }
                    DurabilityQosPolicyKind::TransientLocal => 0,
                };

                let reader_proxy = RtpsReaderProxy::new(
                    message
                        .discovered_reader_data
                        .reader_proxy()
                        .remote_reader_guid(),
                    message
                        .discovered_reader_data
                        .reader_proxy()
                        .remote_group_entity_id(),
                    &unicast_locator_list,
                    &multicast_locator_list,
                    message
                        .discovered_reader_data
                        .reader_proxy()
                        .expects_inline_qos(),
                    true,
                    proxy_reliability,
                    first_relevant_sample_seq_num,
                );

                if !self
                    .matched_readers
                    .iter()
                    .any(|x| x.remote_reader_guid() == reader_proxy.remote_reader_guid())
                {
                    self.matched_readers.push(reader_proxy)
                }

                if !self
                    .matched_subscriptions
                    .get_matched_subscriptions()
                    .contains(&instance_handle)
                    || self
                        .matched_subscriptions
                        .get_matched_subscription_data(instance_handle)
                        != Some(
                            message
                                .discovered_reader_data
                                .subscription_builtin_topic_data(),
                        )
                {
                    self.matched_subscriptions.add_matched_subscription(
                        instance_handle,
                        message
                            .discovered_reader_data
                            .subscription_builtin_topic_data()
                            .clone(),
                    );
                    self.on_publication_matched(
                        message.data_writer_address,
                        message.publisher,
                        message.publisher_mask_listener,
                        message.participant_mask_listener,
                    )?;
                }

                self.send_message(message.message_sender_actor);
            } else if !self.incompatible_subscriptions.contains(&instance_handle) {
                self.incompatible_subscriptions
                    .add_offered_incompatible_qos(instance_handle, incompatible_qos_policy_list);
                self.on_offered_incompatible_qos(
                    message.data_writer_address,
                    message.publisher,
                    message.publisher_mask_listener,
                    message.participant_mask_listener,
                )?;
            }
        }
        Ok(())
    }
}

pub struct RemoveMatchedReader {
    pub discovered_reader_handle: InstanceHandle,
    pub data_writer_address: ActorAddress<DataWriterActor>,
    pub publisher: PublisherAsync,
    pub publisher_mask_listener: (
        Option<MpscSender<PublisherListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
}
impl Mail for RemoveMatchedReader {
    type Result = DdsResult<()>;
}
impl MailHandler<RemoveMatchedReader> for DataWriterActor {
    fn handle(&mut self, message: RemoveMatchedReader) -> <RemoveMatchedReader as Mail>::Result {
        if let Some(r) = self
            .matched_subscriptions
            .get_matched_subscription_data(message.discovered_reader_handle)
        {
            let handle = r.key().value.into();
            self.matched_reader_remove(handle);
            self.matched_subscriptions
                .remove_matched_subscription(InstanceHandle::new(handle.into()));

            self.on_publication_matched(
                message.data_writer_address,
                message.publisher,
                message.publisher_mask_listener,
                message.participant_mask_listener,
            )?;
        }

        Ok(())
    }
}

pub struct ProcessAckNackSubmessage {
    pub acknack_submessage: AckNackSubmessage,
    pub source_guid_prefix: GuidPrefix,
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
}
impl Mail for ProcessAckNackSubmessage {
    type Result = ();
}
impl MailHandler<ProcessAckNackSubmessage> for DataWriterActor {
    fn handle(
        &mut self,
        message: ProcessAckNackSubmessage,
    ) -> <ProcessAckNackSubmessage as Mail>::Result {
        self.on_acknack_submessage_received(
            &message.acknack_submessage,
            message.source_guid_prefix,
            message.message_sender_actor,
        )
    }
}

pub struct ProcessNackFragSubmessage {
    pub nackfrag_submessage: NackFragSubmessage,
    pub source_guid_prefix: GuidPrefix,
}
impl Mail for ProcessNackFragSubmessage {
    type Result = ();
}
impl MailHandler<ProcessNackFragSubmessage> for DataWriterActor {
    fn handle(
        &mut self,
        message: ProcessNackFragSubmessage,
    ) -> <ProcessNackFragSubmessage as Mail>::Result {
        self.on_nack_frag_submessage_received(
            &message.nackfrag_submessage,
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
impl MailHandler<SendMessage> for DataWriterActor {
    fn handle(&mut self, message: SendMessage) -> <SendMessage as Mail>::Result {
        self.send_message(message.message_sender_actor)
    }
}

pub struct SetListener {
    pub listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for SetListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetListener> for DataWriterActor {
    fn handle(&mut self, message: SetListener) -> <SetListener as Mail>::Result {
        if let Some(listener) = self.data_writer_listener_thread.take() {
            listener.join()?;
        }

        self.data_writer_listener_thread = message.listener.map(DataWriterListenerThread::new);
        self.status_kind = message.status_kind;
        Ok(())
    }
}

pub struct RemoveChange {
    pub seq_num: SequenceNumber,
}

impl Mail for RemoveChange {
    type Result = ();
}
impl MailHandler<RemoveChange> for DataWriterActor {
    fn handle(&mut self, message: RemoveChange) -> <RemoveChange as Mail>::Result {
        self.remove_change(message.seq_num)
    }
}

pub struct GetTopicAddress;
impl Mail for GetTopicAddress {
    type Result = ActorAddress<TopicActor>;
}
impl MailHandler<GetTopicAddress> for DataWriterActor {
    fn handle(&mut self, _: GetTopicAddress) -> <GetTopicAddress as Mail>::Result {
        self.topic_address.clone()
    }
}

fn get_discovered_reader_incompatible_qos_policy_list(
    writer_qos: &DataWriterQos,
    discovered_reader_data: &SubscriptionBuiltinTopicData,
    publisher_qos: &PublisherQos,
) -> Vec<QosPolicyId> {
    let mut incompatible_qos_policy_list = Vec::new();
    if &writer_qos.durability < discovered_reader_data.durability() {
        incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
    }
    if publisher_qos.presentation.access_scope < discovered_reader_data.presentation().access_scope
        || publisher_qos.presentation.coherent_access
            != discovered_reader_data.presentation().coherent_access
        || publisher_qos.presentation.ordered_access
            != discovered_reader_data.presentation().ordered_access
    {
        incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
    }
    if &writer_qos.deadline < discovered_reader_data.deadline() {
        incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
    }
    if &writer_qos.latency_budget < discovered_reader_data.latency_budget() {
        incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
    }
    if &writer_qos.liveliness < discovered_reader_data.liveliness() {
        incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
    }
    if writer_qos.reliability.kind < discovered_reader_data.reliability().kind {
        incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
    }
    if &writer_qos.destination_order < discovered_reader_data.destination_order() {
        incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
    }
    incompatible_qos_policy_list
}

fn send_message_to_reader_proxy_best_effort(
    reader_proxy: &mut RtpsReaderProxy,
    writer_id: EntityId,
    writer_cache: &WriterHistoryCache,
    data_max_size_serialized: usize,
    message_sender_actor: &ActorAddress<MessageSenderActor>,
) {
    // a_change_seq_num := the_reader_proxy.next_unsent_change();
    // if ( a_change_seq_num > the_reader_proxy.higuest_sent_seq_num +1 ) {
    //      GAP = new GAP(the_reader_locator.higuest_sent_seq_num + 1, a_change_seq_num -1);
    //      GAP.readerId := ENTITYID_UNKNOWN;
    //      GAP.filteredCount := 0;
    //      send GAP;
    // }
    // a_change := the_writer.writer_cache.get_change(a_change_seq_num );
    // if ( DDS_FILTER(the_reader_proxy, a_change) ) {
    //      DATA = new DATA(a_change);
    //      IF (the_reader_proxy.expectsInlineQos) {
    //          DATA.inlineQos := the_rtps_writer.related_dds_writer.qos;
    //          DATA.inlineQos += a_change.inlineQos;
    //      }
    //      DATA.readerId := ENTITYID_UNKNOWN;
    //      send DATA;
    // }
    // else {
    //      GAP = new GAP(a_change.sequenceNumber);
    //      GAP.readerId := ENTITYID_UNKNOWN;
    //      GAP.filteredCount := 1;
    //      send GAP;
    // }
    // the_reader_proxy.higuest_sent_seq_num := a_change_seq_num;
    while let Some(next_unsent_change_seq_num) = reader_proxy.next_unsent_change(writer_cache) {
        if next_unsent_change_seq_num > reader_proxy.highest_sent_seq_num() + 1 {
            let gap_start_sequence_number = reader_proxy.highest_sent_seq_num() + 1;
            let gap_end_sequence_number = next_unsent_change_seq_num - 1;
            let gap_submessage = Box::new(GapSubmessage::new(
                reader_proxy.remote_reader_guid().entity_id(),
                writer_id,
                gap_start_sequence_number,
                SequenceNumberSet::new(gap_end_sequence_number + 1, []),
            ));

            message_sender_actor
                .send_actor_mail(message_sender_actor::WriteMessage {
                    submessages: vec![gap_submessage],
                    destination_locator_list: reader_proxy.unicast_locator_list().to_vec(),
                })
                .ok();

            reader_proxy.set_highest_sent_seq_num(next_unsent_change_seq_num);
        } else if let Some(cache_change) = writer_cache
            .change_list()
            .find(|cc| cc.sequence_number() == next_unsent_change_seq_num)
        {
            let number_of_fragments = cache_change
                .data_value()
                .len()
                .div_ceil(data_max_size_serialized);

            // Either send a DATAFRAG submessages or send a single DATA submessage
            if number_of_fragments > 1 {
                for frag_index in 0..number_of_fragments {
                    let info_dst = Box::new(InfoDestinationSubmessage::new(
                        reader_proxy.remote_reader_guid().prefix(),
                    ));

                    let info_timestamp = Box::new(InfoTimestampSubmessage::new(
                        false,
                        cache_change.timestamp(),
                    ));

                    let inline_qos_flag = true;
                    let key_flag = match cache_change.kind() {
                        ChangeKind::Alive => false,
                        ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => true,
                        _ => todo!(),
                    };
                    let non_standard_payload_flag = false;
                    let reader_id = reader_proxy.remote_reader_guid().entity_id();
                    let writer_id = cache_change.writer_guid().entity_id();
                    let writer_sn = cache_change.sequence_number();
                    let fragment_starting_num = (frag_index + 1) as u32;
                    let fragments_in_submessage = 1;
                    let fragment_size = data_max_size_serialized as u16;
                    let data_size = cache_change.data_value().len() as u32;
                    let inline_qos = cache_change.inline_qos().clone();

                    let start = frag_index * data_max_size_serialized;
                    let end = std::cmp::min(
                        (frag_index + 1) * data_max_size_serialized,
                        cache_change.data_value().len(),
                    );

                    let serialized_payload =
                        SerializedDataFragment::new(cache_change.data_value().clone(), start..end);

                    let data_frag = Box::new(DataFragSubmessage::new(
                        inline_qos_flag,
                        non_standard_payload_flag,
                        key_flag,
                        reader_id,
                        writer_id,
                        writer_sn,
                        fragment_starting_num,
                        fragments_in_submessage,
                        fragment_size,
                        data_size,
                        inline_qos,
                        serialized_payload,
                    ));

                    message_sender_actor
                        .send_actor_mail(message_sender_actor::WriteMessage {
                            submessages: vec![info_dst, info_timestamp, data_frag],
                            destination_locator_list: reader_proxy.unicast_locator_list().to_vec(),
                        })
                        .ok();
                }
            } else {
                let info_dst = Box::new(InfoDestinationSubmessage::new(
                    reader_proxy.remote_reader_guid().prefix(),
                ));

                let info_timestamp = Box::new(InfoTimestampSubmessage::new(
                    false,
                    cache_change.timestamp(),
                ));

                let data_submessage = Box::new(
                    cache_change.as_data_submessage(reader_proxy.remote_reader_guid().entity_id()),
                );

                message_sender_actor
                    .send_actor_mail(message_sender_actor::WriteMessage {
                        submessages: vec![info_dst, info_timestamp, data_submessage],
                        destination_locator_list: reader_proxy.unicast_locator_list().to_vec(),
                    })
                    .ok();
            }
        } else {
            message_sender_actor
                .send_actor_mail(message_sender_actor::WriteMessage {
                    submessages: vec![Box::new(GapSubmessage::new(
                        ENTITYID_UNKNOWN,
                        writer_id,
                        next_unsent_change_seq_num,
                        SequenceNumberSet::new(next_unsent_change_seq_num + 1, []),
                    ))],
                    destination_locator_list: reader_proxy.unicast_locator_list().to_vec(),
                })
                .ok();
        }

        reader_proxy.set_highest_sent_seq_num(next_unsent_change_seq_num);
    }
}

fn send_message_to_reader_proxy_reliable(
    reader_proxy: &mut RtpsReaderProxy,
    writer_id: EntityId,
    writer_cache: &WriterHistoryCache,
    data_max_size_serialized: usize,
    heartbeat_period: Duration,
    message_sender_actor: &ActorAddress<MessageSenderActor>,
) {
    // Top part of the state machine - Figure 8.19 RTPS standard
    if reader_proxy.unsent_changes(writer_cache) {
        while let Some(next_unsent_change_seq_num) = reader_proxy.next_unsent_change(writer_cache) {
            if next_unsent_change_seq_num > reader_proxy.highest_sent_seq_num() + 1 {
                let gap_start_sequence_number = reader_proxy.highest_sent_seq_num() + 1;
                let gap_end_sequence_number = next_unsent_change_seq_num - 1;
                let gap_submessage = Box::new(GapSubmessage::new(
                    reader_proxy.remote_reader_guid().entity_id(),
                    writer_id,
                    gap_start_sequence_number,
                    SequenceNumberSet::new(gap_end_sequence_number + 1, []),
                ));
                let first_sn = writer_cache.get_seq_num_min().unwrap_or(1);
                let last_sn = writer_cache.get_seq_num_max().unwrap_or(0);
                let heartbeat_submessage = Box::new(
                    reader_proxy
                        .heartbeat_machine()
                        .generate_new_heartbeat(writer_id, first_sn, last_sn),
                );
                message_sender_actor
                    .send_actor_mail(message_sender_actor::WriteMessage {
                        submessages: vec![gap_submessage, heartbeat_submessage],
                        destination_locator_list: reader_proxy.unicast_locator_list().to_vec(),
                    })
                    .ok();
            } else {
                send_change_message_reader_proxy_reliable(
                    reader_proxy,
                    writer_id,
                    writer_cache,
                    data_max_size_serialized,
                    next_unsent_change_seq_num,
                    message_sender_actor,
                );
            }
            reader_proxy.set_highest_sent_seq_num(next_unsent_change_seq_num);
        }
    } else if !reader_proxy.unacked_changes(writer_cache) {
        // Idle
    } else if reader_proxy
        .heartbeat_machine()
        .is_time_for_heartbeat(heartbeat_period.into())
    {
        let first_sn = writer_cache.get_seq_num_min().unwrap_or(1);
        let last_sn = writer_cache.get_seq_num_max().unwrap_or(0);
        let heartbeat_submessage = Box::new(
            reader_proxy
                .heartbeat_machine()
                .generate_new_heartbeat(writer_id, first_sn, last_sn),
        );

        message_sender_actor
            .send_actor_mail(message_sender_actor::WriteMessage {
                submessages: vec![heartbeat_submessage],
                destination_locator_list: reader_proxy.unicast_locator_list().to_vec(),
            })
            .ok();
    }

    // Middle-part of the state-machine - Figure 8.19 RTPS standard
    if !reader_proxy.requested_changes().is_empty() {
        while let Some(next_requested_change_seq_num) = reader_proxy.next_requested_change() {
            // "a_change.status := UNDERWAY;" should be done by next_requested_change() as
            // it's not done here to avoid the change being a mutable reference
            // Also the post-condition:
            // a_change BELONGS-TO the_reader_proxy.requested_changes() ) == FALSE
            // should be full-filled by next_requested_change()
            send_change_message_reader_proxy_reliable(
                reader_proxy,
                writer_id,
                writer_cache,
                data_max_size_serialized,
                next_requested_change_seq_num,
                message_sender_actor,
            );
        }
    }
}

fn send_change_message_reader_proxy_reliable(
    reader_proxy: &mut RtpsReaderProxy,
    writer_id: EntityId,
    writer_cache: &WriterHistoryCache,
    data_max_size_serialized: usize,
    change_seq_num: SequenceNumber,
    message_sender_actor: &ActorAddress<MessageSenderActor>,
) {
    match writer_cache
        .change_list()
        .find(|cc| cc.sequence_number() == change_seq_num)
    {
        Some(cache_change) if change_seq_num > reader_proxy.first_relevant_sample_seq_num() => {
            let number_of_fragments = cache_change
                .data_value()
                .len()
                .div_ceil(data_max_size_serialized);

            // Either send a DATAFRAG submessages or send a single DATA submessage
            if number_of_fragments > 1 {
                for frag_index in 0..number_of_fragments {
                    let info_dst = Box::new(InfoDestinationSubmessage::new(
                        reader_proxy.remote_reader_guid().prefix(),
                    ));

                    let info_timestamp = Box::new(InfoTimestampSubmessage::new(
                        false,
                        cache_change.timestamp(),
                    ));

                    let inline_qos_flag = true;
                    let key_flag = match cache_change.kind() {
                        ChangeKind::Alive => false,
                        ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => true,
                        _ => todo!(),
                    };
                    let non_standard_payload_flag = false;
                    let reader_id = reader_proxy.remote_reader_guid().entity_id();
                    let writer_id = cache_change.writer_guid().entity_id();
                    let writer_sn = cache_change.sequence_number();
                    let fragment_starting_num = (frag_index + 1) as u32;
                    let fragments_in_submessage = 1;
                    let fragment_size = data_max_size_serialized as u16;
                    let data_size = cache_change.data_value().len() as u32;
                    let inline_qos = cache_change.inline_qos().clone();

                    let start = frag_index * data_max_size_serialized;
                    let end = std::cmp::min(
                        (frag_index + 1) * data_max_size_serialized,
                        cache_change.data_value().len(),
                    );

                    let serialized_payload =
                        SerializedDataFragment::new(cache_change.data_value().clone(), start..end);

                    let data_frag = Box::new(DataFragSubmessage::new(
                        inline_qos_flag,
                        non_standard_payload_flag,
                        key_flag,
                        reader_id,
                        writer_id,
                        writer_sn,
                        fragment_starting_num,
                        fragments_in_submessage,
                        fragment_size,
                        data_size,
                        inline_qos,
                        serialized_payload,
                    ));

                    message_sender_actor
                        .send_actor_mail(message_sender_actor::WriteMessage {
                            submessages: vec![info_dst, info_timestamp, data_frag],
                            destination_locator_list: reader_proxy.unicast_locator_list().to_vec(),
                        })
                        .ok();
                }
            } else {
                let info_dst = Box::new(InfoDestinationSubmessage::new(
                    reader_proxy.remote_reader_guid().prefix(),
                ));

                let info_timestamp = Box::new(InfoTimestampSubmessage::new(
                    false,
                    cache_change.timestamp(),
                ));

                let data_submessage = Box::new(
                    cache_change.as_data_submessage(reader_proxy.remote_reader_guid().entity_id()),
                );

                let first_sn = writer_cache.get_seq_num_min().unwrap_or(1);
                let last_sn = writer_cache.get_seq_num_max().unwrap_or(0);
                let heartbeat = Box::new(
                    reader_proxy
                        .heartbeat_machine()
                        .generate_new_heartbeat(writer_id, first_sn, last_sn),
                );

                message_sender_actor
                    .send_actor_mail(message_sender_actor::WriteMessage {
                        submessages: vec![info_dst, info_timestamp, data_submessage, heartbeat],
                        destination_locator_list: reader_proxy.unicast_locator_list().to_vec(),
                    })
                    .ok();
            }
        }
        _ => {
            let info_dst = Box::new(InfoDestinationSubmessage::new(
                reader_proxy.remote_reader_guid().prefix(),
            ));

            let gap_submessage = Box::new(GapSubmessage::new(
                ENTITYID_UNKNOWN,
                writer_id,
                change_seq_num,
                SequenceNumberSet::new(change_seq_num + 1, []),
            ));

            message_sender_actor
                .send_actor_mail(message_sender_actor::WriteMessage {
                    submessages: vec![info_dst, gap_submessage],
                    destination_locator_list: reader_proxy.unicast_locator_list().to_vec(),
                })
                .ok();
        }
    }
}
