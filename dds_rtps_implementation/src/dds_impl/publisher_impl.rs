use std::{
    any::Any,
    sync::{
        atomic::{self, AtomicU8},
        Arc, Mutex, MutexGuard, RwLock,
    },
};

use rust_dds_api::{
    builtin_topics::PublicationBuiltinTopicData,
    dcps_psm::{BuiltInTopicKey, Duration, StatusMask, Time},
    infrastructure::{
        entity::Entity,
        qos::{DataWriterQos, PublisherQos},
        qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
            DurabilityServiceQosPolicy, GroupDataQosPolicy, LatencyBudgetQosPolicy,
            LifespanQosPolicy, LivelinessQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy,
            PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind, TopicDataQosPolicy, UserDataQosPolicy,
        },
    },
    publication::{
        data_writer::DataWriter,
        data_writer_listener::DataWriterListener,
        publisher::{Publisher, PublisherDataWriterFactory},
        publisher_listener::PublisherListener,
    },
    return_type::DDSResult,
    topic::topic_description::TopicDescription,
};
use rust_rtps_pim::{
    behavior::{
        reader::writer_proxy::RtpsWriterProxy,
        writer::stateful_writer::RtpsStatefulWriterConstructor,
    },
    structure::{
        entity::RtpsEntityAttributes,
        types::{
            EntityId, Guid, ReliabilityKind, TopicKind, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
    },
};

use crate::{
    data_representation_builtin_endpoints::sedp_discovered_writer_data::SedpDiscoveredWriterData,
    dds_impl::data_writer_impl::DataWriterImpl,
    dds_type::{DdsSerialize, DdsType},
    rtps_impl::{
        rtps_group_impl::RtpsGroupImpl, rtps_stateful_writer_impl::RtpsStatefulWriterImpl,
    },
    utils::shared_object::{
        rtps_shared_new, rtps_shared_read_lock, rtps_shared_write_lock, RtpsShared,
    },
};

use super::data_writer_impl::RtpsWriter;

pub trait AnyStatelessDataWriter {
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;

    fn into_as_mut_stateless_writer(self: Arc<Self>) -> Arc<RwLock<dyn AsMut<RtpsWriter>>>;
}

impl<T> AnyStatelessDataWriter for RwLock<DataWriterImpl<T>>
where
    T: Send + Sync + 'static,
{
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }

    fn into_as_mut_stateless_writer(self: Arc<Self>) -> Arc<RwLock<dyn AsMut<RtpsWriter>>> {
        self
    }
}

pub trait AnyStatefulDataWriter {
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;

    fn into_as_mut_stateful_writer(self: Arc<Self>) -> Arc<RwLock<dyn AsMut<RtpsWriter>>>;
}

impl<T> AnyStatefulDataWriter for RwLock<DataWriterImpl<T>>
where
    T: Send + Sync + 'static,
{
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }

    fn into_as_mut_stateful_writer(self: Arc<Self>) -> Arc<RwLock<dyn AsMut<RtpsWriter>>> {
        self
    }
}

pub struct PublisherImpl {
    _qos: PublisherQos,
    rtps_group: RtpsGroupImpl,
    stateless_data_writer_impl_list: Mutex<Vec<Arc<dyn AnyStatelessDataWriter + Send + Sync>>>,
    stateful_data_writer_impl_list: Mutex<Vec<Arc<dyn AnyStatefulDataWriter + Send + Sync>>>,
    user_defined_data_writer_counter: AtomicU8,
    default_datawriter_qos: DataWriterQos,
    sedp_builtin_publications_announcer:
        Option<RtpsShared<dyn DataWriter<SedpDiscoveredWriterData> + Send + Sync>>,
}

pub struct StatelessWriterListIterator<'a> {
    stateless_data_writer_list_lock:
        MutexGuard<'a, Vec<Arc<dyn AnyStatelessDataWriter + Send + Sync>>>,
    index: usize,
}

impl<'a> Iterator for StatelessWriterListIterator<'a> {
    type Item = Arc<dyn AnyStatelessDataWriter + Send + Sync>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.stateless_data_writer_list_lock.get(self.index)?;
        self.index += 1;
        Some(item.clone())
    }
}

pub struct StatefulWriterListIterator<'a> {
    stateful_data_writer_list_lock:
        MutexGuard<'a, Vec<Arc<dyn AnyStatefulDataWriter + Send + Sync>>>,
    index: usize,
}

