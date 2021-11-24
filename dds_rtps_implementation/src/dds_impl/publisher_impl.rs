use std::{
    any::Any,
    cell::RefCell,
    sync::{
        atomic::{self, AtomicU8},
        Arc, Mutex, RwLock,
    },
};

use rust_dds_api::{
    dcps_psm::StatusMask,
    infrastructure::{
        entity::Entity,
        qos::{DataWriterQos, PublisherQos},
        qos_policy::ReliabilityQosPolicyKind,
    },
    publication::{
        data_writer::DataWriter,
        data_writer_listener::DataWriterListener,
        publisher::{DataWriterGAT, Publisher},
        publisher_listener::PublisherListener,
    },
    return_type::DDSResult,
};
use rust_rtps_pim::{
    behavior::writer::{
        stateful_writer::{RtpsStatefulWriter, StatefulWriterBehavior},
        stateless_writer::StatelessWriterBehavior,
        writer::RtpsWriter,
    },
    messages::overall_structure::RtpsMessageHeader,
    structure::{
        group::RtpsGroup,
        types::{
            EntityId, Guid, ReliabilityKind, TopicKind, PROTOCOLVERSION,
            USER_DEFINED_WRITER_NO_KEY, USER_DEFINED_WRITER_WITH_KEY, VENDOR_ID_S2E,
        },
    },
};
use rust_rtps_psm::messages::{
    overall_structure::{RtpsMessageWrite, RtpsSubmessageTypeWrite},
    submessages::{DataSubmessageWrite, GapSubmessageWrite, HeartbeatSubmessageWrite},
};

use crate::{
    data_representation_builtin_endpoints::spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    dds_impl::data_writer_impl::DataWriterImpl,
    dds_type::{DdsSerialize, DdsType},
    rtps_impl::{
        rtps_stateful_writer_impl::RtpsStatefulWriterImpl,
        rtps_stateless_writer_impl::RtpsStatelessWriterImpl,
    },
    utils::{
        shared_object::{rtps_shared_new, RtpsShared},
        transport::TransportWrite,
    },
};

pub trait AnyStatelessDataWriter {
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;

    fn into_as_mut_stateless_writer(
        self: Arc<Self>,
    ) -> Arc<RwLock<dyn AsMut<RtpsStatelessWriterImpl>>>;
}

impl<T> AnyStatelessDataWriter for RwLock<DataWriterImpl<T, RtpsStatelessWriterImpl>>
where
    T: 'static,
{
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }

    fn into_as_mut_stateless_writer(
        self: Arc<Self>,
    ) -> Arc<RwLock<dyn AsMut<RtpsStatelessWriterImpl>>> {
        self
    }
}

pub trait AnyStatefulDataWriter {
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;

    fn into_as_mut_stateful_writer(
        self: Arc<Self>,
    ) -> Arc<RwLock<dyn AsMut<RtpsStatefulWriterImpl>>>;
}

impl<T> AnyStatefulDataWriter for RwLock<DataWriterImpl<T, RtpsStatefulWriterImpl>>
where
    T: 'static,
{
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }

    fn into_as_mut_stateful_writer(
        self: Arc<Self>,
    ) -> Arc<RwLock<dyn AsMut<RtpsStatefulWriterImpl>>> {
        self
    }
}

pub struct PublisherImpl {
    _qos: PublisherQos,
    rtps_group: RtpsGroup,
    stateless_data_writer_impl_list: Mutex<Vec<Arc<dyn AnyStatelessDataWriter + Send + Sync>>>,
    stateful_data_writer_impl_list: Mutex<Vec<Arc<dyn AnyStatefulDataWriter + Send + Sync>>>,
    user_defined_data_writer_counter: AtomicU8,
    default_datawriter_qos: DataWriterQos,
}

