use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    data_representation_builtin_endpoints::{
        discovered_reader_data::DiscoveredReaderData,
        discovered_writer_data::{DiscoveredWriterData, WriterProxy},
    },
    dds_async::{publisher::PublisherAsync, topic::TopicAsync},
    implementation::{
        actor::{Actor, ActorAddress, DEFAULT_ACTOR_BUFFER_SIZE},
        data_representation_inline_qos::{
            parameter_id_values::PID_STATUS_INFO,
            types::{
                STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED, STATUS_INFO_UNREGISTERED,
            },
        },
        payload_serializer_deserializer::{
            cdr_serializer::ClassicCdrSerializer, endianness::CdrEndianness,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::{InstanceHandle, HANDLE_NIL},
        qos::{DataWriterQos, PublisherQos, TopicQos},
        qos_policy::{
            DurabilityQosPolicyKind, HistoryQosPolicyKind, QosPolicyId, ReliabilityQosPolicyKind,
            DEADLINE_QOS_POLICY_ID, DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID,
            INVALID_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID,
            PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID,
        },
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, QosPolicyCount, StatusKind,
        },
        time::{Duration, DurationKind, Time},
    },
    rtps::{
        message_receiver::MessageReceiver,
        messages::{
            overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
            submessage_elements::{ArcSlice, Parameter, ParameterList, SequenceNumberSet},
            submessages::{
                ack_nack::AckNackSubmessage, gap::GapSubmessage,
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
        writer_history_cache::{DataFragSubmessages, RtpsWriterCacheChange, WriterHistoryCache},
    },
    serialized_payload::cdr::serialize::CdrSerialize,
    topic_definition::type_support::DdsKey,
};
use dust_dds_derive::actor_interface;
use std::collections::{HashMap, HashSet};

use super::{
    any_data_writer_listener::AnyDataWriterListener,
    data_writer_listener_actor::DataWriterListenerActor,
    domain_participant_listener_actor::DomainParticipantListenerActor,
    message_sender_actor::MessageSenderActor, publisher_listener_actor::PublisherListenerActor,
    status_condition_actor::StatusConditionActor, topic_actor::TopicActor,
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
}

pub struct DataWriterActor {
    rtps_writer: RtpsWriter,
    reader_locators: Vec<RtpsReaderLocator>,
    matched_readers: Vec<RtpsReaderProxy>,
    topic: Actor<TopicActor>,
    matched_subscriptions: MatchedSubscriptions,
    incompatible_subscriptions: IncompatibleSubscriptions,
    enabled: bool,
    status_condition: Actor<StatusConditionActor>,
    listener: Actor<DataWriterListenerActor>,
    status_kind: Vec<StatusKind>,
    writer_cache: WriterHistoryCache,
    qos: DataWriterQos,
    registered_instance_list: HashSet<InstanceHandle>,
}

impl DataWriterActor {
    pub fn new(
        rtps_writer: RtpsWriter,
        topic: Actor<TopicActor>,
        listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        status_kind: Vec<StatusKind>,
        qos: DataWriterQos,
        handle: &tokio::runtime::Handle,
    ) -> Self {
        let status_condition = Actor::spawn(
            StatusConditionActor::default(),
            handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );
        let listener = Actor::spawn(
            DataWriterListenerActor::new(listener),
            handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );
        let max_changes = match qos.history.kind {
            HistoryQosPolicyKind::KeepLast(keep_last) => Some(keep_last),
            HistoryQosPolicyKind::KeepAll => None,
        };
        DataWriterActor {
            rtps_writer,
            reader_locators: Vec::new(),
            matched_readers: Vec::new(),
            topic,
            matched_subscriptions: MatchedSubscriptions::new(),
            incompatible_subscriptions: IncompatibleSubscriptions::new(),
            enabled: false,
            status_condition,
            listener,
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

    async fn add_change(
        &mut self,
        change: RtpsWriterCacheChange,
        message_sender_actor: Actor<MessageSenderActor>,
        now: Time,
        writer_address: Actor<DataWriterActor>,
    ) {
        let seq_num = change.sequence_number();

        if let DurationKind::Finite(lifespan) = self.qos.lifespan.duration {
            let change_lifespan =
                (crate::infrastructure::time::Time::from(change.timestamp()) - now) + lifespan;
            if change_lifespan > Duration::new(0, 0) {
                self.writer_cache.add_change(change);

                tokio::spawn(async move {
                    tokio::time::sleep(change_lifespan.into()).await;
                    writer_address.remove_change(seq_num).await;
                });
            }
        } else {
            self.writer_cache.add_change(change);
        }

        self.send_message(message_sender_actor).await;
    }
}

#[actor_interface]
impl DataWriterActor {
    pub fn get_instance_handle(&self) -> InstanceHandle {
        InstanceHandle::new(self.rtps_writer.guid().into())
    }

    #[allow(clippy::unused_unit)]
    fn add_matched_publication(
        &mut self,
        handle: InstanceHandle,
        subscription_data: SubscriptionBuiltinTopicData,
    ) -> () {
        self.matched_subscriptions
            .add_matched_subscription(handle, subscription_data)
    }

    #[allow(clippy::unused_unit)]
    fn remove_matched_subscription(&mut self, handle: InstanceHandle) -> () {
        self.matched_subscriptions
            .remove_matched_subscription(handle)
    }

    fn get_matched_subscriptions(&self) -> Vec<InstanceHandle> {
        self.matched_subscriptions.get_matched_subscriptions()
    }

    fn get_matched_subscription_data(
        &self,
        handle: InstanceHandle,
    ) -> Option<SubscriptionBuiltinTopicData> {
        self.matched_subscriptions
            .get_matched_subscription_data(handle)
            .cloned()
    }

    #[allow(clippy::unused_unit)]
    fn add_offered_incompatible_qos(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) -> () {
        self.incompatible_subscriptions
            .add_offered_incompatible_qos(handle, incompatible_qos_policy_list)
    }

    fn get_offered_incompatible_qos_status(&mut self) -> OfferedIncompatibleQosStatus {
        self.incompatible_subscriptions
            .get_offered_incompatible_qos_status()
    }

    fn get_offered_deadline_missed_status(&self) -> OfferedDeadlineMissedStatus {
        todo!()
    }

    fn get_liveliness_lost_status(&self) -> LivelinessLostStatus {
        todo!()
    }

    fn get_incompatible_subscriptions(&self) -> Vec<InstanceHandle> {
        self.incompatible_subscriptions
            .get_incompatible_subscriptions()
    }

    #[allow(clippy::unused_unit)]
    fn enable(&mut self) -> () {
        self.enabled = true;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_statuscondition(&self) -> ActorAddress<StatusConditionActor> {
        self.status_condition.address()
    }

    fn guid(&self) -> Guid {
        self.rtps_writer.guid()
    }

    fn heartbeat_period(&self) -> Duration {
        self.rtps_writer.heartbeat_period().into()
    }

    #[allow(clippy::unused_unit)]
    fn matched_reader_remove(&mut self, a_reader_guid: Guid) -> () {
        self.matched_readers
            .retain(|x| x.remote_reader_guid() != a_reader_guid)
    }

    fn get_qos(&self) -> DataWriterQos {
        self.qos.clone()
    }

    #[allow(clippy::unused_unit)]
    fn set_qos(&mut self, qos: DataWriterQos) -> () {
        self.qos = qos;
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

    async fn unregister_instance_w_timestamp(
        &mut self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
        message_sender_actor: Actor<MessageSenderActor>,
        now: Time,
        data_writer_address: Actor<DataWriterActor>,
    ) -> DdsResult<()> {
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

        let inline_qos = ParameterList::new(vec![Parameter::new(
            PID_STATUS_INFO,
            ArcSlice::from(serialized_status_info),
        )]);

        let change: RtpsWriterCacheChange = self.rtps_writer.new_change(
            ChangeKind::NotAliveUnregistered,
            instance_serialized_key,
            inline_qos,
            handle.into(),
            timestamp.into(),
        );

        self.add_change(change, message_sender_actor, now, data_writer_address)
            .await;
        Ok(())
    }

    fn lookup_instance(
        &self,
        instance_handle: InstanceHandle,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        Ok(
            if self.registered_instance_list.contains(&instance_handle) {
                Some(instance_handle)
            } else {
                None
            },
        )
    }

    async fn dispose_w_timestamp(
        &mut self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
        message_sender_actor: Actor<MessageSenderActor>,
        now: Time,
        data_writer_address: Actor<DataWriterActor>,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let mut serialized_status_info = Vec::new();
        let mut serializer =
            ClassicCdrSerializer::new(&mut serialized_status_info, CdrEndianness::LittleEndian);
        STATUS_INFO_DISPOSED.serialize(&mut serializer).unwrap();

        let inline_qos = ParameterList::new(vec![Parameter::new(
            PID_STATUS_INFO,
            ArcSlice::from(serialized_status_info),
        )]);

        let change: RtpsWriterCacheChange = self.rtps_writer.new_change(
            ChangeKind::NotAliveDisposed,
            instance_serialized_key,
            inline_qos,
            handle.into(),
            timestamp.into(),
        );

        self.add_change(change, message_sender_actor, now, data_writer_address)
            .await;

        Ok(())
    }

    fn are_all_changes_acknowledge(&mut self) -> bool {
        !self
            .matched_readers
            .iter()
            .any(|rp| rp.unacked_changes(&self.writer_cache))
    }

    async fn as_discovered_writer_data(
        &self,
        topic_qos: TopicQos,
        publisher_qos: PublisherQos,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        xml_type: String,
    ) -> DiscoveredWriterData {
        let type_name = self.topic.get_type_name().await;
        let topic_name = self.topic.get_name().await;
        let writer_qos = &self.qos;
        let unicast_locator_list = if self.rtps_writer.unicast_locator_list().is_empty() {
            default_unicast_locator_list
        } else {
            self.rtps_writer.unicast_locator_list().to_vec()
        };

        let multicast_locator_list = if self.rtps_writer.unicast_locator_list().is_empty() {
            default_multicast_locator_list
        } else {
            self.rtps_writer.multicast_locator_list().to_vec()
        };

        DiscoveredWriterData::new(
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
                publisher_qos.clone(),
                topic_qos.topic_data,
                xml_type,
            ),
            WriterProxy::new(
                self.rtps_writer.guid(),
                EntityId::new([0; 3], USER_DEFINED_UNKNOWN),
                unicast_locator_list,
                multicast_locator_list,
                None,
            ),
        )
    }

    async fn get_publication_matched_status(&mut self) -> PublicationMatchedStatus {
        self.status_condition
            .remove_communication_state(StatusKind::PublicationMatched)
            .await;
        self.matched_subscriptions.get_publication_matched_status()
    }

    async fn get_topic_name(&self) -> String {
        self.topic.get_name().await
    }

    #[allow(clippy::too_many_arguments)]
    async fn write_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        instance_handle: InstanceHandle,
        _handle: Option<InstanceHandle>,
        timestamp: Time,
        message_sender_actor: Actor<MessageSenderActor>,
        now: Time,
        data_writer_address: Actor<DataWriterActor>,
    ) -> DdsResult<()> {
        let handle = self
            .register_instance_w_timestamp(instance_handle, timestamp)?
            .unwrap_or(HANDLE_NIL);
        let change = self.rtps_writer.new_change(
            ChangeKind::Alive,
            serialized_data,
            ParameterList::empty(),
            handle.into(),
            timestamp.into(),
        );

        self.add_change(change, message_sender_actor, now, data_writer_address)
            .await;

        Ok(())
    }

    async fn get_type_name(&self) -> String {
        self.topic.get_name().await
    }

    #[allow(clippy::too_many_arguments, clippy::unused_unit)]
    async fn add_matched_reader(
        &mut self,
        discovered_reader_data: DiscoveredReaderData,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        data_writer_address: ActorAddress<DataWriterActor>,
        publisher: PublisherAsync,
        publisher_qos: PublisherQos,
        publisher_mask_listener: (ActorAddress<PublisherListenerActor>, Vec<StatusKind>),
        participant_mask_listener: (
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
    ) -> () {
        let type_name = self.topic.get_type_name().await;
        let topic_name = self.topic.get_name().await;
        let is_matched_topic_name = discovered_reader_data
            .subscription_builtin_topic_data()
            .topic_name()
            == topic_name;
        let is_matched_type_name = discovered_reader_data
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
                discovered_reader_data.subscription_builtin_topic_data(),
                &publisher_qos,
            );
            let instance_handle =
                InstanceHandle::try_from_key(&discovered_reader_data.get_key().unwrap()).unwrap();

            if incompatible_qos_policy_list.is_empty() {
                let unicast_locator_list = if discovered_reader_data
                    .reader_proxy()
                    .unicast_locator_list()
                    .is_empty()
                {
                    default_unicast_locator_list
                } else {
                    discovered_reader_data
                        .reader_proxy()
                        .unicast_locator_list()
                        .to_vec()
                };

                let multicast_locator_list = if discovered_reader_data
                    .reader_proxy()
                    .multicast_locator_list()
                    .is_empty()
                {
                    default_multicast_locator_list
                } else {
                    discovered_reader_data
                        .reader_proxy()
                        .multicast_locator_list()
                        .to_vec()
                };

                let proxy_reliability = match discovered_reader_data
                    .subscription_builtin_topic_data()
                    .reliability()
                    .kind
                {
                    ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
                    ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
                };

                let first_relevant_sample_seq_num = match discovered_reader_data
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
                    discovered_reader_data.reader_proxy().remote_reader_guid(),
                    discovered_reader_data
                        .reader_proxy()
                        .remote_group_entity_id(),
                    &unicast_locator_list,
                    &multicast_locator_list,
                    discovered_reader_data.reader_proxy().expects_inline_qos(),
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

                if !self.get_matched_subscriptions().contains(&instance_handle)
                    || self.get_matched_subscription_data(instance_handle).as_ref()
                        != Some(discovered_reader_data.subscription_builtin_topic_data())
                {
                    self.matched_subscriptions.add_matched_subscription(
                        instance_handle,
                        discovered_reader_data
                            .subscription_builtin_topic_data()
                            .clone(),
                    );
                    self.on_publication_matched(
                        data_writer_address,
                        publisher,
                        publisher_mask_listener,
                        participant_mask_listener,
                    )
                    .await;
                }
            } else {
                self.incompatible_subscriptions
                    .add_offered_incompatible_qos(instance_handle, incompatible_qos_policy_list);
                self.on_offered_incompatible_qos(
                    data_writer_address,
                    publisher,
                    publisher_mask_listener,
                    participant_mask_listener,
                )
                .await;
            }
        }
    }

    #[allow(clippy::unused_unit)]
    async fn remove_matched_reader(
        &mut self,
        discovered_reader_handle: InstanceHandle,
        data_writer_address: ActorAddress<DataWriterActor>,
        publisher: PublisherAsync,
        publisher_mask_listener: (ActorAddress<PublisherListenerActor>, Vec<StatusKind>),
        participant_mask_listener: (
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
    ) -> () {
        if let Some(r) = self.get_matched_subscription_data(discovered_reader_handle) {
            let handle = r.key().value.into();
            self.matched_reader_remove(handle);
            self.remove_matched_subscription(InstanceHandle::new(handle.into()));

            self.on_publication_matched(
                data_writer_address,
                publisher,
                publisher_mask_listener,
                participant_mask_listener,
            )
            .await;
        }
    }

    fn process_rtps_message(&mut self, message: RtpsMessageRead) {
        let mut message_receiver = MessageReceiver::new(&message);
        while let Some(submessage) = message_receiver.next() {
            match &submessage {
                RtpsSubmessageReadKind::AckNack(acknack_submessage) => self
                    .on_acknack_submessage_received(
                        acknack_submessage,
                        message_receiver.source_guid_prefix(),
                    ),
                RtpsSubmessageReadKind::NackFrag(nackfrag_submessage) => self
                    .on_nack_frag_submessage_received(
                        nackfrag_submessage,
                        message_receiver.source_guid_prefix(),
                    ),
                _ => (),
            }
        }
    }

    async fn send_message(&mut self, message_sender_actor: Actor<MessageSenderActor>) {
        self.send_message_to_reader_locators(&message_sender_actor)
            .await;
        self.send_message_to_reader_proxies(&message_sender_actor)
            .await;
    }

    #[allow(clippy::unused_unit)]
    fn set_listener(
        &mut self,
        listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        status_kind: Vec<StatusKind>,
        runtime_handle: tokio::runtime::Handle,
    ) -> () {
        self.listener = Actor::spawn(
            DataWriterListenerActor::new(listener),
            &runtime_handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );
        self.status_kind = status_kind;
    }

    fn remove_change(&mut self, seq_num: SequenceNumber) {
        self.writer_cache
            .remove_change(|cc| cc.sequence_number() == seq_num)
    }
}

impl DataWriterActor {
    fn on_acknack_submessage_received(
        &mut self,
        acknack_submessage: &AckNackSubmessage,
        source_guid_prefix: GuidPrefix,
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
                        }
                    }
                }
            }
        }
    }

    async fn send_message_to_reader_locators(
        &mut self,
        message_sender_actor: &Actor<MessageSenderActor>,
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
                                .write(
                                    vec![info_ts_submessage, data_submessage],
                                    vec![reader_locator.locator()],
                                )
                                .await;
                        } else {
                            let gap_submessage = Box::new(GapSubmessage::new(
                                ENTITYID_UNKNOWN,
                                self.rtps_writer.guid().entity_id(),
                                unsent_change_seq_num,
                                SequenceNumberSet::new(unsent_change_seq_num + 1, []),
                            ));
                            message_sender_actor
                                .write(vec![gap_submessage], vec![reader_locator.locator()])
                                .await;
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

    async fn send_message_to_reader_proxies(
        &mut self,
        message_sender_actor: &Actor<MessageSenderActor>,
    ) {
        for reader_proxy in &mut self.matched_readers {
            match (&self.qos.reliability.kind, reader_proxy.reliability()) {
                (ReliabilityQosPolicyKind::BestEffort, ReliabilityKind::BestEffort)
                | (ReliabilityQosPolicyKind::Reliable, ReliabilityKind::BestEffort) => {
                    send_message_to_reader_proxy_best_effort(
                        reader_proxy,
                        self.rtps_writer.guid().entity_id(),
                        &self.writer_cache,
                        message_sender_actor,
                    )
                    .await
                }
                (ReliabilityQosPolicyKind::Reliable, ReliabilityKind::Reliable) => {
                    send_message_to_reader_proxy_reliable(
                        reader_proxy,
                        self.rtps_writer.guid().entity_id(),
                        &self.writer_cache,
                        self.rtps_writer.heartbeat_period().into(),
                        message_sender_actor,
                    )
                    .await
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

    async fn on_publication_matched(
        &mut self,
        data_writer_address: ActorAddress<DataWriterActor>,
        publisher: PublisherAsync,
        (publisher_listener, publisher_listener_mask): (
            ActorAddress<PublisherListenerActor>,
            Vec<StatusKind>,
        ),
        (participant_listener, participant_listener_mask): (
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
    ) {
        self.status_condition
            .add_communication_state(StatusKind::PublicationMatched)
            .await;
        if self.status_kind.contains(&StatusKind::PublicationMatched) {
            let type_name = self.topic.get_type_name().await;
            let topic_name = self.topic.get_name().await;
            let status = self.get_publication_matched_status().await;
            let participant = publisher.get_participant();
            let topic_address = self.topic.address();
            let topic_status_condition_address = self.topic.get_statuscondition().await;
            self.listener
                .trigger_on_publication_matched(
                    data_writer_address,
                    self.status_condition.address(),
                    publisher,
                    TopicAsync::new(
                        topic_address,
                        topic_status_condition_address,
                        type_name,
                        topic_name,
                        participant,
                    ),
                    status,
                )
                .await;
        } else if publisher_listener_mask.contains(&StatusKind::PublicationMatched) {
            let status = self.get_publication_matched_status().await;
            publisher_listener
                .upgrade()
                .expect("Listener should exist")
                .trigger_on_publication_matched(status)
                .await;
        } else if participant_listener_mask.contains(&StatusKind::PublicationMatched) {
            let status = self.get_publication_matched_status().await;
            participant_listener
                .upgrade()
                .expect("Listener should exist")
                .trigger_on_publication_matched(status)
                .await;
        }
    }

    async fn on_offered_incompatible_qos(
        &mut self,
        data_writer_address: ActorAddress<DataWriterActor>,
        publisher: PublisherAsync,
        (publisher_listener, publisher_listener_mask): (
            ActorAddress<PublisherListenerActor>,
            Vec<StatusKind>,
        ),
        (participant_listener, participant_listener_mask): (
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
    ) {
        self.status_condition
            .add_communication_state(StatusKind::OfferedIncompatibleQos)
            .await;
        if self
            .status_kind
            .contains(&StatusKind::OfferedIncompatibleQos)
        {
            let type_name = self.topic.get_type_name().await;
            let topic_name = self.topic.get_name().await;
            let status = self.get_offered_incompatible_qos_status();
            let participant = publisher.get_participant();
            let topic_address = self.topic.address();
            let topic_status_condition_address = self.topic.get_statuscondition().await;
            self.listener
                .trigger_on_offered_incompatible_qos(
                    data_writer_address,
                    self.status_condition.address(),
                    publisher,
                    TopicAsync::new(
                        topic_address,
                        topic_status_condition_address,
                        type_name,
                        topic_name,
                        participant,
                    ),
                    status,
                )
                .await;
        } else if publisher_listener_mask.contains(&StatusKind::OfferedIncompatibleQos) {
            let status = self.get_offered_incompatible_qos_status();
            publisher_listener
                .upgrade()
                .expect("Listener should exist")
                .trigger_on_offered_incompatible_qos(status)
                .await;
        } else if participant_listener_mask.contains(&StatusKind::OfferedIncompatibleQos) {
            let status = self.get_offered_incompatible_qos_status();
            participant_listener
                .upgrade()
                .expect("Listener should exist")
                .trigger_on_offered_incompatible_qos(status)
                .await;
        }
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

async fn send_message_to_reader_proxy_best_effort(
    reader_proxy: &mut RtpsReaderProxy,
    writer_id: EntityId,
    writer_cache: &WriterHistoryCache,
    message_sender_actor: &Actor<MessageSenderActor>,
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
                .write(
                    vec![gap_submessage],
                    reader_proxy.unicast_locator_list().to_vec(),
                )
                .await;
            reader_proxy.set_highest_sent_seq_num(next_unsent_change_seq_num);
        } else if let Some(cache_change) = writer_cache
            .change_list()
            .find(|cc| cc.sequence_number() == next_unsent_change_seq_num)
        {
            // Either send a DATAFRAG submessages or send a single DATA submessage
            if cache_change.data_value().len() > 1 {
                let cache_change_frag = DataFragSubmessages::new(
                    cache_change,
                    reader_proxy.remote_reader_guid().entity_id(),
                );
                for data_frag_submessage in cache_change_frag.into_iter() {
                    let info_dst = Box::new(InfoDestinationSubmessage::new(
                        reader_proxy.remote_reader_guid().prefix(),
                    ));

                    let info_timestamp = Box::new(InfoTimestampSubmessage::new(
                        false,
                        cache_change.timestamp(),
                    ));

                    let data_frag = Box::new(data_frag_submessage);

                    message_sender_actor
                        .write(
                            vec![info_dst, info_timestamp, data_frag],
                            reader_proxy.unicast_locator_list().to_vec(),
                        )
                        .await;
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
                    .write(
                        vec![info_dst, info_timestamp, data_submessage],
                        reader_proxy.unicast_locator_list().to_vec(),
                    )
                    .await;
            }
        } else {
            message_sender_actor
                .write(
                    vec![Box::new(GapSubmessage::new(
                        ENTITYID_UNKNOWN,
                        writer_id,
                        next_unsent_change_seq_num,
                        SequenceNumberSet::new(next_unsent_change_seq_num + 1, []),
                    ))],
                    reader_proxy.unicast_locator_list().to_vec(),
                )
                .await;
        }

        reader_proxy.set_highest_sent_seq_num(next_unsent_change_seq_num);
    }
}

async fn send_message_to_reader_proxy_reliable(
    reader_proxy: &mut RtpsReaderProxy,
    writer_id: EntityId,
    writer_cache: &WriterHistoryCache,
    heartbeat_period: Duration,
    message_sender_actor: &Actor<MessageSenderActor>,
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
                        .submessage(writer_id, first_sn, last_sn),
                );
                message_sender_actor
                    .write(
                        vec![gap_submessage, heartbeat_submessage],
                        reader_proxy.unicast_locator_list().to_vec(),
                    )
                    .await;
            } else {
                send_change_message_reader_proxy_reliable(
                    reader_proxy,
                    writer_id,
                    writer_cache,
                    next_unsent_change_seq_num,
                    message_sender_actor,
                )
                .await;
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
                .submessage(writer_id, first_sn, last_sn),
        );
        message_sender_actor
            .write(
                vec![heartbeat_submessage],
                reader_proxy.unicast_locator_list().to_vec(),
            )
            .await;
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
                next_requested_change_seq_num,
                message_sender_actor,
            )
            .await;
        }
    }
}

async fn send_change_message_reader_proxy_reliable(
    reader_proxy: &mut RtpsReaderProxy,
    writer_id: EntityId,
    writer_cache: &WriterHistoryCache,
    change_seq_num: SequenceNumber,
    message_sender_actor: &Actor<MessageSenderActor>,
) {
    match writer_cache
        .change_list()
        .find(|cc| cc.sequence_number() == change_seq_num)
    {
        Some(cache_change) if change_seq_num > reader_proxy.first_relevant_sample_seq_num() => {
            // Either send a DATAFRAG submessages or send a single DATA submessage
            if cache_change.data_value().len() > 1 {
                let cache_change_frag = DataFragSubmessages::new(
                    cache_change,
                    reader_proxy.remote_reader_guid().entity_id(),
                );
                for data_frag_submessage in cache_change_frag.into_iter() {
                    let info_dst = Box::new(InfoDestinationSubmessage::new(
                        reader_proxy.remote_reader_guid().prefix(),
                    ));

                    let info_timestamp = Box::new(InfoTimestampSubmessage::new(
                        false,
                        cache_change.timestamp(),
                    ));

                    let data_frag = Box::new(data_frag_submessage);

                    message_sender_actor
                        .write(
                            vec![info_dst, info_timestamp, data_frag],
                            reader_proxy.unicast_locator_list().to_vec(),
                        )
                        .await;
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
                        .submessage(writer_id, first_sn, last_sn),
                );

                message_sender_actor
                    .write(
                        vec![info_dst, info_timestamp, data_submessage, heartbeat],
                        reader_proxy.unicast_locator_list().to_vec(),
                    )
                    .await;
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
                .write(
                    vec![info_dst, gap_submessage],
                    reader_proxy.unicast_locator_list().to_vec(),
                )
                .await;
        }
    }
}