impl<'a> Iterator for StatefulWriterListIterator<'a> {
    type Item = Arc<dyn AnyStatefulDataWriter + Send + Sync>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.stateful_data_writer_list_lock.get(self.index)?;
        self.index += 1;
        Some(item.clone())
    }
}

impl PublisherImpl {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroupImpl,
        stateless_data_writer_impl_list: Vec<Arc<dyn AnyStatelessDataWriter + Send + Sync>>,
        stateful_data_writer_impl_list: Vec<Arc<dyn AnyStatefulDataWriter + Send + Sync>>,
        sedp_builtin_publications_announcer: Option<
            RtpsShared<dyn DataWriter<SedpDiscoveredWriterData> + Send + Sync>,
        >,
    ) -> Self {
        Self {
            _qos: qos,
            rtps_group,
            stateless_data_writer_impl_list: Mutex::new(stateless_data_writer_impl_list),
            stateful_data_writer_impl_list: Mutex::new(stateful_data_writer_impl_list),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            sedp_builtin_publications_announcer,
        }
    }

    pub fn iter_stateless_writer_list(&self) -> StatelessWriterListIterator {
        StatelessWriterListIterator {
            stateless_data_writer_list_lock: self.stateless_data_writer_impl_list.lock().unwrap(),
            index: 0,
        }
    }

    pub fn iter_stateful_writer_list(&self) -> StatefulWriterListIterator {
        StatefulWriterListIterator {
            stateful_data_writer_list_lock: self.stateful_data_writer_impl_list.lock().unwrap(),
            index: 0,
        }
    }
}

impl<Foo> PublisherDataWriterFactory<'_, Foo> for PublisherImpl
where
    Foo: DdsType + DdsSerialize + Send + Sync + 'static,
{
    type TopicType = RtpsShared<dyn TopicDescription<Foo> + Send + Sync>;
    type DataWriterType = RtpsShared<dyn DataWriter<Foo> + Send + Sync>;

    fn datawriter_factory_create_datawriter(
        &'_ self,
        a_topic: &'_ Self::TopicType,
        qos: Option<DataWriterQos>,
        _a_listener: Option<&'static dyn DataWriterListener<DataType = Foo>>,
        _mask: StatusMask,
    ) -> Option<Self::DataWriterType> {
        let qos = qos.unwrap_or(self.default_datawriter_qos.clone());
        let user_defined_data_writer_counter = self
            .user_defined_data_writer_counter
            .fetch_add(1, atomic::Ordering::SeqCst);
        let (entity_kind, topic_kind) = match Foo::has_key() {
            true => (USER_DEFINED_WRITER_WITH_KEY, TopicKind::WithKey),
            false => (USER_DEFINED_WRITER_NO_KEY, TopicKind::NoKey),
        };
        let entity_id = EntityId::new(
            [
                self.rtps_group.guid().entity_id().entity_key()[0],
                user_defined_data_writer_counter,
                0,
            ],
            entity_kind,
        );
        let guid = Guid::new(*self.rtps_group.guid().prefix(), entity_id);
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let unicast_locator_list = &[];
        let multicast_locator_list = &[];
        let push_mode = true;
        let heartbeat_period = rust_rtps_pim::behavior::types::Duration::new(0, 200_000_000);
        let nack_response_delay = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let nack_suppression_duration = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let data_max_size_serialized = None;
        let rtps_writer_impl = RtpsWriter::Stateful(RtpsStatefulWriterImpl::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        ));

        if let Some(sedp_builtin_publications_announcer) = &self.sedp_builtin_publications_announcer
        {
            let topic_shared = rtps_shared_read_lock(a_topic);
            let mut sedp_builtin_publications_announcer_lock =
                rtps_shared_write_lock(sedp_builtin_publications_announcer);
            let sedp_discovered_writer_data = SedpDiscoveredWriterData {
                writer_proxy: RtpsWriterProxy {
                    remote_writer_guid: guid,
                    unicast_locator_list: vec![],
                    multicast_locator_list: vec![],
                    data_max_size_serialized: None,
                    remote_group_entity_id: EntityId::new([0; 3], 0),
                },
                publication_builtin_topic_data: PublicationBuiltinTopicData {
                    key: BuiltInTopicKey { value: guid.into() },
                    participant_key: BuiltInTopicKey { value: [1; 16] },
                    topic_name: topic_shared.get_name().unwrap().to_string(),
                    type_name: Foo::type_name().to_string(),
                    durability: DurabilityQosPolicy::default(),
                    durability_service: DurabilityServiceQosPolicy::default(),
                    deadline: DeadlineQosPolicy::default(),
                    latency_budget: LatencyBudgetQosPolicy::default(),
                    liveliness: LivelinessQosPolicy::default(),
                    reliability: ReliabilityQosPolicy {
                        kind: ReliabilityQosPolicyKind::BestEffortReliabilityQos,
                        max_blocking_time: Duration::new(3, 0),
                    },
                    lifespan: LifespanQosPolicy::default(),
                    user_data: UserDataQosPolicy::default(),
                    ownership: OwnershipQosPolicy::default(),
                    ownership_strength: OwnershipStrengthQosPolicy::default(),
                    destination_order: DestinationOrderQosPolicy::default(),
                    presentation: PresentationQosPolicy::default(),
                    partition: PartitionQosPolicy::default(),
                    topic_data: TopicDataQosPolicy::default(),
                    group_data: GroupDataQosPolicy::default(),
                },
            };
            sedp_builtin_publications_announcer_lock
                .write_w_timestamp(
                    &sedp_discovered_writer_data,
                    None,
                    Time { sec: 0, nanosec: 0 },
                )
                .unwrap();
        }
        let data_writer_impl = DataWriterImpl::new(qos, rtps_writer_impl);
        let data_writer_impl_shared = rtps_shared_new(data_writer_impl);
        self.stateful_data_writer_impl_list
            .lock()
            .unwrap()
            .push(data_writer_impl_shared.clone());
        Some(data_writer_impl_shared)
    }

    fn datawriter_factory_delete_datawriter(
        &self,
        _a_datawriter: &Self::DataWriterType,
    ) -> DDSResult<()> {
        todo!()
    }

    fn datawriter_factory_lookup_datawriter(
        &'_ self,
        _topic: &'_ Self::TopicType,
    ) -> Option<Self::DataWriterType> {
        let data_writer_impl_list_lock = self.stateful_data_writer_impl_list.lock().unwrap();
        let found_data_writer = data_writer_impl_list_lock
            .iter()
            .cloned()
            .find_map(|x| Arc::downcast::<RwLock<DataWriterImpl<Foo>>>(x.into_any()).ok());

        if let Some(found_data_writer) = found_data_writer {
            return Some(found_data_writer);
        };

        let data_writer_impl_list_lock = self.stateless_data_writer_impl_list.lock().unwrap();
        let found_data_writer = data_writer_impl_list_lock
            .iter()
            .cloned()
            .find_map(|x| Arc::downcast::<RwLock<DataWriterImpl<Foo>>>(x.into_any()).ok());

        if let Some(found_data_writer) = found_data_writer {
            return Some(found_data_writer);
        };

        None
    }
}

