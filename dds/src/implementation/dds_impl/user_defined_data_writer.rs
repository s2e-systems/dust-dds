use std::{collections::HashMap, sync::mpsc::SyncSender, time::Instant};

use crate::{
    builtin_topics::BuiltInTopicKey,
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
        },
        rtps::{
            messages::{
                overall_structure::RtpsMessageHeader,
                submessages::{AckNackSubmessage, NackFragSubmessage},
            },
            reader_proxy::RtpsReaderProxy,
            stateful_writer::RtpsStatefulWriter,
            transport::TransportWrite,
            types::{
                DurabilityKind, EntityId, EntityKey, Locator, ReliabilityKind, GUID_UNKNOWN,
                USER_DEFINED_UNKNOWN,
            },
        },
        utils::{
            condvar::DdsCondvar,
            shared_object::{DdsRwLock, DdsShared, DdsWeak},
        },
    },
    infrastructure::{
        instance::InstanceHandle,
        qos::{PublisherQos, QosKind},
        qos_policy::{
            DurabilityQosPolicyKind, QosPolicyId, ReliabilityQosPolicyKind, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID,
        },
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, QosPolicyCount, StatusKind,
        },
    },
    publication::{data_writer::AnyDataWriter, publisher_listener::PublisherListener},
    topic_definition::type_support::{DdsSerializedKey, DdsType},
    {
        builtin_topics::{PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
        infrastructure::{
            error::{DdsError, DdsResult},
            qos::DataWriterQos,
            time::{Duration, Time},
        },
    },
};

use super::{
    any_data_writer_listener::AnyDataWriterListener, domain_participant_impl::AnnounceKind,
    message_receiver::MessageReceiver, status_condition_impl::StatusConditionImpl,
    status_listener::StatusListener, topic_impl::TopicImpl,
    user_defined_publisher_impl::UserDefinedPublisherImpl,
};

