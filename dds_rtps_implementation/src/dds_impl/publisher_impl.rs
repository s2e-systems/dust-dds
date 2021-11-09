use std::{
    any::Any,
    sync::{
        atomic::{self, AtomicU8},
        Arc, Mutex, RwLock, RwLockWriteGuard,
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
        data_writer_listener::DataWriterListener,
        publisher::{DataWriterGAT, Publisher},
        publisher_listener::PublisherListener,
    },
    return_type::DDSResult,
};
use rust_rtps_pim::{
    messages::overall_structure::RtpsMessageHeader,
    structure::{
        group::RtpsGroup,
        types::{
            EntityId, Guid, ReliabilityKind, TopicKind, LOCATOR_INVALID, PROTOCOLVERSION,
            USER_DEFINED_WRITER_NO_KEY, USER_DEFINED_WRITER_WITH_KEY, VENDOR_ID_S2E,
        },
    },
};
use rust_rtps_psm::messages::overall_structure::{RtpsMessageWrite, RtpsSubmessageTypeWrite};

use crate::{
    dds_impl::data_writer_impl::RtpsStatelessWriterType,
    dds_type::DdsType,
    utils::{
        message_receiver::ProcessAckNackSubmessage,
        shared_object::{rtps_shared_new, RtpsShared},
        transport::TransportWrite,
    },
};

use super::data_writer_impl::{DataWriterImpl, RtpsStatefulWriterType};

pub trait ProduceSubmessages {
    fn produce_submessages(&mut self) -> Vec<RtpsSubmessageTypeWrite>;
}

pub trait DataWriterObject {
    fn into_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;

    fn into_process_ack_nack_submessage(
        self: Arc<Self>,
    ) -> Arc<RwLock<dyn ProcessAckNackSubmessage>>;

    fn into_produce_submessages(self: Arc<Self>) -> Arc<RwLock<dyn ProduceSubmessages>>;
}

impl<T> DataWriterObject for RwLock<T>
where
    T: Any + Send + Sync + ProcessAckNackSubmessage + ProduceSubmessages,
{
    fn into_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }

    fn into_process_ack_nack_submessage(
        self: Arc<Self>,
    ) -> Arc<RwLock<dyn ProcessAckNackSubmessage>> {
        self
    }

    fn into_produce_submessages(self: Arc<Self>) -> Arc<RwLock<dyn ProduceSubmessages>> {
        self
    }
}

pub struct PublisherImpl {
    _qos: PublisherQos,
    rtps_group: RtpsGroup,
    data_writer_impl_list: Mutex<Vec<Arc<dyn DataWriterObject + Send + Sync>>>,
    user_defined_data_writer_counter: AtomicU8,
    default_datawriter_qos: DataWriterQos,
}

impl PublisherImpl {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroup,
        data_writer_impl_list: Vec<Arc<dyn DataWriterObject + Send + Sync>>,
    ) -> Self {
        Self {
            _qos: qos,
            rtps_group,
            data_writer_impl_list: Mutex::new(data_writer_impl_list),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
        }
    }

    pub fn send_message(&self, transport: &mut (impl TransportWrite + ?Sized)) {
        let data_writer_list_lock = self.data_writer_impl_list.lock().unwrap();

        let message_producer_list: Vec<Arc<RwLock<dyn ProduceSubmessages>>> = data_writer_list_lock
            .iter()
            .map(|x| x.clone().into_produce_submessages())
            .collect();

        let mut locked_message_producer_list: Vec<RwLockWriteGuard<dyn ProduceSubmessages>> =
            message_producer_list
                .iter()
                .map(|x| x.write().unwrap())
                .collect();

        let mut submessages = Vec::new();
        for locked_message_producer in &mut locked_message_producer_list {
            submessages.append(&mut locked_message_producer.produce_submessages())
        }

        let header = RtpsMessageHeader {
            protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
            version: PROTOCOLVERSION,
            vendor_id: VENDOR_ID_S2E,
            guid_prefix: self.rtps_group.guid.prefix,
        };
        let message = RtpsMessageWrite::new(header, submessages);

        transport.write(&message, &LOCATOR_INVALID);
    }
}

