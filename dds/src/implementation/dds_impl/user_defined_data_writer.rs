use std::{collections::HashMap, time::Instant};

use crate::{
    builtin_topics::BuiltInTopicKey,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::{DiscoveredWriterData, WriterProxy},
        },
        rtps::{
            messages::submessages::AckNackSubmessage,
            reader_proxy::RtpsReaderProxy,
            stateful_writer::RtpsStatefulWriter,
            transport::TransportWrite,
            types::{EntityId, EntityKey, Locator, GUID_UNKNOWN, USER_DEFINED_UNKNOWN},
            utils::clock::StdTimer,
        },
        utils::{
            condvar::DdsCondvar,
            shared_object::{DdsRwLock, DdsShared, DdsWeak},
        },
    },
    infrastructure::{
        instance::InstanceHandle,
        qos::QosKind,
        qos_policy::{QosPolicyId, ReliabilityQosPolicyKind},
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, QosPolicyCount, StatusKind,
        },
    },
    publication::data_writer::AnyDataWriter,
    topic_definition::type_support::{DdsSerialize, DdsType},
    {
        builtin_topics::{PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
        infrastructure::{
            error::{DdsError, DdsResult},
            qos::DataWriterQos,
            qos_policy::{
                DEADLINE_QOS_POLICY_ID, DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID,
                LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
                RELIABILITY_QOS_POLICY_ID,
            },
            time::{Duration, Time},
        },
    },
};

use super::{
    any_data_writer_listener::AnyDataWriterListener, message_receiver::MessageReceiver,
    status_condition_impl::StatusConditionImpl, topic_impl::TopicImpl,
    user_defined_publisher::UserDefinedPublisher,
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
    rtps_writer: DdsRwLock<RtpsStatefulWriter<StdTimer>>,
    topic: DdsShared<TopicImpl>,
    publisher: DdsWeak<UserDefinedPublisher>,
    publication_matched_status: DdsRwLock<PublicationMatchedStatus>,
    offered_deadline_missed_status: DdsRwLock<OfferedDeadlineMissedStatus>,
    offered_incompatible_qos_status: DdsRwLock<OfferedIncompatibleQosStatus>,
    liveliness_lost_status: DdsRwLock<LivelinessLostStatus>,
    matched_subscription_list: DdsRwLock<HashMap<InstanceHandle, SubscriptionBuiltinTopicData>>,
    enabled: DdsRwLock<bool>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    listener: DdsRwLock<Option<Box<dyn AnyDataWriterListener + Send + Sync>>>,
    listener_status_mask: DdsRwLock<Vec<StatusKind>>,
    user_defined_data_send_condvar: DdsCondvar,
}

