use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    time::Instant,
};

use crate::{
    builtin_topics::BuiltInTopicKey,
    implementation::{
        rtps::{
            messages::{
                overall_structure::RtpsMessageHeader,
                submessage_elements::{
                    GuidPrefixSubmessageElement, ProtocolVersionSubmessageElement,
                    VendorIdSubmessageElement,
                },
                submessages::AckNackSubmessage,
                types::ProtocolId,
                RtpsMessage,
            },
            reader_proxy::RtpsReaderProxy,
            transport::TransportWrite,
            types::{EntityId, EntityKind, PROTOCOLVERSION, VENDOR_ID_S2E},
        },
        utils::condvar::DdsCondvar,
    },
    infrastructure::{
        instance::{InstanceHandle, HANDLE_NIL},
        qos::QosKind,
        qos_policy::ReliabilityQosPolicyKind,
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, QosPolicyCount, StatusKind,
        },
    },
    publication::data_writer::DataWriter,
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
use crate::{
    implementation::{
        data_representation_builtin_endpoints::discovered_writer_data::WriterProxy,
        rtps::{stateful_writer::RtpsStatefulWriter, utils::clock::StdTimer},
    },
    publication::data_writer_listener::DataWriterListener,
};

use crate::implementation::{
    data_representation_builtin_endpoints::{
        discovered_reader_data::DiscoveredReaderData, discovered_writer_data::DiscoveredWriterData,
    },
    utils::shared_object::{DdsRwLock, DdsShared, DdsWeak},
};

use super::{
    domain_participant_impl::DomainParticipantImpl, message_receiver::MessageReceiver,
    status_condition_impl::StatusConditionImpl, topic_impl::TopicImpl,
    user_defined_publisher::UserDefinedPublisher,
};

pub trait AnyDataWriterListener {
    fn trigger_on_liveliness_lost(
        &mut self,
        _the_writer: &DdsShared<UserDefinedDataWriter>,
        _status: LivelinessLostStatus,
    );
    fn trigger_on_offered_deadline_missed(
        &mut self,
        _the_writer: &DdsShared<UserDefinedDataWriter>,
        _status: OfferedDeadlineMissedStatus,
    );
    fn trigger_on_offered_incompatible_qos(
        &mut self,
        _the_writer: &DdsShared<UserDefinedDataWriter>,
        _status: OfferedIncompatibleQosStatus,
    );
    fn trigger_on_publication_matched(
        &mut self,
        _the_writer: &DdsShared<UserDefinedDataWriter>,
        _status: PublicationMatchedStatus,
    );
}

impl<Foo> AnyDataWriterListener for Box<dyn DataWriterListener<Foo = Foo> + Send + Sync>
where
    Foo: DdsType + DdsSerialize + 'static,
{
    fn trigger_on_liveliness_lost(
        &mut self,
        the_writer: &DdsShared<UserDefinedDataWriter>,
        status: LivelinessLostStatus,
    ) {
        self.on_liveliness_lost(&DataWriter::new(the_writer.downgrade()), status);
    }

    fn trigger_on_offered_deadline_missed(
        &mut self,
        the_writer: &DdsShared<UserDefinedDataWriter>,
        status: OfferedDeadlineMissedStatus,
    ) {
        self.on_offered_deadline_missed(&DataWriter::new(the_writer.downgrade()), status);
    }

    fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: &DdsShared<UserDefinedDataWriter>,
        status: OfferedIncompatibleQosStatus,
    ) {
        self.on_offered_incompatible_qos(&DataWriter::new(the_writer.downgrade()), status);
    }

    fn trigger_on_publication_matched(
        &mut self,
        the_writer: &DdsShared<UserDefinedDataWriter>,
        status: PublicationMatchedStatus,
    ) {
        self.on_publication_matched(&DataWriter::new(the_writer.downgrade()), status)
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
    user_defined_data_send_condvar: DdsCondvar,
}

