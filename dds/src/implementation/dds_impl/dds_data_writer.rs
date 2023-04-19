use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use crate::{
    builtin_topics::BuiltInTopicKey,
    implementation::{
        data_representation_builtin_endpoints::discovered_writer_data::{
            DiscoveredWriterData, WriterProxy,
        },
        rtps::{
            history_cache::{RtpsParameter, RtpsWriterCacheChange},
            messages::submessages::{AckNackSubmessage, NackFragSubmessage},
            reader_locator::RtpsReaderLocator,
            reader_proxy::RtpsReaderProxy,
            stateful_writer::RtpsStatefulWriter,
            stateless_writer::RtpsStatelessWriter,
            types::{
                ChangeKind, EntityId, EntityKey, Guid, Locator, GUID_UNKNOWN, USER_DEFINED_UNKNOWN,
            },
        },
        utils::{
            condvar::DdsCondvar,
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        instance::{InstanceHandle, HANDLE_NIL},
        qos::{PublisherQos, TopicQos},
        qos_policy::{QosPolicyId, ReliabilityQosPolicyKind, INVALID_QOS_POLICY_ID},
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, QosPolicyCount, StatusKind,
        },
    },
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
    any_data_writer_listener::AnyDataWriterListener,
    iterators::{
        ReaderLocatorListIntoIter, ReaderProxyListIntoIter, StatelessWriterChangeListIntoIter,
        WriterChangeListIntoIter,
    },
    message_receiver::MessageReceiver,
    node_listener_data_writer::ListenerDataWriterNode,
    status_condition_impl::StatusConditionImpl,
    status_listener::StatusListener,
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

pub struct DdsDataWriter<T> {
    rtps_writer: DdsRwLock<T>,
    type_name: &'static str,
    topic_name: String,
    matched_subscriptions: DdsRwLock<MatchedSubscriptions>,
    incompatible_subscriptions: DdsRwLock<IncompatibleSubscriptions>,
    enabled: DdsRwLock<bool>,
    status_listener: DdsRwLock<StatusListener<dyn AnyDataWriterListener + Send + Sync>>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    acked_by_all_condvar: DdsCondvar,
}

impl<T> DdsDataWriter<T> {
    pub fn new(
        rtps_writer: T,
        listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        mask: &[StatusKind],
        type_name: &'static str,
        topic_name: String,
    ) -> DdsShared<Self> {
        DdsShared::new(DdsDataWriter {
            rtps_writer: DdsRwLock::new(rtps_writer),
            type_name,
            topic_name,
            matched_subscriptions: DdsRwLock::new(MatchedSubscriptions::new()),
            incompatible_subscriptions: DdsRwLock::new(IncompatibleSubscriptions::new()),
            enabled: DdsRwLock::new(false),
            status_listener: DdsRwLock::new(StatusListener::new(listener, mask)),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            acked_by_all_condvar: DdsCondvar::new(),
        })
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        mask: &[StatusKind],
    ) {
        *self.status_listener.write_lock() = StatusListener::new(a_listener, mask);
    }

    pub fn is_listener_enabled(&self, status_kind: &StatusKind) -> bool {
        self.status_listener.read_lock().is_enabled(status_kind)
    }

    pub fn _trigger_on_liveliness_lost(
        &self,
        the_writer: ListenerDataWriterNode,
        status: LivelinessLostStatus,
    ) {
        if let Some(l) = self.status_listener.write_lock().listener_mut() {
            l.trigger_on_liveliness_lost(the_writer, status)
        }
    }

    pub fn _trigger_on_offered_deadline_missed(
        &self,
        the_writer: ListenerDataWriterNode,
        status: OfferedDeadlineMissedStatus,
    ) {
        if let Some(l) = self.status_listener.write_lock().listener_mut() {
            l.trigger_on_offered_deadline_missed(the_writer, status)
        }
    }

    pub fn trigger_on_offered_incompatible_qos(
        &self,
        the_writer: ListenerDataWriterNode,
        status: OfferedIncompatibleQosStatus,
    ) {
        if let Some(l) = self.status_listener.write_lock().listener_mut() {
            l.trigger_on_offered_incompatible_qos(the_writer, status)
        }
    }

    pub fn trigger_on_publication_matched(
        &self,
        the_writer: ListenerDataWriterNode,
        status: PublicationMatchedStatus,
    ) {
        if let Some(l) = self.status_listener.write_lock().listener_mut() {
            l.trigger_on_publication_matched(the_writer, status)
        }
    }

    pub fn get_publication_matched_status(&self) -> PublicationMatchedStatus {
        self.matched_subscriptions
            .write_lock()
            .get_publication_matched_status()
    }

    pub fn add_matched_publication(
        &self,
        handle: InstanceHandle,
        subscription_data: SubscriptionBuiltinTopicData,
    ) {
        self.matched_subscriptions
            .write_lock()
            .add_matched_subscription(handle, subscription_data)
    }

    pub fn remove_matched_subscription(&self, handle: InstanceHandle) {
        self.matched_subscriptions
            .write_lock()
            .remove_matched_subscription(handle)
    }

    pub fn get_matched_subscriptions(&self) -> Vec<InstanceHandle> {
        self.matched_subscriptions
            .read_lock()
            .get_matched_subscriptions()
    }

    pub fn get_matched_subscription_data(
        &self,
        handle: InstanceHandle,
    ) -> Option<SubscriptionBuiltinTopicData> {
        self.matched_subscriptions
            .read_lock()
            .get_matched_subscription_data(handle)
            .cloned()
    }

    pub fn add_offered_incompatible_qos(
        &self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        self.incompatible_subscriptions
            .write_lock()
            .add_offered_incompatible_qos(handle, incompatible_qos_policy_list)
    }

    pub fn get_offered_incompatible_qos_status(&self) -> OfferedIncompatibleQosStatus {
        self.incompatible_subscriptions
            .write_lock()
            .get_offered_incompatible_qos_status()
    }

    pub fn get_incompatible_subscriptions(&self) -> Vec<InstanceHandle> {
        self.incompatible_subscriptions
            .read_lock()
            .get_incompatible_subscriptions()
    }

    pub fn add_communication_state(&self, state: StatusKind) {
        self.status_condition
            .write_lock()
            .add_communication_state(state)
    }

    pub fn remove_communication_state(&self, state: StatusKind) {
        self.status_condition
            .write_lock()
            .remove_communication_state(state)
    }

    pub fn get_status_changes(&self) -> Vec<StatusKind> {
        self.status_condition.read_lock().get_status_changes()
    }

    pub fn enable(&self) {
        *self.enabled.write_lock() = true;
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }

    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
    }

    pub fn get_type_name(&self) -> &'static str {
        self.type_name
    }

}

