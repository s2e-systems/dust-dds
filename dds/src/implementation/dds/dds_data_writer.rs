use std::collections::{HashMap, HashSet};

use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData},
    implementation::{
        data_representation_builtin_endpoints::discovered_writer_data::{
            DiscoveredWriterData, WriterProxy,
        },
        rtps::{
            history_cache::RtpsWriterCacheChange,
            messages::{
                submessage_elements::ParameterList,
                submessages::{AckNackSubmessageRead, NackFragSubmessageRead},
            },
            reader_locator::{RtpsReaderLocator, WriterAssociatedReaderLocator},
            reader_proxy::{RtpsReaderProxy, WriterAssociatedReaderProxy},
            stateful_writer::RtpsStatefulWriter,
            stateless_writer::RtpsStatelessWriter,
            types::{
                ChangeKind, EntityId, EntityKey, Guid, Locator, GUID_UNKNOWN, USER_DEFINED_UNKNOWN,
            },
        },
        utils::actor,
    },
    infrastructure::{
        instance::{InstanceHandle, HANDLE_NIL},
        qos::{PublisherQos, TopicQos},
        qos_policy::{QosPolicyId, ReliabilityQosPolicyKind, INVALID_QOS_POLICY_ID},
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, QosPolicyCount,
        },
    },
    topic_definition::type_support::{DdsSerializedKey, DdsType},
    {
        builtin_topics::SubscriptionBuiltinTopicData,
        infrastructure::{
            error::{DdsError, DdsResult},
            qos::DataWriterQos,
            time::{Duration, Time},
        },
    },
};

use super::message_receiver::MessageReceiver;

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
    rtps_writer: T,
    type_name: &'static str,
    topic_name: String,
    matched_subscriptions: MatchedSubscriptions,
    incompatible_subscriptions: IncompatibleSubscriptions,
    enabled: bool,
}

impl<T> actor::Actor for DdsDataWriter<T> {}

pub struct EnableMessage;

impl actor::Message for EnableMessage {
    type Result = ();
}

impl<T> actor::Handler<EnableMessage> for DdsDataWriter<T> {
    fn handle(&mut self, _message: EnableMessage) -> <EnableMessage as actor::Message>::Result {
        self.enable();
    }
}

impl<T> DdsDataWriter<T> {
    pub fn new(rtps_writer: T, type_name: &'static str, topic_name: String) -> Self {
        DdsDataWriter {
            rtps_writer,
            type_name,
            topic_name,
            matched_subscriptions: MatchedSubscriptions::new(),
            incompatible_subscriptions: IncompatibleSubscriptions::new(),
            enabled: false,
        }
    }

    pub fn get_publication_matched_status(&mut self) -> PublicationMatchedStatus {
        self.matched_subscriptions.get_publication_matched_status()
    }

    pub fn add_matched_publication(
        &mut self,
        handle: InstanceHandle,
        subscription_data: SubscriptionBuiltinTopicData,
    ) {
        self.matched_subscriptions
            .add_matched_subscription(handle, subscription_data)
    }

    pub fn remove_matched_subscription(&mut self, handle: InstanceHandle) {
        self.matched_subscriptions
            .remove_matched_subscription(handle)
    }

    pub fn get_matched_subscriptions(&self) -> Vec<InstanceHandle> {
        self.matched_subscriptions.get_matched_subscriptions()
    }

    pub fn get_matched_subscription_data(
        &self,
        handle: InstanceHandle,
    ) -> Option<SubscriptionBuiltinTopicData> {
        self.matched_subscriptions
            .get_matched_subscription_data(handle)
            .cloned()
    }

    pub fn add_offered_incompatible_qos(
        &mut self,
        handle: InstanceHandle,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
    ) {
        self.incompatible_subscriptions
            .add_offered_incompatible_qos(handle, incompatible_qos_policy_list)
    }

    pub fn get_offered_incompatible_qos_status(&mut self) -> OfferedIncompatibleQosStatus {
        self.incompatible_subscriptions
            .get_offered_incompatible_qos_status()
    }

    pub fn get_offered_deadline_missed_status(&self) -> OfferedDeadlineMissedStatus {
        todo!()
    }

    pub fn get_liveliness_lost_status(&self) -> LivelinessLostStatus {
        todo!()
    }

    pub fn get_incompatible_subscriptions(&self) -> Vec<InstanceHandle> {
        self.incompatible_subscriptions
            .get_incompatible_subscriptions()
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
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
        self.rtps_writer.guid()
    }