impl UserDefinedDataWriter {
    pub fn new(
        rtps_writer: RtpsStatefulWriter<StdTimer>,
        listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        mask: &[StatusKind],
        topic: DdsShared<TopicImpl>,
        publisher: DdsWeak<UserDefinedPublisher>,
        user_defined_data_send_condvar: DdsCondvar,
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
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            listener: DdsRwLock::new(listener),
            listener_status_mask: DdsRwLock::new(mask.to_vec()),
            user_defined_data_send_condvar,
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
        }
    }

    pub fn add_matched_reader(
        &self,
        discovered_reader_data: &DiscoveredReaderData,
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
    ) {
        let reader_info = &discovered_reader_data.subscription_builtin_topic_data;
        let writer_topic_name = self.topic.get_name();
        let writer_type_name = self.topic.get_type_name();

        if reader_info.topic_name == writer_topic_name && reader_info.type_name == writer_type_name
        {
            let incompatible_qos_policy_list =
                self.get_discovered_reader_incompatible_qos_policy_list(discovered_reader_data);
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

                let reader_proxy = RtpsReaderProxy::new(
                    discovered_reader_data.reader_proxy.remote_reader_guid,
                    discovered_reader_data.reader_proxy.remote_group_entity_id,
                    unicast_locator_list,
                    multicast_locator_list,
                    discovered_reader_data.reader_proxy.expects_inline_qos,
                    true,
                );

                self.rtps_writer
                    .write_lock()
                    .matched_reader_add(reader_proxy);

                let instance_handle = discovered_reader_data.get_serialized_key().into();
                self.matched_subscription_list
                    .write_lock()
                    .insert(instance_handle, reader_info.clone());

                self.on_publication_matched(instance_handle);
            } else {
                self.on_offered_incompatible_qos(incompatible_qos_policy_list)
            }
        }
    }

    fn get_discovered_reader_incompatible_qos_policy_list(
        &self,
        discovered_reader_data: &DiscoveredReaderData,
    ) -> Vec<QosPolicyId> {
        let reader_info = &discovered_reader_data.subscription_builtin_topic_data;
        let parent_publisher_qos = self.get_publisher().get_qos();
        let qos = self.rtps_writer.read_lock().get_qos().clone();
        let mut incompatible_qos_policy_list = Vec::new();
        if qos.durability < reader_info.durability {
            incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
        }
        if parent_publisher_qos.presentation.access_scope < reader_info.presentation.access_scope
            || parent_publisher_qos.presentation.coherent_access
                != reader_info.presentation.coherent_access
            || parent_publisher_qos.presentation.ordered_access
                != reader_info.presentation.ordered_access
        {
            incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
        }
        if qos.deadline < reader_info.deadline {
            incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
        }
        if qos.latency_budget < reader_info.latency_budget {
            incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
        }
        if qos.liveliness < reader_info.liveliness {
            incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
        }
        if qos.reliability.kind < reader_info.reliability.kind {
            incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
        }
        if qos.destination_order < reader_info.destination_order {
            incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
        }
        incompatible_qos_policy_list
    }

    pub fn remove_matched_reader(&self, discovered_reader_handle: InstanceHandle) {
        if let Some(r) = self
            .matched_subscription_list
            .write_lock()
            .remove(&discovered_reader_handle)
        {
            self.rtps_writer
                .write_lock()
                .matched_reader_remove(r.key.value.into());

            self.on_publication_matched(discovered_reader_handle)
        }
    }

    pub fn register_instance_w_timestamp<Foo>(
        &self,
        instance: &Foo,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>>
    where
        Foo: DdsType + DdsSerialize,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .write_lock()
            .register_instance_w_timestamp(instance, timestamp)
    }

    pub fn unregister_instance_w_timestamp<Foo>(
        &self,
        instance: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()>
    where
        Foo: DdsType + DdsSerialize,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .write_lock()
            .unregister_instance_w_timestamp(instance, handle, timestamp)
    }

    pub fn get_key_value<Foo>(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .write_lock()
            .get_key_value(key_holder, handle)
    }

    pub fn lookup_instance<Foo>(&self, instance: &Foo) -> DdsResult<Option<InstanceHandle>>
    where
        Foo: DdsType,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self.rtps_writer.write_lock().lookup_instance(instance))
    }

    pub fn write_w_timestamp<Foo>(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()>
    where
        Foo: DdsType + DdsSerialize,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .write_lock()
            .write_w_timestamp(data, handle, timestamp)?;

        self.user_defined_data_send_condvar.notify_all();

        Ok(())
    }

    pub fn dispose_w_timestamp<Foo>(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .write_lock()
            .dispose_w_timestamp(data, handle, timestamp)
    }

    pub fn wait_for_acknowledgments(&self, max_wait: Duration) -> DdsResult<()> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        let start_time = Instant::now();

        let max_wait_time_std = std::time::Duration::new(max_wait.sec() as u64, max_wait.nanosec());

        while start_time.elapsed() < max_wait_time_std {
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

            std::thread::sleep(std::time::Duration::from_millis(20));
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
        self.publication_matched_status
            .write_lock()
            .read_and_reset(self.matched_subscription_list.read_lock().len() as i32)
    }

    pub fn get_topic(&self) -> DdsShared<TopicImpl> {
        self.topic.clone()
    }

    pub fn get_publisher(&self) -> DdsShared<UserDefinedPublisher> {
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
            self.get_publisher()
                .get_participant()
                .announce_created_datawriter(self.as_discovered_writer_data())?;
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
        *self.listener.write_lock() = a_listener;
        *self.listener_status_mask.write_lock() = mask.to_vec();
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    pub fn enable(&self) -> DdsResult<()> {
        if !self.get_publisher().is_enabled() {
            return Err(DdsError::PreconditionNotMet(
                "Parent publisher disabled".to_string(),
            ));
        }

        self.get_publisher()
            .get_participant()
            .announce_created_datawriter(self.as_discovered_writer_data())?;
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

    pub fn send_message(&self, transport: &mut impl TransportWrite) {
        self.rtps_writer.write_lock().send_message(transport);
    }

    fn on_publication_matched(&self, instance_handle: InstanceHandle) {
        self.publication_matched_status
            .write_lock()
            .increment(instance_handle);

        match self.listener.write_lock().as_mut() {
            Some(l)
                if self
                    .listener_status_mask
                    .read_lock()
                    .contains(&StatusKind::PublicationMatched) =>
            {
                l.trigger_on_publication_matched(self)
            }
            _ => self.get_publisher().on_publication_matched(self),
        }

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::PublicationMatched);
    }

    fn on_offered_incompatible_qos(&self, incompatible_qos_policy_list: Vec<QosPolicyId>) {
        self.offered_incompatible_qos_status
            .write_lock()
            .increment(incompatible_qos_policy_list);

        match self.listener.write_lock().as_mut() {
            Some(l)
                if self
                    .listener_status_mask
                    .read_lock()
                    .contains(&StatusKind::OfferedIncompatibleQos) =>
            {
                l.trigger_on_offered_incompatible_qos(self)
            }
            _ => self.get_publisher().on_offered_incompatible_qos(self),
        }

        self.status_condition
            .write_lock()
            .add_communication_state(StatusKind::OfferedIncompatibleQos);
    }
}

impl AnyDataWriter for DdsShared<UserDefinedDataWriter> {}

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
        topic_definition::type_support::{DdsSerializedKey, Endianness},
    };

    use mockall::mock;

    use super::*;

    mock! {
        Transport{}

        impl TransportWrite for Transport {
            fn write<'a>(&'a mut self, message: &RtpsMessage<'a>, destination_locator: Locator);
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
        let dummy_topic = TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            "",
            "",
            None,
            &[],
            DdsWeak::new(),
        );

        let rtps_writer = RtpsStatefulWriter::new(RtpsWriter::new(
            RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]),
            true,
            DURATION_ZERO,
            DURATION_ZERO,
            DURATION_ZERO,
            None,
            DataWriterQos::default(),
        ));

        let data_writer = UserDefinedDataWriter::new(
            rtps_writer,
            None,
            &[],
            dummy_topic,
            DdsWeak::new(),
            DdsCondvar::new(),
        );
        *data_writer.enabled.write_lock() = true;
        data_writer
    }

    #[test]
    fn get_key_value_known_instance() {
        let data_writer = create_data_writer_test_fixture();

        let instance_handle = data_writer
            .register_instance_w_timestamp(
                &MockKeyedFoo { key: vec![1, 2] },
                Time { sec: 0, nanosec: 0 },
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
            .register_instance_w_timestamp(&registered_foo, Time { sec: 0, nanosec: 0 })
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