impl Publisher for PublisherImpl {
    fn suspend_publications(&self) -> DDSResult<()> {
        todo!()
    }

    fn resume_publications(&self) -> DDSResult<()> {
        todo!()
    }

    fn begin_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(
        &self,
        _max_wait: rust_dds_api::dcps_psm::Duration,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_participant(&self) -> &dyn rust_dds_api::domain::domain_participant::DomainParticipant {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datawriter_qos(&mut self, qos: Option<DataWriterQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        self.default_datawriter_qos = qos;
        Ok(())
    }

    fn get_default_datawriter_qos(&self) -> DataWriterQos {
        self.default_datawriter_qos.clone()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &rust_dds_api::infrastructure::qos::TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl Entity for PublisherImpl {
    type Qos = PublisherQos;
    type Listener = &'static dyn PublisherListener;

    fn set_qos(&mut self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(
        &self,
    ) -> DDSResult<rust_dds_api::infrastructure::entity::StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<rust_dds_api::dcps_psm::InstanceHandle> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use rust_dds_api::domain::domain_participant::DomainParticipant;
    use rust_rtps_pim::structure::types::GUID_UNKNOWN;

    struct MockDDSType;

    impl DdsType for MockDDSType {
        fn type_name() -> &'static str {
            todo!()
        }

        fn has_key() -> bool {
            true
        }
    }

    impl DdsSerialize for MockDDSType {
        fn serialize<W: std::io::Write, E: crate::dds_type::Endianness>(
            &self,
            _writer: W,
        ) -> DDSResult<()> {
            todo!()
        }
    }

    mock! {
        Topic<Foo>{}

        impl<Foo> TopicDescription<Foo> for Topic<Foo> {
            fn get_participant(&self) -> &'static dyn DomainParticipant;
            fn get_type_name(&self) -> DDSResult<&'static str>;
            fn get_name(&self) -> DDSResult<String>;
        }
    }

    #[test]
    fn set_default_datawriter_qos_some_value() {
        let rtps_group_impl = RtpsGroupImpl::new(GUID_UNKNOWN);
        let mut publisher_impl = PublisherImpl::new(
            PublisherQos::default(),
            rtps_group_impl,
            vec![],
            vec![],
            None,
        );

        let mut qos = DataWriterQos::default();
        qos.user_data.value = vec![1, 2, 3, 4];
        publisher_impl
            .set_default_datawriter_qos(Some(qos.clone()))
            .unwrap();

        assert_eq!(publisher_impl.get_default_datawriter_qos(), qos);
    }

    #[test]
    fn set_default_datawriter_qos_none() {
        let rtps_group_impl = RtpsGroupImpl::new(GUID_UNKNOWN);
        let mut publisher_impl = PublisherImpl::new(
            PublisherQos::default(),
            rtps_group_impl,
            vec![],
            vec![],
            None,
        );

        let mut qos = DataWriterQos::default();
        qos.user_data.value = vec![1, 2, 3, 4];
        publisher_impl
            .set_default_datawriter_qos(Some(qos.clone()))
            .unwrap();
        publisher_impl.set_default_datawriter_qos(None).unwrap();

        assert_eq!(
            publisher_impl.get_default_datawriter_qos(),
            DataWriterQos::default()
        );
    }

    #[test]
    fn create_datawriter() {
        let rtps_group_impl = RtpsGroupImpl::new(GUID_UNKNOWN);
        let publisher_impl = PublisherImpl::new(
            PublisherQos::default(),
            rtps_group_impl,
            vec![],
            vec![],
            None,
        );
        let a_topic_shared: Arc<RwLock<dyn TopicDescription<MockDDSType> + Send + Sync>> =
            rtps_shared_new(MockTopic::new());

        let data_writer_counter_before = publisher_impl
            .user_defined_data_writer_counter
            .load(atomic::Ordering::Relaxed);
        let datawriter =
            publisher_impl.create_datawriter::<MockDDSType>(&a_topic_shared, None, None, 0);
        let data_writer_counter_after = publisher_impl
            .user_defined_data_writer_counter
            .load(atomic::Ordering::Relaxed);

        assert!(datawriter.is_some());
        assert_eq!(
            publisher_impl
                .stateful_data_writer_impl_list
                .lock()
                .unwrap()
                .len(),
            1
        );
        assert_ne!(data_writer_counter_before, data_writer_counter_after);
    }

    // #[test]
    // fn send_message() {
    //     struct MockStatelessWriterBehavior;

    //     impl RtpsWriterBehavior for MockStatelessWriterBehavior {
    //         fn stateless_writer(
    //     &mut self,
    // ) -> rust_rtps_pim::behavior::writer::stateless_writer::RtpsStatelessWriterRef<
    //     '_,
    //     Vec<Locator>,
    //     crate::rtps_impl::rtps_writer_history_cache_impl::WriterHistoryCache,
    //     std::slice::IterMut<'_, rust_rtps_psm::rtps_reader_locator_impl::RtpsReaderLocatorImpl>,
    //         >{
    //             todo!()
    //         }

    //         fn stateful_writer(
    //             &mut self,
    //         ) -> rust_rtps_pim::behavior::writer::stateful_writer::RtpsStatefulWriterRef<
    //             '_,
    //             Vec<Locator>,
    //             crate::rtps_impl::rtps_writer_history_cache_impl::WriterHistoryCache,
    //             std::slice::IterMut<'_, rust_rtps_psm::rtps_reader_proxy_impl::RtpsReaderProxyImpl>,
    //         > {
    //             todo!()
    //         }
    //     }

    //     impl<'a> StatelessWriterBehavior<'a, Vec<SequenceNumber>, Vec<Parameter<Vec<u8>>>, &'a [u8]>
    //         for MockStatelessWriterBehavior
    //     {
    //         fn send_unsent_data(
    //             &'a mut self,
    //             send_data: &mut dyn FnMut(
    //                 &RtpsReaderLocator,
    //                 DataSubmessage<Vec<Parameter<Vec<u8>>>, &'a [u8]>,
    //             ),
    //             _send_gap: &mut dyn FnMut(
    //                 &RtpsReaderLocator,
    //                 rust_rtps_pim::messages::submessages::GapSubmessage<Vec<SequenceNumber>>,
    //             ),
    //         ) {
    //             let endianness_flag = true;
    //             let inline_qos_flag = true;
    //             let data_flag = true;
    //             let key_flag = false;
    //             let non_standard_payload_flag = false;
    //             let reader_id = EntityIdSubmessageElement {
    //                 value: ENTITYID_UNKNOWN,
    //             };
    //             let writer_id = EntityIdSubmessageElement {
    //                 value: ENTITYID_UNKNOWN,
    //             };
    //             let writer_sn = SequenceNumberSubmessageElement { value: 1 };
    //             let inline_qos: ParameterListSubmessageElement<Vec<Parameter<Vec<u8>>>> =
    //                 ParameterListSubmessageElement { parameter: vec![] };
    //             let serialized_payload = SerializedDataSubmessageElement {
    //                 value: &[1, 2, 3][..],
    //             };

    //             let data_submessage = DataSubmessage {
    //                 endianness_flag,
    //                 inline_qos_flag,
    //                 data_flag,
    //                 key_flag,
    //                 non_standard_payload_flag,
    //                 reader_id,
    //                 writer_id,
    //                 writer_sn,
    //                 inline_qos,
    //                 serialized_payload,
    //             };
    //             let reader_locator = RtpsReaderLocator::new(LOCATOR_INVALID, true);
    //             send_data(&reader_locator, data_submessage)
    //         }
    //     }

    //     struct MockTransport;

    //     impl TransportWrite for MockTransport {
    //         fn write(&mut self, message: &RtpsMessageWrite, _destination_locator: &Locator) {
    //             let endianness_flag = true;
    //             let final_flag = true;
    //             let liveliness_flag = false;
    //             let reader_id = EntityIdSubmessageElement {
    //                 value: ENTITYID_UNKNOWN,
    //             };
    //             let writer_id = EntityIdSubmessageElement {
    //                 value: ENTITYID_UNKNOWN,
    //             };
    //             let first_sn = SequenceNumberSubmessageElement { value: 1 };
    //             let last_sn = SequenceNumberSubmessageElement { value: 2 };
    //             let count = CountSubmessageElement { value: Count(1) };
    //             let _heartbeat_submessage = HeartbeatSubmessageWrite::new(
    //                 endianness_flag,
    //                 final_flag,
    //                 liveliness_flag,
    //                 reader_id,
    //                 writer_id,
    //                 first_sn,
    //                 last_sn,
    //                 count,
    //             );

    //             let endianness_flag = true;
    //             let inline_qos_flag = true;
    //             let data_flag = true;
    //             let key_flag = false;
    //             let non_standard_payload_flag = false;
    //             let reader_id = EntityIdSubmessageElement {
    //                 value: ENTITYID_UNKNOWN,
    //             };
    //             let writer_id = EntityIdSubmessageElement {
    //                 value: ENTITYID_UNKNOWN,
    //             };
    //             let writer_sn = SequenceNumberSubmessageElement { value: 1 };
    //             let inline_qos = ParameterListSubmessageElement { parameter: vec![] };
    //             let serialized_payload = SerializedDataSubmessageElement {
    //                 value: &[1, 2, 3][..],
    //             };

    //             let data_submessage = DataSubmessageWrite::new(
    //                 endianness_flag,
    //                 inline_qos_flag,
    //                 data_flag,
    //                 key_flag,
    //                 non_standard_payload_flag,
    //                 reader_id,
    //                 writer_id,
    //                 writer_sn,
    //                 inline_qos,
    //                 serialized_payload,
    //             );

    //             let expected_submessages = vec![
    //                 // RtpsSubmessageTypeWrite::Heartbeat(heartbeat_submessage),
    //                 RtpsSubmessageTypeWrite::Data(data_submessage),
    //             ];

    //             assert_eq!(message.submessages, expected_submessages)
    //         }
    //     }

    //     let rtps_group_impl = RtpsGroup::new(GUID_UNKNOWN);
    //     let publisher_impl = PublisherImpl::new(
    //         PublisherQos::default(),
    //         rtps_group_impl,
    //         vec![
    //             Arc::new(RwLock::new(MockStatelessWriterBehavior)),
    //             Arc::new(RwLock::new(MockStatelessWriterBehavior)),
    //         ],
    //     );

    //     publisher_impl.send_message(&mut MockTransport)
    // }
}