    pub fn _unicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer.unicast_locator_list().to_vec()
    }

    pub fn _multicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer.multicast_locator_list().to_vec()
    }

    pub fn _push_mode(&self) -> bool {
        self.rtps_writer.push_mode()
    }

    pub fn heartbeat_period(&self) -> Duration {
        self.rtps_writer.heartbeat_period()
    }

    pub fn data_max_size_serialized(&self) -> usize {
        self.rtps_writer.data_max_size_serialized()
    }

    pub fn _new_change(
        &mut self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: ParameterList,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> RtpsWriterCacheChange {
        self.rtps_writer
            .new_change(kind, data, inline_qos, handle, timestamp)
    }

    pub fn change_list(&self) -> &[RtpsWriterCacheChange] {
        self.rtps_writer.change_list()
    }

    pub fn _add_change(&mut self, change: RtpsWriterCacheChange) {
        self.rtps_writer.add_change(change)
    }

    pub fn remove_change<F>(&mut self, f: F)
    where
        F: FnMut(&RtpsWriterCacheChange) -> bool,
    {
        self.rtps_writer.remove_change(f)
    }

    pub fn matched_reader_add(&mut self, a_reader_proxy: RtpsReaderProxy) {
        self.rtps_writer.matched_reader_add(a_reader_proxy)
    }

    pub fn matched_reader_remove(&mut self, a_reader_guid: Guid) {
        self.rtps_writer.matched_reader_remove(a_reader_guid)
    }

    pub fn matched_reader_list(&mut self) -> Vec<WriterAssociatedReaderProxy> {
        self.rtps_writer.matched_reader_list()
    }

    pub fn _is_acked_by_all(&self, a_change: &RtpsWriterCacheChange) -> bool {
        self.rtps_writer.is_acked_by_all(a_change)
    }

    pub fn get_qos(&self) -> DataWriterQos {
        self.rtps_writer.get_qos().clone()
    }

    pub fn set_qos(&mut self, qos: DataWriterQos) {
        self.rtps_writer.set_qos(qos);
    }

    pub fn on_acknack_submessage_received(
        &mut self,
        acknack_submessage: &AckNackSubmessageRead,
        message_receiver: &MessageReceiver,
    ) {
        if self.rtps_writer.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            self.rtps_writer.on_acknack_submessage_received(
                acknack_submessage,
                message_receiver.source_guid_prefix(),
            );
        }
    }

    pub fn on_nack_frag_submessage_received(
        &mut self,
        nackfrag_submessage: &NackFragSubmessageRead,
        message_receiver: &MessageReceiver,
    ) {
        if self.rtps_writer.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            self.rtps_writer.on_nack_frag_submessage_received(
                nackfrag_submessage,
                message_receiver.source_guid_prefix(),
            );
        }
    }

    pub fn register_instance_w_timestamp(
        &mut self,
        instance_serialized_key: DdsSerializedKey,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .register_instance_w_timestamp(instance_serialized_key, timestamp)
    }

    pub fn unregister_instance_w_timestamp(
        &mut self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .unregister_instance_w_timestamp(instance_serialized_key, handle, timestamp)
    }

    pub fn get_key_value<Foo>(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        key_holder.set_key_fields_from_serialized_key(
            self.rtps_writer
                .get_key_value(handle)
                .ok_or(DdsError::BadParameter)?,
        )
    }

    pub fn lookup_instance(
        &self,
        instance_serialized_key: DdsSerializedKey,
    ) -> DdsResult<Option<InstanceHandle>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        Ok(self.rtps_writer.lookup_instance(instance_serialized_key))
    }

    pub fn write_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        self.rtps_writer.write_w_timestamp(
            serialized_data,
            instance_serialized_key,
            handle,
            timestamp,
        )?;

        Ok(())
    }

    pub fn dispose_w_timestamp(
        &mut self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.rtps_writer
            .dispose_w_timestamp(instance_serialized_key, handle, timestamp)
    }

    pub fn are_all_changes_acknowledge(&self) -> bool {
        let changes = self.rtps_writer.change_list();

        changes
            .iter()
            .map(|c| self.rtps_writer.is_acked_by_all(c))
            .all(|r| r)
    }

    pub fn as_discovered_writer_data(
        &self,
        topic_qos: &TopicQos,
        publisher_qos: &PublisherQos,
    ) -> DiscoveredWriterData {
        let writer_qos = self.rtps_writer.get_qos().clone();
        DiscoveredWriterData::new(
            PublicationBuiltinTopicData::new(
                BuiltInTopicKey {
                    value: self.rtps_writer.guid().into(),
                },
                BuiltInTopicKey {
                    value: GUID_UNKNOWN.into(),
                },
                self.topic_name.clone(),
                self.type_name.to_string(),
                writer_qos.durability.clone(),
                writer_qos.deadline.clone(),
                writer_qos.latency_budget.clone(),
                writer_qos.liveliness.clone(),
                writer_qos.reliability.clone(),
                writer_qos.lifespan.clone(),
                writer_qos.user_data.clone(),
                writer_qos.ownership.clone(),
                writer_qos.destination_order,
                publisher_qos.presentation.clone(),
                publisher_qos.partition.clone(),
                topic_qos.topic_data.clone(),
                publisher_qos.group_data.clone(),
            ),
            WriterProxy::new(
                self.rtps_writer.guid(),
                EntityId::new(EntityKey::new([0; 3]), USER_DEFINED_UNKNOWN),
                self.rtps_writer.unicast_locator_list().to_vec(),
                self.rtps_writer.multicast_locator_list().to_vec(),
                None,
            ),
        )
    }
}