impl UserDefinedDataWriter {
    pub fn new(
        rtps_writer: RtpsStatefulWriter<StdTimer>,
        listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        topic: DdsShared<TopicImpl>,
        publisher: DdsWeak<UserDefinedPublisher>,
        user_defined_data_send_condvar: DdsCondvar,
    ) -> DdsShared<Self> {
        let liveliness_lost_status = LivelinessLostStatus {
            total_count: 0,
            total_count_change: 0,
        };

        let publication_matched_status = PublicationMatchedStatus {
            total_count: 0,
            total_count_change: 0,
            last_subscription_handle: HANDLE_NIL,
            current_count: 0,
            current_count_change: 0,
        };

        let offered_deadline_missed_status = OfferedDeadlineMissedStatus {
            total_count: 0,
            total_count_change: 0,
            last_instance_handle: HANDLE_NIL,
        };

        let offered_incompatible_qos_status = OfferedIncompatibleQosStatus {
            total_count: 0,
            total_count_change: 0,
            last_policy_id: 0,
            policies: vec![],
        };

        DdsShared::new(UserDefinedDataWriter {
            rtps_writer: DdsRwLock::new(rtps_writer),
            topic,
            publisher,
            publication_matched_status: DdsRwLock::new(publication_matched_status),
            offered_deadline_missed_status: DdsRwLock::new(offered_deadline_missed_status),
            offered_incompatible_qos_status: DdsRwLock::new(offered_incompatible_qos_status),
            liveliness_lost_status: DdsRwLock::new(liveliness_lost_status),
            matched_subscription_list: DdsRwLock::new(HashMap::new()),
            enabled: DdsRwLock::new(false),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            listener: DdsRwLock::new(listener),
            user_defined_data_send_condvar,
        })
    }
}

impl DdsShared<UserDefinedDataWriter> {
    pub fn on_acknack_submessage_received(
        &self,
        acknack_submessage: &AckNackSubmessage,
        message_receiver: &MessageReceiver,
    ) {
        let mut rtps_writer_lock = self.rtps_writer.write_lock();
        if rtps_writer_lock.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            rtps_writer_lock.on_acknack_submessage_received(
                acknack_submessage,
                message_receiver.source_guid_prefix(),
            );
        }
    }
}

impl DdsShared<UserDefinedDataWriter> {
    pub fn add_matched_reader(&self, discovered_reader_data: &DiscoveredReaderData) {
        let reader_info = &discovered_reader_data.subscription_builtin_topic_data;
        let writer_topic_name = self.topic.get_name().unwrap();
        let writer_type_name = self.topic.get_type_name().unwrap();
        let mut rtps_writer_lock = self.rtps_writer.write_lock();

        if reader_info.topic_name == writer_topic_name && reader_info.type_name == writer_type_name
        {
            let parent_publisher_qos = self.get_publisher().get_qos();
            let qos = rtps_writer_lock.get_qos();
            let mut incompatible_qos_policy_list = Vec::new();
            if qos.durability < reader_info.durability {
                incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
            }
            if parent_publisher_qos.presentation.access_scope
                < reader_info.presentation.access_scope
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

            if incompatible_qos_policy_list.is_empty() {
                let reader_proxy = RtpsReaderProxy::new(
                    discovered_reader_data.reader_proxy.remote_reader_guid,
                    discovered_reader_data.reader_proxy.remote_group_entity_id,
                    discovered_reader_data
                        .reader_proxy
                        .unicast_locator_list
                        .as_ref(),
                    discovered_reader_data
                        .reader_proxy
                        .multicast_locator_list
                        .as_ref(),
                    discovered_reader_data.reader_proxy.expects_inline_qos,
                    true,
                );

                rtps_writer_lock.matched_reader_add(reader_proxy);

                self.matched_subscription_list
                    .write_lock()
                    .insert(reader_info.key.value.as_ref().into(), reader_info.clone());

                // Drop the publication_matched_status_lock such that the listener can be triggered
                // if needed
                {
                    let mut publication_matched_status_lock =
                        self.publication_matched_status.write_lock();
                    publication_matched_status_lock.total_count += 1;
                    publication_matched_status_lock.total_count_change += 1;
                    publication_matched_status_lock.current_count_change += 1;
                }

                self.status_condition
                    .write_lock()
                    .add_communication_state(StatusKind::PublicationMatched);
            } else {
                {
                    let mut offered_incompatible_qos_status_lock =
                        self.offered_incompatible_qos_status.write_lock();
                    offered_incompatible_qos_status_lock.total_count += 1;
                    offered_incompatible_qos_status_lock.total_count_change += 1;
                    offered_incompatible_qos_status_lock.last_policy_id =
                        incompatible_qos_policy_list[0];
                    for incompatible_qos_policy in incompatible_qos_policy_list.into_iter() {
                        if let Some(policy_count) = offered_incompatible_qos_status_lock
                            .policies
                            .iter_mut()
                            .find(|x| x.policy_id == incompatible_qos_policy)
                        {
                            policy_count.count += 1;
                        } else {
                            offered_incompatible_qos_status_lock
                                .policies
                                .push(QosPolicyCount {
                                    policy_id: incompatible_qos_policy,
                                    count: 1,
                                })
                        }
                    }
                }
            }
        }
    }
}