impl PublisherImpl {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroup,
        stateless_data_writer_impl_list: Vec<Arc<dyn AnyStatelessDataWriter + Send + Sync>>,
        stateful_data_writer_impl_list: Vec<Arc<dyn AnyStatefulDataWriter + Send + Sync>>,
    ) -> Self {
        Self {
            _qos: qos,
            rtps_group,
            stateless_data_writer_impl_list: Mutex::new(stateless_data_writer_impl_list),
            stateful_data_writer_impl_list: Mutex::new(stateful_data_writer_impl_list),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
        }
    }

    pub fn send_message(&self, transport: &mut (impl TransportWrite + ?Sized)) {
        let stateless_data_writer_list_lock = self.stateless_data_writer_impl_list.lock().unwrap();

        for stateless_writer in stateless_data_writer_list_lock.iter().cloned() {
            let rtps_stateless_writer_arc_lock = stateless_writer.into_as_mut_stateless_writer();
            let mut rtps_stateless_writer_lock = rtps_stateless_writer_arc_lock.write().unwrap();
            let rtps_stateless_writer = rtps_stateless_writer_lock.as_mut();

            let destined_submessages = RefCell::new(Vec::new());
            rtps_stateless_writer.send_unsent_data(
                &mut |rl, data| {
                    destined_submessages.borrow_mut().push((
                        rl.locator,
                        RtpsSubmessageTypeWrite::Data(DataSubmessageWrite::new(
                            data.endianness_flag,
                            data.inline_qos_flag,
                            data.data_flag,
                            data.key_flag,
                            data.non_standard_payload_flag,
                            data.reader_id,
                            data.writer_id,
                            data.writer_sn,
                            data.inline_qos,
                            data.serialized_payload,
                        )),
                    ));
                },
                &mut |rl, gap| {
                    destined_submessages.borrow_mut().push((
                        rl.locator,
                        RtpsSubmessageTypeWrite::Gap(GapSubmessageWrite::new(
                            gap.endianness_flag,
                            gap.reader_id,
                            gap.writer_id,
                            gap.gap_start,
                            gap.gap_list,
                        )),
                    ));
                },
            );

            for (locator, submessage) in destined_submessages.take() {
                let header = RtpsMessageHeader {
                    protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
                    version: PROTOCOLVERSION,
                    vendor_id: VENDOR_ID_S2E,
                    guid_prefix: self.rtps_group.guid.prefix,
                };
                let message = RtpsMessageWrite::new(header, vec![submessage]);

                transport.write(&message, &locator);
            }
        }

        let stateful_data_writer_list_lock = self.stateful_data_writer_impl_list.lock().unwrap();

        for stateful_writer in stateful_data_writer_list_lock.iter().cloned() {
            let rtps_stateful_writer_arc_lock = stateful_writer.into_as_mut_stateful_writer();
            let mut rtps_stateful_writer_lock = rtps_stateful_writer_arc_lock.write().unwrap();
            let rtps_stateful_writer = rtps_stateful_writer_lock.as_mut();

            let destined_submessages = RefCell::new(Vec::new());

            rtps_stateful_writer.send_heartbeat(&mut |rp, heartbeat| {
                destined_submessages.borrow_mut().push((
                    rp.unicast_locator_list[0],
                    RtpsSubmessageTypeWrite::Heartbeat(HeartbeatSubmessageWrite::new(
                        heartbeat.endianness_flag,
                        heartbeat.final_flag,
                        heartbeat.liveliness_flag,
                        heartbeat.reader_id,
                        heartbeat.writer_id,
                        heartbeat.first_sn,
                        heartbeat.last_sn,
                        heartbeat.count,
                    )),
                ))
            });

            rtps_stateful_writer.send_unsent_data(
                &mut |rp, data| {
                    destined_submessages.borrow_mut().push((
                        rp.unicast_locator_list[0],
                        RtpsSubmessageTypeWrite::Data(DataSubmessageWrite::new(
                            data.endianness_flag,
                            data.inline_qos_flag,
                            data.data_flag,
                            data.key_flag,
                            data.non_standard_payload_flag,
                            data.reader_id,
                            data.writer_id,
                            data.writer_sn,
                            data.inline_qos,
                            data.serialized_payload,
                        )),
                    ));
                },
                &mut |rp, gap| {
                    destined_submessages.borrow_mut().push((
                        rp.unicast_locator_list[0],
                        RtpsSubmessageTypeWrite::Gap(GapSubmessageWrite::new(
                            gap.endianness_flag,
                            gap.reader_id,
                            gap.writer_id,
                            gap.gap_start,
                            gap.gap_list,
                        )),
                    ));
                },
            );

            for (locator, submessage) in destined_submessages.take() {
                let header = RtpsMessageHeader {
                    protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
                    version: PROTOCOLVERSION,
                    vendor_id: VENDOR_ID_S2E,
                    guid_prefix: self.rtps_group.guid.prefix,
                };
                let message = RtpsMessageWrite::new(header, vec![submessage]);

                transport.write(&message, &locator);
            }
        }
    }
}