impl DdsDataWriter<RtpsStatelessWriter> {
    pub fn guid(&self) -> Guid {
        self.rtps_writer.guid()
    }

    pub fn _unicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer.unicast_locator_list().to_vec()
    }

    pub fn _multicast_locator_list(&self) -> Vec<Locator> {
        self.rtps_writer.multicast_locator_list().to_vec()
    }

    pub fn _push_mode(&self) -> bool {
        self.rtps_writer.push_mode()
    }

    pub fn _heartbeat_period(&self) -> Duration {
        self.rtps_writer.heartbeat_period()
    }

    pub fn _data_max_size_serialized(&self) -> usize {
        self.rtps_writer.data_max_size_serialized()
    }

    pub fn _new_change(
        &mut self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: ParameterList,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> RtpsWriterCacheChange {
        self.rtps_writer
            .new_change(kind, data, inline_qos, handle, timestamp)
    }

    pub fn _add_change(&mut self, change: RtpsWriterCacheChange) {
        self.rtps_writer.add_change(change);
    }

    pub fn _remove_change<F>(&mut self, f: F)
    where
        F: FnMut(&RtpsWriterCacheChange) -> bool,
    {
        self.rtps_writer.remove_change(f)
    }

    pub fn reader_locator_add(&mut self, a_locator: RtpsReaderLocator) {
        self.rtps_writer.reader_locator_add(a_locator)
    }

    pub fn _reader_locator_remove(&mut self, a_locator: Locator) {
        self.rtps_writer.reader_locator_remove(a_locator)
    }

    pub fn reader_locator_list(&mut self) -> Vec<WriterAssociatedReaderLocator> {
        self.rtps_writer.reader_locator_list()
    }

    pub fn write_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        self.rtps_writer.write_w_timestamp(
            serialized_data,
            instance_serialized_key,
            handle,
            timestamp,
        )
    }
}

#[cfg(test)]
mod test {
    use crate::{
        implementation::rtps::{
            endpoint::RtpsEndpoint,
            messages::RtpsMessageWrite,
            transport::TransportWrite,
            types::{TopicKind, GUID_UNKNOWN},
            writer::RtpsWriter,
        },
        infrastructure::time::DURATION_ZERO,
        topic_definition::type_support::DdsSerializedKey,
    };

    use mockall::mock;

    use super::*;

    mock! {
        Transport{}

        impl TransportWrite for Transport {
            fn write<'a>(&'a mut self, message: &RtpsMessageWrite<'a>, destination_locator_list: &[Locator]);
        }
    }

    #[derive(serde::Serialize)]
    struct MockFoo {}

    impl DdsType for MockFoo {
        fn type_name() -> &'static str {
            todo!()
        }
    }

    #[derive(serde::Serialize)]
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

    fn create_data_writer_test_fixture() -> DdsDataWriter<RtpsStatefulWriter> {
        let rtps_writer = RtpsStatefulWriter::new(RtpsWriter::new(
            RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]),
            true,
            DURATION_ZERO,
            DURATION_ZERO,
            DURATION_ZERO,
            usize::MAX,
            DataWriterQos::default(),
        ));

        let mut data_writer = DdsDataWriter::new(rtps_writer, "", String::from(""));
        data_writer.enabled = true;
        data_writer
    }

    #[test]
    fn get_key_value_known_instance() {
        let mut data_writer = create_data_writer_test_fixture();

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
        let mut data_writer = create_data_writer_test_fixture();
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
