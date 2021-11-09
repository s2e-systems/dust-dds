use std::{
    any::Any,
    sync::{
        atomic::{self, AtomicU8},
        mpsc::SyncSender,
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
        data_writer_listener::DataWriterListener,
        publisher::{DataWriterGAT, Publisher},
        publisher_listener::PublisherListener,
    },
    return_type::DDSResult,
};
use rust_rtps_pim::structure::{
    group::RtpsGroup,
    types::{
        EntityId, Guid, Locator, ReliabilityKind, TopicKind, USER_DEFINED_WRITER_NO_KEY,
        USER_DEFINED_WRITER_WITH_KEY,
    },
};
use rust_rtps_psm::{
    messages::overall_structure::RtpsSubmessageTypeWrite,
    rtps_stateless_writer_impl::RtpsStatelessWriterImpl,
};

use crate::{
    dds_impl::data_writer_impl::RtpsWriterFlavor,
    dds_type::DdsType,
    utils::{
        message_receiver::ProcessAckNackSubmessage,
        shared_object::{rtps_shared_new, RtpsShared},
        transport::TransportWrite,
    },
};

use super::data_writer_impl::DataWriterImpl;

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
    data_writer_impl_list: Mutex<Vec<Arc<dyn DataWriterObject>>>,
    user_defined_data_writer_counter: AtomicU8,
    default_datawriter_qos: DataWriterQos,
    locator_message_sender: SyncSender<(Locator, Vec<RtpsSubmessageTypeWrite>)>,
    _locator_list_message_sender:
        SyncSender<(Vec<Locator>, Vec<Locator>, Vec<RtpsSubmessageTypeWrite>)>,
}

impl PublisherImpl {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroup,
        data_writer_impl_list: Vec<Arc<dyn DataWriterObject>>,
        locator_message_sender: SyncSender<(Locator, Vec<RtpsSubmessageTypeWrite>)>,
        locator_list_message_sender: SyncSender<(
            Vec<Locator>,
            Vec<Locator>,
            Vec<RtpsSubmessageTypeWrite>,
        )>,
    ) -> Self {
        Self {
            _qos: qos,
            rtps_group,
            data_writer_impl_list: Mutex::new(data_writer_impl_list),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
            locator_message_sender,
            _locator_list_message_sender: locator_list_message_sender,
        }
    }

    pub fn send_message(&self, transport: &mut impl TransportWrite) {
        todo!()
    }
}

impl<T> DataWriterGAT<'_, '_, T> for PublisherImpl
where
    T: DdsType + Send + 'static,
{
    type TopicType = ();
    type DataWriterType = RtpsShared<DataWriterImpl<T>>;

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
        let rtps_writer_impl = RtpsWriterFlavor::new_stateless(
            RtpsStatelessWriterImpl::new(
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
            ),
            self.locator_message_sender.clone(),
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

    use std::sync::mpsc::sync_channel;

    use crate::{dds_impl::topic_impl::TopicImpl, utils::shared_object::rtps_shared_downgrade};

    use super::*;
    use rust_dds_api::infrastructure::qos::TopicQos;
    use rust_rtps_pim::{
        messages::{
            submessage_elements::{
                CountSubmessageElement, EntityIdSubmessageElement, SequenceNumberSubmessageElement,
            },
            types::Count,
        },
        structure::types::{ENTITYID_UNKNOWN, GUID_UNKNOWN},
    };
    use rust_rtps_psm::messages::{
        overall_structure::RtpsMessageWrite, submessages::HeartbeatSubmessageWrite,
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
        let (locator_sender, _) = sync_channel(0);
        let (locator_list_sender, _) = sync_channel(0);
        let rtps_group_impl = RtpsGroup::new(GUID_UNKNOWN);
        let mut publisher_impl = PublisherImpl::new(
            PublisherQos::default(),
            rtps_group_impl,
            vec![],
            locator_sender,
            locator_list_sender,
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
        let (locator_sender, _) = sync_channel(0);
        let (locator_list_sender, _) = sync_channel(0);
        let rtps_group_impl = RtpsGroup::new(GUID_UNKNOWN);
        let mut publisher_impl = PublisherImpl::new(
            PublisherQos::default(),
            rtps_group_impl,
            vec![],
            locator_sender,
            locator_list_sender,
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
        let (locator_sender, _) = sync_channel(0);
        let (locator_list_sender, _) = sync_channel(0);
        let rtps_group_impl = RtpsGroup::new(GUID_UNKNOWN);
        let publisher_impl = PublisherImpl::new(
            PublisherQos::default(),
            rtps_group_impl,
            vec![],
            locator_sender,
            locator_list_sender,
        );
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
        struct MockMessageProducer;

        impl ProcessAckNackSubmessage for MockMessageProducer {
            fn process_acknack_submessage(
                &self,
                _source_guid_prefix: rust_rtps_pim::structure::types::GuidPrefix,
                _acknack: &rust_rtps_psm::messages::submessages::AckNackSubmessageRead,
            ) {
                todo!()
            }
        }

        impl ProduceSubmessages for MockMessageProducer {
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

                let expected_submessages = vec![RtpsSubmessageTypeWrite::Heartbeat(
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
                )];

                assert_eq!(message.submessages, expected_submessages)
            }
        }

        let (locator_sender, _) = sync_channel(0);
        let (locator_list_sender, _) = sync_channel(0);
        let rtps_group_impl = RtpsGroup::new(GUID_UNKNOWN);
        let publisher_impl = PublisherImpl::new(
            PublisherQos::default(),
            rtps_group_impl,
            vec![
                Arc::new(RwLock::new(MockMessageProducer)),
                Arc::new(RwLock::new(MockMessageProducer)),
            ],
            locator_sender,
            locator_list_sender,
        );

        publisher_impl.send_message(&mut MockTransport)
    }
}