impl DdsShared<UserDefinedDataWriter> {
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

    pub fn get_liveliness_lost_status(&self) -> DdsResult<LivelinessLostStatus> {
        let liveliness_lost_status = self.liveliness_lost_status.read_lock().clone();
        self.liveliness_lost_status.write_lock().total_count_change = 0;
        Ok(liveliness_lost_status)
    }

    pub fn get_offered_deadline_missed_status(&self) -> DdsResult<OfferedDeadlineMissedStatus> {
        let offered_deadline_missed_status =
            self.offered_deadline_missed_status.read_lock().clone();
        self.offered_deadline_missed_status
            .write_lock()
            .total_count_change = 0;
        Ok(offered_deadline_missed_status)
    }

    pub fn get_offered_incompatible_qos_status(&self) -> DdsResult<OfferedIncompatibleQosStatus> {
        let offered_incompatible_qos_status =
            self.offered_incompatible_qos_status.read_lock().clone();
        self.offered_incompatible_qos_status
            .write_lock()
            .total_count_change = 0;
        Ok(offered_incompatible_qos_status)
    }

    pub fn get_publication_matched_status(&self) -> DdsResult<PublicationMatchedStatus> {
        let mut publication_matched_status_lock = self.publication_matched_status.write_lock();

        let mut publication_matched_status = publication_matched_status_lock.clone();
        publication_matched_status.current_count =
            self.matched_subscription_list.read_lock().len() as i32;

        publication_matched_status_lock.current_count_change = 0;
        publication_matched_status_lock.total_count_change = 0;
        Ok(publication_matched_status)
    }

    pub fn get_topic(&self) -> DdsShared<TopicImpl> {
        self.topic.clone()
    }

    pub fn get_publisher(&self) -> DdsShared<UserDefinedPublisher> {
        self.publisher
            .upgrade()
            .expect("Failed to get parent publisher of data writer")
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
}

impl DdsShared<UserDefinedDataWriter> {
    pub fn set_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        let mut rtps_writer_lock = self.rtps_writer.write_lock();

        if *self.enabled.read_lock() {
            rtps_writer_lock.get_qos().check_immutability(&qos)?;
        }

        rtps_writer_lock.set_qos(qos)
    }

    pub fn get_qos(&self) -> DataWriterQos {
        self.rtps_writer.read_lock().get_qos().clone()
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        *self.listener.write_lock() = a_listener;
        Ok(())
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    pub fn enable(&self, parent_participant: &DdsShared<DomainParticipantImpl>) -> DdsResult<()> {
        if !self.publisher.upgrade()?.is_enabled() {
            return Err(DdsError::PreconditionNotMet(
                "Parent publisher disabled".to_string(),
            ));
        }

        parent_participant.announce_datawriter(self.try_into()?);
        *self.enabled.write_lock() = true;

        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_writer.read_lock().guid().into()
    }
}

impl TryFrom<&DdsShared<UserDefinedDataWriter>> for DiscoveredWriterData {
    type Error = DdsError;