impl PublicationMatchedStatus {
    fn increment(&mut self, subscription_handle: InstanceHandle) {
        self.total_count += 1;
        self.total_count_change += 1;
        self.last_subscription_handle = subscription_handle;
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

impl LivelinessLostStatus {
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

impl OfferedDeadlineMissedStatus {
    fn _increment(&mut self, instance_handle: InstanceHandle) {
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

impl OfferedIncompatibleQosStatus {
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

pub struct UserDefinedDataWriter {
    rtps_writer: DdsRwLock<RtpsStatefulWriter>,
    topic: DdsShared<TopicImpl>,
    publisher: DdsWeak<UserDefinedPublisherImpl>,
    publication_matched_status: DdsRwLock<PublicationMatchedStatus>,
    offered_deadline_missed_status: DdsRwLock<OfferedDeadlineMissedStatus>,
    offered_incompatible_qos_status: DdsRwLock<OfferedIncompatibleQosStatus>,
    liveliness_lost_status: DdsRwLock<LivelinessLostStatus>,
    matched_subscription_list: DdsRwLock<HashMap<InstanceHandle, SubscriptionBuiltinTopicData>>,
    enabled: DdsRwLock<bool>,
    status_listener: DdsRwLock<StatusListener<dyn AnyDataWriterListener + Send + Sync>>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    user_defined_data_send_condvar: DdsCondvar,
    acked_by_all_condvar: DdsCondvar,
    announce_sender: SyncSender<AnnounceKind>,
}

impl UserDefinedDataWriter {
    pub fn new(
        rtps_writer: RtpsStatefulWriter,
        listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        mask: &[StatusKind],
        topic: DdsShared<TopicImpl>,
        publisher: DdsWeak<UserDefinedPublisherImpl>,
        user_defined_data_send_condvar: DdsCondvar,
        announce_sender: SyncSender<AnnounceKind>,
    ) -> DdsShared<Self> {
        DdsShared::new(UserDefinedDataWriter {
            rtps_writer: DdsRwLock::new(rtps_writer),
            topic,
            publisher,
            publication_matched_status: DdsRwLock::new(PublicationMatchedStatus::default()),
            offered_deadline_missed_status: DdsRwLock::new(OfferedDeadlineMissedStatus::default()),
            offered_incompatible_qos_status: DdsRwLock::new(OfferedIncompatibleQosStatus::default()),
            liveliness_lost_status: DdsRwLock::new(LivelinessLostStatus::default()),
            matched_subscription_list: DdsRwLock::new(HashMap::new()),
            enabled: DdsRwLock::new(false),
            status_listener: DdsRwLock::new(StatusListener::new(listener, mask)),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),

            user_defined_data_send_condvar,
            acked_by_all_condvar: DdsCondvar::new(),
            announce_sender,
        })
    }
}

impl DdsShared<UserDefinedDataWriter> {
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }

    pub fn on_acknack_submessage_received(
        &self,
        acknack_submessage: &AckNackSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        if self.rtps_writer.read_lock().get_qos().reliability.kind
            == ReliabilityQosPolicyKind::Reliable
        {
            self.rtps_writer
                .write_lock()
                .on_acknack_submessage_received(
                    acknack_submessage,
                    message_receiver.source_guid_prefix(),
                );
            self.acked_by_all_condvar.notify_all();
        }
    }

    pub fn on_nack_frag_submessage_received(
        &self,
        nackfrag_submessage: &NackFragSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        if self.rtps_writer.read_lock().get_qos().reliability.kind
            == ReliabilityQosPolicyKind::Reliable
        {
            self.rtps_writer
                .write_lock()
                .on_nack_frag_submessage_received(
                    nackfrag_submessage,
                    message_receiver.source_guid_prefix(),
                );
            self.acked_by_all_condvar.notify_all();
        }
    }

    pub fn add_matched_reader(
        &self,
        discovered_reader_data: &DiscoveredReaderData,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
        publisher_status_listener: &mut StatusListener<dyn PublisherListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let is_matched_topic_name = discovered_reader_data
            .subscription_builtin_topic_data
            .topic_name
            == self.topic.get_name();
        let is_matched_type_name = discovered_reader_data
            .subscription_builtin_topic_data
            .type_name
            == self.topic.get_type_name();

        if is_matched_topic_name && is_matched_type_name {
            let add_matched_reader_result = add_discovered_reader(
                &mut self.rtps_writer.write_lock(),
                discovered_reader_data,
                &self.get_publisher().get_qos(),
                default_unicast_locator_list,
                default_multicast_locator_list,
            );

            match add_matched_reader_result {
                Ok(_) => {
                    let instance_handle = discovered_reader_data.get_serialized_key().into();
                    let insert_result = self.matched_subscription_list.write_lock().insert(
                        instance_handle,
                        discovered_reader_data
                            .subscription_builtin_topic_data
                            .clone(),
                    );
                    match insert_result {
                        Some(value)
                            if value != discovered_reader_data.subscription_builtin_topic_data =>
                        {
                            self.on_publication_matched(
                                instance_handle,
                                publisher_status_listener,
                                participant_status_listener,
                            )
                        }
                        None => self.on_publication_matched(
                            instance_handle,
                            publisher_status_listener,
                            participant_status_listener,
                        ),
                        _ => (),
                    }
                }
                Err(incompatible_qos_policy_list) => self.on_offered_incompatible_qos(
                    incompatible_qos_policy_list,
                    publisher_status_listener,
                    participant_status_listener,
                ),
            }
        }
    }

    pub fn remove_matched_reader(
        &self,
        discovered_reader_handle: InstanceHandle,
        publisher_status_listener: &mut StatusListener<dyn PublisherListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        if let Some(r) = self
            .matched_subscription_list
            .write_lock()
            .remove(&discovered_reader_handle)
        {
            self.rtps_writer
                .write_lock()
                .matched_reader_remove(r.key.value.into());

            self.on_publication_matched(
                discovered_reader_handle,
                publisher_status_listener,
                participant_status_listener,
            )
        }
    }

    pub fn register_instance_w_timestamp(
        &self,
        instance_serialized_key: DdsSerializedKey,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .write_lock()
            .register_instance_w_timestamp(instance_serialized_key, timestamp)
    }

    pub fn unregister_instance_w_timestamp(
        &self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .write_lock()
            .unregister_instance_w_timestamp(instance_serialized_key, handle, timestamp)
    }

    pub fn get_key_value<Foo>(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        key_holder.set_key_fields_from_serialized_key(
            self.rtps_writer
                .write_lock()
                .get_key_value(handle)
                .ok_or(DdsError::BadParameter)?,
        )
    }

    pub fn lookup_instance(
        &self,
        instance_serialized_key: DdsSerializedKey,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self
            .rtps_writer
            .write_lock()
            .lookup_instance(instance_serialized_key))
    }

    pub fn write_w_timestamp(
        &self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer.write_lock().write_w_timestamp(
            serialized_data,
            instance_serialized_key,
            handle,
            timestamp,
        )?;

        self.user_defined_data_send_condvar.notify_all();

        Ok(())
    }

    pub fn dispose_w_timestamp(
        &self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer.write_lock().dispose_w_timestamp(
            instance_serialized_key,
            handle,
            timestamp,
        )
    }

    pub fn wait_for_acknowledgments(&self, max_wait: Duration) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let start_time = Instant::now();

        while start_time.elapsed() < std::time::Duration::from(max_wait) {
            {
                // This is done in an inner scope such that the lock can be dropped and new acknowledgements
                // can be processed when received
                let rtps_writer_lock = self.rtps_writer.write_lock();
                let changes = rtps_writer_lock.writer_cache().changes();

                if changes
                    .iter()
                    .map(|c| rtps_writer_lock.is_acked_by_all(c))
                    .all(|r| r)
                {
                    return Ok(());
                }
            }

            let elapsed = Duration::from(start_time.elapsed());
            if elapsed >= max_wait {
                return Err(DdsError::Timeout);
            }
            let duration_until_timeout = max_wait - elapsed;
            self.acked_by_all_condvar
                .wait_timeout(duration_until_timeout)
                .ok();
        }
        Err(DdsError::Timeout)
    }

    pub fn get_liveliness_lost_status(&self) -> LivelinessLostStatus {
        self.liveliness_lost_status.write_lock().read_and_reset()
    }

    pub fn get_offered_deadline_missed_status(&self) -> OfferedDeadlineMissedStatus {
        self.offered_deadline_missed_status
            .write_lock()
            .read_and_reset()
    }

    pub fn get_offered_incompatible_qos_status(&self) -> OfferedIncompatibleQosStatus {
        self.offered_incompatible_qos_status
            .write_lock()
            .read_and_reset()
    }

    pub fn get_publication_matched_status(&self) -> PublicationMatchedStatus {
        self.status_condition
            .write_lock()
            .remove_communication_state(StatusKind::PublicationMatched);
        self.publication_matched_status
            .write_lock()
            .read_and_reset(self.matched_subscription_list.read_lock().len() as i32)
    }

    pub fn get_topic(&self) -> DdsShared<TopicImpl> {
        self.topic.clone()
    }

    pub fn get_publisher(&self) -> DdsShared<UserDefinedPublisherImpl> {
        self.publisher
            .upgrade()
            .expect("Parent publisher of data writer must exist")
    }

    pub fn assert_liveliness(&self) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        todo!()
    }

    pub fn get_matched_subscription_data(
        &self,
        subscription_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.matched_subscription_list
            .read_lock()
            .get(&subscription_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    pub fn get_matched_subscriptions(&self) -> DdsResult<Vec<InstanceHandle>> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self
            .matched_subscription_list
            .read_lock()
            .iter()
            .map(|(&key, _)| key)
            .collect())
    }

    pub fn set_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        if self.is_enabled() {
            self.rtps_writer
                .write_lock()
                .get_qos()
                .check_immutability(&qos)?;
        }

        self.rtps_writer.write_lock().set_qos(qos)?;

        if self.is_enabled() {
            self.announce_sender
                .send(AnnounceKind::CreatedDataWriter(
                    self.as_discovered_writer_data(),
                ))
                .ok();
        }

        Ok(())
    }