impl<T> DataWriterGAT<'_, '_, T> for PublisherImpl
where
    T: DdsType + DdsSerialize + Send + 'static,
{
    type TopicType = ();
    type DataWriterType = Arc<RwLock<dyn DataWriter<T> + Send + Sync>>;

    fn create_datawriter_gat(
        &'_ self,
        _a_topic: &'_ Self::TopicType,
        qos: Option<DataWriterQos>,
        _a_listener: Option<&'static dyn DataWriterListener<DataType = T>>,
        _mask: StatusMask,
    ) -> Option<Self::DataWriterType> {
        let qos = qos.unwrap_or(self.default_datawriter_qos.clone());
        let user_defined_data_writer_counter = self
            .user_defined_data_writer_counter
            .fetch_add(1, atomic::Ordering::SeqCst);
        let (entity_kind, topic_kind) = match T::has_key() {
            true => (USER_DEFINED_WRITER_WITH_KEY, TopicKind::WithKey),
            false => (USER_DEFINED_WRITER_NO_KEY, TopicKind::NoKey),
        };
        let entity_id = EntityId::new(
            [
                self.rtps_group.guid.entity_id().entity_key()[0],
                user_defined_data_writer_counter,
                0,
            ],
            entity_kind,
        );
        let guid = Guid::new(*self.rtps_group.guid.prefix(), entity_id);
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let push_mode = true;
        let heartbeat_period = rust_rtps_pim::behavior::types::Duration::new(0, 200_000_000);
        let nack_response_delay = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let nack_suppression_duration = rust_rtps_pim::behavior::types::DURATION_ZERO;
        let data_max_size_serialized = None;
        let rtps_writer_impl = RtpsStatefulWriterImpl::new(RtpsStatefulWriter::new(
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
        let data_writer_impl = DataWriterImpl::new(qos, rtps_writer_impl);
        let data_writer_impl_shared = rtps_shared_new(data_writer_impl);
        self.stateful_data_writer_impl_list
            .lock()
            .unwrap()
            .push(data_writer_impl_shared.clone());
        Some(data_writer_impl_shared)
    }

    fn delete_datawriter_gat(&self, _a_datawriter: &Self::DataWriterType) -> DDSResult<()> {
        todo!()
    }

    fn lookup_datawriter_gat(
        &'_ self,
        _topic: &'_ Self::TopicType,
    ) -> Option<Self::DataWriterType> {
        let data_writer_impl_list_lock = self.stateful_data_writer_impl_list.lock().unwrap();
        let found_data_writer = data_writer_impl_list_lock
            .iter()
            .cloned()
            .find_map(|x| Arc::downcast::<RwLock<DataWriterImpl<T, RtpsStatefulWriterImpl>>>(x.into_any()).ok());

        if let Some(found_data_writer) = found_data_writer{
            return Some(found_data_writer);
        };

        let data_writer_impl_list_lock = self.stateless_data_writer_impl_list.lock().unwrap();
        let found_data_writer = data_writer_impl_list_lock
            .iter()
            .cloned()
            .find_map(|x| Arc::downcast::<RwLock<DataWriterImpl<T, RtpsStatelessWriterImpl>>>(x.into_any()).ok());

        if let Some(found_data_writer) = found_data_writer{
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
    use crate::{dds_impl::topic_impl::TopicImpl, utils::shared_object::rtps_shared_downgrade};

    use super::*;
    use rust_dds_api::infrastructure::qos::TopicQos;
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

    #[test]
    fn set_default_datawriter_qos_some_value() {
        let rtps_group_impl = RtpsGroup::new(GUID_UNKNOWN);
        let mut publisher_impl =
            PublisherImpl::new(PublisherQos::default(), rtps_group_impl, vec![], vec![]);

        let mut qos = DataWriterQos::default();
        qos.user_data.value = vec![1, 2, 3, 4];
        publisher_impl
            .set_default_datawriter_qos(Some(qos.clone()))
            .unwrap();

        assert_eq!(publisher_impl.get_default_datawriter_qos(), qos);
    }

    #[test]
    fn set_default_datawriter_qos_none() {
        let rtps_group_impl = RtpsGroup::new(GUID_UNKNOWN);
        let mut publisher_impl =
            PublisherImpl::new(PublisherQos::default(), rtps_group_impl, vec![], vec![]);

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
        let rtps_group_impl = RtpsGroup::new(GUID_UNKNOWN);
        let publisher_impl =
            PublisherImpl::new(PublisherQos::default(), rtps_group_impl, vec![], vec![]);
        let a_topic_shared = rtps_shared_new(TopicImpl::new(TopicQos::default()));
        let _a_topic_weak = rtps_shared_downgrade(&a_topic_shared);

        let data_writer_counter_before = publisher_impl
            .user_defined_data_writer_counter
            .load(atomic::Ordering::Relaxed);
        let datawriter = publisher_impl.create_datawriter::<MockDDSType>(&(), None, None, 0);
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