impl DdsDataWriter<RtpsStatefulWriter> {
    pub fn guid(&self) -> Guid {
        self.rtps_writer.read_lock().guid()
    }

    pub fn _unicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer.read_lock().unicast_locator_list().to_vec()
    }

    pub fn _multicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer
            .read_lock()
            .multicast_locator_list()
            .to_vec()
    }

    pub fn _push_mode(&self) -> bool {
        self.rtps_writer.read_lock().push_mode()
    }

    pub fn heartbeat_period(&self) -> Duration {
        self.rtps_writer.read_lock().heartbeat_period()
    }

    pub fn data_max_size_serialized(&self) -> usize {
        self.rtps_writer.read_lock().data_max_size_serialized()
    }

    pub fn _new_change(
        &mut self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: Vec<RtpsParameter>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> RtpsWriterCacheChange {
        self.rtps_writer
            .write_lock()
            .new_change(kind, data, inline_qos, handle, timestamp)
    }

    pub fn change_list(&self) -> WriterChangeListIntoIter {
        WriterChangeListIntoIter::new(self.rtps_writer.read_lock())
    }

    pub fn _add_change(&self, change: RtpsWriterCacheChange) {
        self.rtps_writer.write_lock().add_change(change)
    }

    pub fn remove_change<F>(&self, f: F)
    where
        F: FnMut(&RtpsWriterCacheChange) -> bool,
    {
        self.rtps_writer.write_lock().remove_change(f)
    }

    pub fn matched_reader_add(&self, a_reader_proxy: RtpsReaderProxy) {
        self.rtps_writer
            .write_lock()
            .matched_reader_add(a_reader_proxy)
    }

    pub fn matched_reader_remove(&self, a_reader_guid: Guid) {
        self.rtps_writer
            .write_lock()
            .matched_reader_remove(a_reader_guid)
    }

    pub fn matched_reader_list(&self) -> ReaderProxyListIntoIter {
        ReaderProxyListIntoIter::new(self.rtps_writer.write_lock())
    }

    pub fn _is_acked_by_all(&self, a_change: &RtpsWriterCacheChange) -> bool {
        self.rtps_writer.read_lock().is_acked_by_all(a_change)
    }



    pub fn get_qos(&self) -> DataWriterQos {
        self.rtps_writer.read_lock().get_qos().clone()
    }

    pub fn set_qos(&self, qos: DataWriterQos) {
        self.rtps_writer.write_lock().set_qos(qos);
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
        self.rtps_writer.write_lock().write_w_timestamp(
            serialized_data,
            instance_serialized_key,
            handle,
            timestamp,
        )?;

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
                let changes = rtps_writer_lock.change_list();

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

    pub fn as_discovered_writer_data(
        &self,
        topic_qos: &TopicQos,
        publisher_qos: &PublisherQos,
    ) -> DiscoveredWriterData {
        let writer_qos = self.rtps_writer.read_lock().get_qos().clone();

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
                topic_name: self.topic_name.clone(),
                type_name: self.type_name.to_string(),
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
                topic_data: topic_qos.topic_data.clone(),
                group_data: publisher_qos.group_data.clone(),
            },
        }
    }
}