    pub fn get_qos(&self) -> DataWriterQos {
        self.rtps_writer.read_lock().get_qos().clone()
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        mask: &[StatusKind],
    ) {
        *self.status_listener.write_lock() = StatusListener::new(a_listener, mask);
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_status_changes(&self) -> Vec<StatusKind> {
        self.status_condition.read_lock().get_status_changes()
    }

    pub fn enable(&self) -> DdsResult<()> {
        self.announce_sender
            .send(AnnounceKind::CreatedDataWriter(
                self.as_discovered_writer_data(),
            ))
            .ok();
        *self.enabled.write_lock() = true;

        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_writer.read_lock().guid().into()
    }

    pub fn as_discovered_writer_data(&self) -> DiscoveredWriterData {
        let writer_qos = self.rtps_writer.read_lock().get_qos().clone();
        let topic_qos = self.topic.get_qos();
        let publisher_qos = self.get_publisher().get_qos();

        DiscoveredWriterData {
            writer_proxy: WriterProxy {
                remote_writer_guid: self.rtps_writer.read_lock().guid(),
                unicast_locator_list: self.rtps_writer.read_lock().unicast_locator_list().to_vec(),
                multicast_locator_list: self
                    .rtps_writer
                    .read_lock()
                    .multicast_locator_list()
                    .to_vec(),
                data_max_size_serialized: None,
                remote_group_entity_id: EntityId::new(EntityKey::new([0; 3]), USER_DEFINED_UNKNOWN),
            },

            publication_builtin_topic_data: PublicationBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.rtps_writer.read_lock().guid().into(),
                },
                participant_key: BuiltInTopicKey {
                    value: GUID_UNKNOWN.into(),
                },
                topic_name: self.topic.get_name(),
                type_name: self.topic.get_type_name().to_string(),
                durability: writer_qos.durability.clone(),
                deadline: writer_qos.deadline.clone(),
                latency_budget: writer_qos.latency_budget.clone(),
                liveliness: writer_qos.liveliness.clone(),
                reliability: writer_qos.reliability.clone(),
                lifespan: writer_qos.lifespan.clone(),
                user_data: writer_qos.user_data.clone(),
                ownership: writer_qos.ownership.clone(),
                destination_order: writer_qos.destination_order,
                presentation: publisher_qos.presentation.clone(),
                partition: publisher_qos.partition.clone(),
                topic_data: topic_qos.topic_data,
                group_data: publisher_qos.group_data,
            },
        }
    }

    pub fn send_message(
        &self,
        header: RtpsMessageHeader,
        transport: &mut impl TransportWrite,
        now: Time,
    ) {
        self.rtps_writer
            .write_lock()
            .send_message(header, transport, now);
    }

    fn on_offered_incompatible_qos(
        &self,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
        publisher_status_listener: &mut StatusListener<dyn PublisherListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        self.offered_incompatible_qos_status
            .write_lock()
            .increment(incompatible_qos_policy_list);

        self.trigger_on_offered_incompatible_qos_listener(
            &mut self.status_listener.write_lock(),
            publisher_status_listener,
            participant_status_listener,
        );

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::OfferedIncompatibleQos);
    }

    fn on_publication_matched(
        &self,
        instance_handle: InstanceHandle,
        publisher_status_listener: &mut StatusListener<dyn PublisherListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        self.publication_matched_status
            .write_lock()
            .increment(instance_handle);

        self.trigger_on_publication_matched_listener(
            &mut self.status_listener.write_lock(),
            publisher_status_listener,
            participant_status_listener,
        );

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::PublicationMatched);
    }

    fn trigger_on_publication_matched_listener(
        &self,
        writer_status_listener: &mut StatusListener<dyn AnyDataWriterListener + Send + Sync>,
        publisher_status_listener: &mut StatusListener<dyn PublisherListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let publication_matched_status_kind = &StatusKind::PublicationMatched;

        if writer_status_listener.is_enabled(publication_matched_status_kind) {
            writer_status_listener
                .listener_mut()
                .trigger_on_publication_matched(self)
        } else if publisher_status_listener.is_enabled(publication_matched_status_kind) {
            publisher_status_listener
                .listener_mut()
                .on_publication_matched(self, self.get_publication_matched_status())
        } else if participant_status_listener.is_enabled(publication_matched_status_kind) {
            participant_status_listener
                .listener_mut()
                .on_publication_matched(self, self.get_publication_matched_status())
        }
    }

    fn trigger_on_offered_incompatible_qos_listener(
        &self,
        writer_status_listener: &mut StatusListener<dyn AnyDataWriterListener + Send + Sync>,
        publisher_status_listener: &mut StatusListener<dyn PublisherListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let offerered_incompatible_qos_status_kind = &StatusKind::OfferedIncompatibleQos;
        if writer_status_listener.is_enabled(offerered_incompatible_qos_status_kind) {
            writer_status_listener
                .listener_mut()
                .trigger_on_offered_incompatible_qos(self)
        } else if publisher_status_listener.is_enabled(offerered_incompatible_qos_status_kind) {
            publisher_status_listener
                .listener_mut()
                .on_offered_incompatible_qos(self, self.get_offered_incompatible_qos_status())
        } else if participant_status_listener.is_enabled(offerered_incompatible_qos_status_kind) {
            participant_status_listener
                .listener_mut()
                .on_offered_incompatible_qos(self, self.get_offered_incompatible_qos_status())
        }
    }
}