    fn try_from(val: &DdsShared<UserDefinedDataWriter>) -> DdsResult<Self> {
        let rtps_writer_lock = val.rtps_writer.read_lock();
        let guid = val.rtps_writer.read_lock().guid();
        let writer_qos = rtps_writer_lock.get_qos();
        let topic_qos = val.topic.get_qos()?;
        let publisher_qos = val.publisher.upgrade()?.get_qos();

        Ok(DiscoveredWriterData {
            writer_proxy: WriterProxy {
                remote_writer_guid: rtps_writer_lock.guid(),
                unicast_locator_list: rtps_writer_lock.unicast_locator_list().to_vec(),
                multicast_locator_list: rtps_writer_lock.multicast_locator_list().to_vec(),
                data_max_size_serialized: None,
                remote_group_entity_id: EntityId::new([0; 3], EntityKind::UserDefinedUnknown),
            },

            publication_builtin_topic_data: PublicationBuiltinTopicData {
                key: BuiltInTopicKey { value: guid.into() },
                participant_key: BuiltInTopicKey { value: [1; 16] },
                topic_name: val.topic.get_name().unwrap(),
                type_name: val.topic.get_type_name().unwrap().to_string(),
                durability: writer_qos.durability.clone(),
                deadline: writer_qos.deadline.clone(),
                latency_budget: writer_qos.latency_budget.clone(),
                liveliness: writer_qos.liveliness.clone(),
                reliability: writer_qos.reliability.clone(),
                lifespan: writer_qos.lifespan.clone(),
                user_data: writer_qos.user_data.clone(),
                ownership: writer_qos.ownership.clone(),
                destination_order: writer_qos.destination_order.clone(),
                presentation: publisher_qos.presentation.clone(),
                partition: publisher_qos.partition.clone(),
                topic_data: topic_qos.topic_data,
                group_data: publisher_qos.group_data,
            },
        })
    }
}

impl DdsShared<UserDefinedDataWriter> {
    pub fn send_message(&self, transport: &mut impl TransportWrite) {
        let mut rtps_writer_lock = self.rtps_writer.write_lock();
        let guid_prefix = rtps_writer_lock.guid().prefix();

        let destined_submessages = rtps_writer_lock.produce_submessages();

        for (reader_proxy, submessages) in destined_submessages {
            let header = RtpsMessageHeader {
                protocol: ProtocolId::PROTOCOL_RTPS,
                version: ProtocolVersionSubmessageElement {
                    value: PROTOCOLVERSION.into(),
                },
                vendor_id: VendorIdSubmessageElement {
                    value: VENDOR_ID_S2E,
                },
                guid_prefix: GuidPrefixSubmessageElement {
                    value: guid_prefix.into(),
                },
            };

            let rtps_message = RtpsMessage {
                header,
                submessages,
            };
            for locator in reader_proxy.unicast_locator_list() {
                transport.write(&rtps_message, *locator);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use crate::{
        implementation::rtps::{
            endpoint::RtpsEndpoint,
            types::{Guid, GuidPrefix, Locator, TopicKind, ENTITYID_UNKNOWN, GUID_UNKNOWN},
            writer::RtpsWriter,
        },
        infrastructure::time::DURATION_ZERO,
        infrastructure::{
            qos::{PublisherQos, TopicQos},
            qos_policy::{
                DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
                GroupDataQosPolicy, LatencyBudgetQosPolicy, LivelinessQosPolicy,
                OwnershipQosPolicy, PartitionQosPolicy, PresentationQosPolicy,
                ReliabilityQosPolicy, ReliabilityQosPolicyKind, TimeBasedFilterQosPolicy,
                TopicDataQosPolicy, UserDataQosPolicy,
            },
        },
        topic_definition::type_support::Endianness,
    };

    use mockall::mock;

    use crate::implementation::{
        data_representation_builtin_endpoints::discovered_reader_data::ReaderProxy,
        rtps::group::RtpsGroupImpl,
    };

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

        fn get_serialized_key<E: Endianness>(&self) -> Vec<u8> {
            self.key.clone()
        }

        fn set_key_fields_from_serialized_key<E: Endianness>(
            &mut self,
            key: &[u8],
        ) -> DdsResult<()> {
            self.key = key.to_vec();
            Ok(())
        }
    }

    impl DdsSerialize for MockKeyedFoo {
        fn serialize<W: Write, E: Endianness>(&self, _writer: W) -> DdsResult<()> {
            Ok(())
        }
    }

    fn create_data_writer_test_fixture() -> DdsShared<UserDefinedDataWriter> {
        let dummy_topic = TopicImpl::new(GUID_UNKNOWN, TopicQos::default(), "", "", DdsWeak::new());

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
            dummy_topic,
            DdsWeak::new(),
            DdsCondvar::new(),
        );
        *data_writer.enabled.write_lock() = true;
        data_writer
    }

    #[test]
    fn write_w_timestamp_stateful_message() {
        let mut stateful_rtps_writer = RtpsStatefulWriter::new(RtpsWriter::new(
            RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]),
            true,
            DURATION_ZERO,
            DURATION_ZERO,
            DURATION_ZERO,
            None,
            DataWriterQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffort,
                    max_blocking_time: DURATION_ZERO,
                },
                ..Default::default()
            },
        ));
        let locator = Locator::new(1, 7400, [1; 16]);
        let expects_inline_qos = false;
        let is_active = true;
        let reader_proxy = RtpsReaderProxy::new(
            GUID_UNKNOWN,
            ENTITYID_UNKNOWN,
            &[locator],
            &[],
            expects_inline_qos,
            is_active,
        );
        stateful_rtps_writer.matched_reader_add(reader_proxy);

        let dummy_topic = TopicImpl::new(GUID_UNKNOWN, TopicQos::default(), "", "", DdsWeak::new());

        let data_writer = UserDefinedDataWriter::new(
            stateful_rtps_writer,
            None,
            dummy_topic,
            DdsWeak::new(),
            DdsCondvar::new(),
        );
        *data_writer.enabled.write_lock() = true;

        data_writer
            .write_w_timestamp(&MockFoo {}, None, Time { sec: 0, nanosec: 0 })
            .unwrap();

        let mut mock_transport = MockTransport::new();
        mock_transport
            .expect_write()
            .withf(move |message, destination_locator| {
                message.submessages.len() == 2 && destination_locator == &locator
            })
            .once()
            .return_const(());
        data_writer.send_message(&mut mock_transport);
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

        data_writer
            .register_instance_w_timestamp(
                &MockKeyedFoo { key: vec![1, 2] },
                Time { sec: 0, nanosec: 0 },
            )
            .unwrap();

        let mut keyed_foo = MockKeyedFoo { key: vec![] };
        assert_eq!(
            data_writer.get_key_value(&mut keyed_foo, [1; 16].as_ref().into()),
            Err(DdsError::BadParameter)
        );
    }

    #[test]
    fn add_compatible_matched_reader() {
        let type_name = "test_type";
        let topic_name = "test_topic".to_string();
        let parent_publisher = UserDefinedPublisher::new(
            PublisherQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsCondvar::new(),
        );
        let test_topic = TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            type_name,
            &topic_name,
            DdsWeak::new(),
        );

        let rtps_writer = RtpsStatefulWriter::new(RtpsWriter::new(
            RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]),
            true,
            DURATION_ZERO,
            DURATION_ZERO,
            DURATION_ZERO,
            None,
            DataWriterQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffort,
                    max_blocking_time: DURATION_ZERO,
                },
                ..Default::default()
            },
        ));

        let data_writer = UserDefinedDataWriter::new(
            rtps_writer,
            None,
            test_topic,
            parent_publisher.downgrade(),
            DdsCondvar::new(),
        );
        *data_writer.enabled.write_lock() = true;
        let subscription_builtin_topic_data = SubscriptionBuiltinTopicData {
            key: BuiltInTopicKey { value: [2; 16] },
            participant_key: BuiltInTopicKey { value: [1; 16] },
            topic_name: topic_name.clone(),
            type_name: type_name.to_string(),
            durability: DurabilityQosPolicy::default(),
            deadline: DeadlineQosPolicy::default(),
            latency_budget: LatencyBudgetQosPolicy::default(),
            liveliness: LivelinessQosPolicy::default(),
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::BestEffort,
                max_blocking_time: Duration::new(0, 0),
            },
            ownership: OwnershipQosPolicy::default(),
            destination_order: DestinationOrderQosPolicy::default(),
            user_data: UserDataQosPolicy::default(),
            time_based_filter: TimeBasedFilterQosPolicy::default(),
            presentation: PresentationQosPolicy::default(),
            partition: PartitionQosPolicy::default(),
            topic_data: TopicDataQosPolicy::default(),
            group_data: GroupDataQosPolicy::default(),
        };
        let discovered_reader_data = DiscoveredReaderData {
            reader_proxy: ReaderProxy {
                remote_reader_guid: Guid::new(
                    GuidPrefix::from([2; 12]),
                    EntityId::new([2; 3], EntityKind::UserDefinedWriterWithKey),
                ),
                remote_group_entity_id: ENTITYID_UNKNOWN,
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                expects_inline_qos: false,
            },
            subscription_builtin_topic_data: subscription_builtin_topic_data.clone(),
        };
        data_writer.add_matched_reader(&discovered_reader_data);

        let publication_matched_status = data_writer.get_publication_matched_status().unwrap();
        assert_eq!(publication_matched_status.current_count, 1);
        assert_eq!(publication_matched_status.current_count_change, 1);
        assert_eq!(publication_matched_status.total_count, 1);
        assert_eq!(publication_matched_status.total_count_change, 1);

        let matched_subscriptions = data_writer.get_matched_subscriptions().unwrap();
        assert_eq!(matched_subscriptions.len(), 1);
        assert_eq!(matched_subscriptions[0], [2; 16].as_ref().into());
        let matched_subscription_data = data_writer
            .get_matched_subscription_data(matched_subscriptions[0])
            .unwrap();
        assert_eq!(matched_subscription_data, subscription_builtin_topic_data);
    }

    #[test]
    fn add_incompatible_matched_reader() {
        let type_name = "test_type";
        let topic_name = "test_topic".to_string();
        let parent_publisher = UserDefinedPublisher::new(
            PublisherQos::default(),
            RtpsGroupImpl::new(GUID_UNKNOWN),
            DdsCondvar::new(),
        );
        let test_topic = TopicImpl::new(
            GUID_UNKNOWN,
            TopicQos::default(),
            type_name,
            &topic_name,
            DdsWeak::new(),
        );

        let rtps_writer = RtpsStatefulWriter::new(RtpsWriter::new(
            RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]),
            true,
            DURATION_ZERO,
            DURATION_ZERO,
            DURATION_ZERO,
            None,
            DataWriterQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffort,
                    max_blocking_time: DURATION_ZERO,
                },
                ..Default::default()
            },
        ));
        let data_writer = UserDefinedDataWriter::new(
            rtps_writer,
            None,
            test_topic,
            parent_publisher.downgrade(),
            DdsCondvar::new(),
        );
        *data_writer.enabled.write_lock() = true;
        let subscription_builtin_topic_data = SubscriptionBuiltinTopicData {
            key: BuiltInTopicKey { value: [2; 16] },
            participant_key: BuiltInTopicKey { value: [1; 16] },
            topic_name: topic_name.clone(),
            type_name: type_name.to_string(),
            durability: DurabilityQosPolicy::default(),
            deadline: DeadlineQosPolicy::default(),
            latency_budget: LatencyBudgetQosPolicy::default(),
            liveliness: LivelinessQosPolicy::default(),
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: Duration::new(0, 0),
            },
            ownership: OwnershipQosPolicy::default(),
            destination_order: DestinationOrderQosPolicy::default(),
            user_data: UserDataQosPolicy::default(),
            time_based_filter: TimeBasedFilterQosPolicy::default(),
            presentation: PresentationQosPolicy::default(),
            partition: PartitionQosPolicy::default(),
            topic_data: TopicDataQosPolicy::default(),
            group_data: GroupDataQosPolicy::default(),
        };
        let discovered_reader_data = DiscoveredReaderData {
            reader_proxy: ReaderProxy {
                remote_reader_guid: Guid::new(
                    GuidPrefix::from([2; 12]),
                    EntityId::new([2; 3], EntityKind::UserDefinedWriterWithKey),
                ),
                remote_group_entity_id: ENTITYID_UNKNOWN,
                unicast_locator_list: vec![],
                multicast_locator_list: vec![],
                expects_inline_qos: false,
            },
            subscription_builtin_topic_data: subscription_builtin_topic_data.clone(),
        };
        data_writer.add_matched_reader(&discovered_reader_data);

        let matched_subscriptions = data_writer.get_matched_subscriptions().unwrap();
        assert_eq!(matched_subscriptions.len(), 0);

        let offered_incompatible_qos_status =
            data_writer.get_offered_incompatible_qos_status().unwrap();
        assert_eq!(offered_incompatible_qos_status.total_count, 1);
        assert_eq!(offered_incompatible_qos_status.total_count_change, 1);
        assert_eq!(
            offered_incompatible_qos_status.last_policy_id,
            RELIABILITY_QOS_POLICY_ID
        );
        assert_eq!(
            offered_incompatible_qos_status.policies,
            vec![QosPolicyCount {
                policy_id: RELIABILITY_QOS_POLICY_ID,
                count: 1,
            }]
        )
    }
}