impl DdsDataWriter<RtpsStatelessWriter> {
    pub fn guid(&self) -> Guid {
        self.rtps_writer.read_lock().guid()
    }

    pub fn _unicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer.read_lock().unicast_locator_list().to_vec()
    }

    pub fn _multicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer
            .read_lock()
            .multicast_locator_list()
            .to_vec()
    }

    pub fn _push_mode(&self) -> bool {
        self.rtps_writer.read_lock().push_mode()
    }

    pub fn _heartbeat_period(&self) -> Duration {
        self.rtps_writer.read_lock().heartbeat_period()
    }

    pub fn _data_max_size_serialized(&self) -> usize {
        self.rtps_writer.read_lock().data_max_size_serialized()
    }

    pub fn _new_change(
        &self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: Vec<RtpsParameter>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> RtpsWriterCacheChange {
        self.rtps_writer
            .write_lock()
            .new_change(kind, data, inline_qos, handle, timestamp)
    }

    pub fn _change_list(&self) -> StatelessWriterChangeListIntoIter {
        StatelessWriterChangeListIntoIter::new(self.rtps_writer.read_lock())
    }

    pub fn _add_change(&self, change: RtpsWriterCacheChange) {
        self.rtps_writer.write_lock().add_change(change);
    }

    pub fn _remove_change<F>(&self, f: F)
    where
        F: FnMut(&RtpsWriterCacheChange) -> bool,
    {
        self.rtps_writer.write_lock().remove_change(f)
    }

    pub fn reader_locator_add(&self, a_locator: RtpsReaderLocator) {
        self.rtps_writer.write_lock().reader_locator_add(a_locator)
    }

    pub fn _reader_locator_remove(&self, a_locator: Locator) {
        self.rtps_writer
            .write_lock()
            .reader_locator_remove(a_locator)
    }

    pub fn reader_locator_list(&self) -> ReaderLocatorListIntoIter {
        ReaderLocatorListIntoIter::new(self.rtps_writer.write_lock())
    }

    pub fn write_w_timestamp(
        &self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        self.rtps_writer.write_lock().write_w_timestamp(
            serialized_data,
            instance_serialized_key,
            handle,
            timestamp,
        )
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use crate::{
        implementation::rtps::{
            endpoint::RtpsEndpoint,
            messages::RtpsMessage,
            transport::TransportWrite,
            types::{TopicKind, GUID_UNKNOWN},
            writer::RtpsWriter,
        },
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

    fn create_data_writer_test_fixture() -> DdsShared<DdsDataWriter<RtpsStatefulWriter>> {
        let rtps_writer = RtpsStatefulWriter::new(RtpsWriter::new(
            RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]),
            true,
            DURATION_ZERO,
            DURATION_ZERO,
            DURATION_ZERO,
            usize::MAX,
            DataWriterQos::default(),
        ));

        let data_writer = DdsDataWriter::new(rtps_writer, None, &[], "", String::from(""));
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