impl AnyDataWriter for DdsShared<UserDefinedDataWriter> {}

//// Helper functions
fn get_discovered_reader_incompatible_qos_policy_list(
    writer: &mut RtpsStatefulWriter,
    discovered_reader_data: &SubscriptionBuiltinTopicData,
    publisher_qos: &PublisherQos,
) -> Vec<QosPolicyId> {
    let mut incompatible_qos_policy_list = Vec::new();
    if writer.get_qos().durability < discovered_reader_data.durability {
        incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
    }
    if publisher_qos.presentation.access_scope < discovered_reader_data.presentation.access_scope
        || publisher_qos.presentation.coherent_access
            != discovered_reader_data.presentation.coherent_access
        || publisher_qos.presentation.ordered_access
            != discovered_reader_data.presentation.ordered_access
    {
        incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
    }
    if writer.get_qos().deadline < discovered_reader_data.deadline {
        incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
    }
    if writer.get_qos().latency_budget < discovered_reader_data.latency_budget {
        incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
    }
    if writer.get_qos().liveliness < discovered_reader_data.liveliness {
        incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
    }
    if writer.get_qos().reliability.kind < discovered_reader_data.reliability.kind {
        incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
    }
    if writer.get_qos().destination_order < discovered_reader_data.destination_order {
        incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
    }
    incompatible_qos_policy_list
}