impl<T> DataWriterGAT<'_, '_, T> for PublisherImpl
where
    T: DdsType + Send + 'static,
{
    type TopicType = ();
    type DataWriterType = RtpsShared<DataWriterImpl<T, RtpsStatefulWriterType>>;

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
        let rtps_writer_impl = RtpsStatefulWriterType::new(
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
        );
        let data_writer_impl = DataWriterImpl::new(qos, rtps_writer_impl);
        let data_writer_impl_shared = rtps_shared_new(data_writer_impl);
        self.data_writer_impl_list
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
        let data_reader_list_lock = self.data_writer_impl_list.lock().unwrap();
        data_reader_list_lock
            .iter()
            .find_map(|x| Arc::downcast(x.clone().into_any_arc()).ok())
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
    use rust_rtps_pim::{
        messages::{
            submessage_elements::{
                CountSubmessageElement, EntityIdSubmessageElement, ParameterListSubmessageElement,
                SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
            },
            types::Count,
        },
        structure::types::{Locator, ENTITYID_UNKNOWN, GUID_UNKNOWN},
    };
    use rust_rtps_psm::messages::{
        overall_structure::RtpsMessageWrite,
        submessages::{DataSubmessageWrite, HeartbeatSubmessageWrite},
    };

    struct MockDDSType;

    impl DdsType for MockDDSType {
        fn type_name() -> &'static str {
            todo!()
        }

        fn has_key() -> bool {
            true
        }
    }

    #[test]
    fn set_default_datawriter_qos_some_value() {
        let rtps_group_impl = RtpsGroup::new(GUID_UNKNOWN);
        let mut publisher_impl =
            PublisherImpl::new(PublisherQos::default(), rtps_group_impl, vec![]);

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
            PublisherImpl::new(PublisherQos::default(), rtps_group_impl, vec![]);

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
        let publisher_impl = PublisherImpl::new(PublisherQos::default(), rtps_group_impl, vec![]);
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
            publisher_impl.data_writer_impl_list.lock().unwrap().len(),
            1
        );
        assert_ne!(data_writer_counter_before, data_writer_counter_after);
    }

    #[test]
    fn send_message() {
        struct MockHeartbeatMessageProducer;

        impl ProcessAckNackSubmessage for MockHeartbeatMessageProducer {
            fn process_acknack_submessage(
                &self,
                _source_guid_prefix: rust_rtps_pim::structure::types::GuidPrefix,
                _acknack: &rust_rtps_psm::messages::submessages::AckNackSubmessageRead,
            ) {
                todo!()
            }
        }

        impl ProduceSubmessages for MockHeartbeatMessageProducer {
            fn produce_submessages(&mut self) -> Vec<RtpsSubmessageTypeWrite> {
                let endianness_flag = true;
                let final_flag = true;
                let liveliness_flag = false;
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let first_sn = SequenceNumberSubmessageElement { value: 1 };
                let last_sn = SequenceNumberSubmessageElement { value: 2 };
                let count = CountSubmessageElement { value: Count(1) };

                vec![RtpsSubmessageTypeWrite::Heartbeat(
                    HeartbeatSubmessageWrite::new(
                        endianness_flag,
                        final_flag,
                        liveliness_flag,
                        reader_id,
                        writer_id,
                        first_sn,
                        last_sn,
                        count,
                    ),
                )]
            }
        }

        struct MockDataMessageProducer;

        impl ProcessAckNackSubmessage for MockDataMessageProducer {
            fn process_acknack_submessage(
                &self,
                _source_guid_prefix: rust_rtps_pim::structure::types::GuidPrefix,
                _acknack: &rust_rtps_psm::messages::submessages::AckNackSubmessageRead,
            ) {
                todo!()
            }
        }

        impl ProduceSubmessages for MockDataMessageProducer {
            fn produce_submessages(&mut self) -> Vec<RtpsSubmessageTypeWrite> {
                let endianness_flag = true;
                let inline_qos_flag = true;
                let data_flag = true;
                let key_flag = false;
                let non_standard_payload_flag = false;
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_sn = SequenceNumberSubmessageElement { value: 1 };
                let inline_qos = ParameterListSubmessageElement { parameter: vec![] };
                let serialized_payload = SerializedDataSubmessageElement {
                    value: &[1, 2, 3][..],
                };

                vec![RtpsSubmessageTypeWrite::Data(DataSubmessageWrite::new(
                    endianness_flag,
                    inline_qos_flag,
                    data_flag,
                    key_flag,
                    non_standard_payload_flag,
                    reader_id,
                    writer_id,
                    writer_sn,
                    inline_qos,
                    serialized_payload,
                ))]
            }
        }

        struct MockTransport;

        impl TransportWrite for MockTransport {
            fn write(&mut self, message: &RtpsMessageWrite, _destination_locator: &Locator) {
                let endianness_flag = true;
                let final_flag = true;
                let liveliness_flag = false;
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let first_sn = SequenceNumberSubmessageElement { value: 1 };
                let last_sn = SequenceNumberSubmessageElement { value: 2 };
                let count = CountSubmessageElement { value: Count(1) };
                let heartbeat_submessage = HeartbeatSubmessageWrite::new(
                    endianness_flag,
                    final_flag,
                    liveliness_flag,
                    reader_id,
                    writer_id,
                    first_sn,
                    last_sn,
                    count,
                );

                let endianness_flag = true;
                let inline_qos_flag = true;
                let data_flag = true;
                let key_flag = false;
                let non_standard_payload_flag = false;
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_sn = SequenceNumberSubmessageElement { value: 1 };
                let inline_qos = ParameterListSubmessageElement { parameter: vec![] };
                let serialized_payload = SerializedDataSubmessageElement {
                    value: &[1, 2, 3][..],
                };

                let data_submessage = DataSubmessageWrite::new(
                    endianness_flag,
                    inline_qos_flag,
                    data_flag,
                    key_flag,
                    non_standard_payload_flag,
                    reader_id,
                    writer_id,
                    writer_sn,
                    inline_qos,
                    serialized_payload,
                );

                let expected_submessages = vec![
                    RtpsSubmessageTypeWrite::Heartbeat(heartbeat_submessage),
                    RtpsSubmessageTypeWrite::Data(data_submessage),
                ];

                assert_eq!(message.submessages, expected_submessages)
            }
        }

        let rtps_group_impl = RtpsGroup::new(GUID_UNKNOWN);
        let publisher_impl = PublisherImpl::new(
            PublisherQos::default(),
            rtps_group_impl,
            vec![
                Arc::new(RwLock::new(MockHeartbeatMessageProducer)),
                Arc::new(RwLock::new(MockDataMessageProducer)),
            ],
        );

        publisher_impl.send_message(&mut MockTransport)
    }
}