fn add_discovered_reader(
    writer: &mut RtpsStatefulWriter,
    discovered_reader_data: &DiscoveredReaderData,
    publisher_qos: &PublisherQos,
    default_unicast_locator_list: &[Locator],
    default_multicast_locator_list: &[Locator],
) -> Result<(), Vec<QosPolicyId>> {
    let incompatible_qos_policy_list = get_discovered_reader_incompatible_qos_policy_list(
        writer,
        &discovered_reader_data.subscription_builtin_topic_data,
        publisher_qos,
    );

    if incompatible_qos_policy_list.is_empty() {
        let unicast_locator_list = if discovered_reader_data
            .reader_proxy
            .unicast_locator_list
            .is_empty()
        {
            default_unicast_locator_list
        } else {
            discovered_reader_data
                .reader_proxy
                .unicast_locator_list
                .as_ref()
        };

        let multicast_locator_list = if discovered_reader_data
            .reader_proxy
            .multicast_locator_list
            .is_empty()
        {
            default_multicast_locator_list
        } else {
            discovered_reader_data
                .reader_proxy
                .multicast_locator_list
                .as_ref()
        };

        let proxy_reliability = match discovered_reader_data
            .subscription_builtin_topic_data
            .reliability
            .kind
        {
            ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
        };

        let proxy_durability = match discovered_reader_data
            .subscription_builtin_topic_data
            .durability
            .kind
        {
            DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
            DurabilityQosPolicyKind::TransientLocal => DurabilityKind::TransientLocal,
        };

        let reader_proxy = RtpsReaderProxy::new(
            discovered_reader_data.reader_proxy.remote_reader_guid,
            discovered_reader_data.reader_proxy.remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            discovered_reader_data.reader_proxy.expects_inline_qos,
            true,
            proxy_reliability,
            proxy_durability,
        );

        writer.matched_reader_add(reader_proxy);

        Ok(())
    } else {
        Err(incompatible_qos_policy_list)
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use crate::{
        implementation::rtps::{
            endpoint::RtpsEndpoint,
            messages::RtpsMessage,
            types::{TopicKind, GUID_UNKNOWN},
            writer::RtpsWriter,
        },
        infrastructure::qos::TopicQos,
        infrastructure::time::DURATION_ZERO,
        topic_definition::type_support::{DdsSerialize, DdsSerializedKey, Endianness},
    };

    use mockall::mock;

    use super::*;

    mock! {
        Transport{}

        impl TransportWrite for Transport {
            fn write<'a>(&'a mut self, message: &RtpsMessage<'a>, destination_locator_list: &[Locator]);
        }
    }

    struct MockFoo {}

    impl DdsSerialize for MockFoo {
        fn serialize<W: Write, E: Endianness>(&self, _writer: W) -> DdsResult<()> {
            Ok(())
        }
    }

    impl DdsType for MockFoo {
        fn type_name() -> &'static str {
            todo!()
        }
    }

    struct MockKeyedFoo {
        key: Vec<u8>,
    }

    impl DdsType for MockKeyedFoo {
        fn type_name() -> &'static str {
            todo!()
        }

        fn has_key() -> bool {
            true
        }

        fn get_serialized_key(&self) -> DdsSerializedKey {
            self.key.as_slice().into()
        }

        fn set_key_fields_from_serialized_key(&mut self, key: &DdsSerializedKey) -> DdsResult<()> {
            self.key = key.as_ref().to_vec();
            Ok(())
        }
    }

    impl DdsSerialize for MockKeyedFoo {
        fn serialize<W: Write, E: Endianness>(&self, _writer: W) -> DdsResult<()> {
            Ok(())
        }
    }

    fn create_data_writer_test_fixture() -> DdsShared<UserDefinedDataWriter> {
        let (sender, _) = std::sync::mpsc::sync_channel(1);
        let dummy_topic = TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            "",
            "",
            None,
            &[],
            DdsWeak::new(),
            sender.clone(),
        );

        let rtps_writer = RtpsStatefulWriter::new(RtpsWriter::new(
            RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]),
            true,
            DURATION_ZERO,
            DURATION_ZERO,
            DURATION_ZERO,
            usize::MAX,
            DataWriterQos::default(),
        ));

        let data_writer = UserDefinedDataWriter::new(
            rtps_writer,
            None,
            &[],
            dummy_topic,
            DdsWeak::new(),
            DdsCondvar::new(),
            sender,
        );
        *data_writer.enabled.write_lock() = true;
        data_writer
    }

    #[test]
    fn get_key_value_known_instance() {
        let data_writer = create_data_writer_test_fixture();

        let instance_handle = data_writer
            .register_instance_w_timestamp(
                MockKeyedFoo { key: vec![1, 2] }.get_serialized_key(),
                Time::new(0, 0),
            )
            .unwrap()
            .unwrap();

        let mut keyed_foo = MockKeyedFoo { key: vec![] };
        data_writer
            .get_key_value(&mut keyed_foo, instance_handle)
            .unwrap();
        assert_eq!(keyed_foo.key, vec![1, 2]);
    }

    #[test]
    fn get_key_value_unknown_instance() {
        let data_writer = create_data_writer_test_fixture();
        let not_registered_foo = MockKeyedFoo { key: vec![1, 16] };
        let registered_foo = MockKeyedFoo { key: vec![1, 2] };
        data_writer
            .register_instance_w_timestamp(registered_foo.get_serialized_key(), Time::new(0, 0))
            .unwrap();

        let mut keyed_foo = MockKeyedFoo { key: vec![] };
        assert_eq!(
            data_writer.get_key_value(
                &mut keyed_foo,
                not_registered_foo.get_serialized_key().into()
            ),
            Err(DdsError::BadParameter)
        );
    }
}
