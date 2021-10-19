use std::sync::{
    atomic::{self, AtomicU8},
    Mutex,
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
    behavior::writer::{stateless_writer::RtpsStatelessWriter, writer::RtpsWriter},
    messages::{RtpsMessage, RtpsMessageHeader},
    structure::{
        types::{
            EntityId, Guid, GuidPrefix, Locator, ProtocolVersion, ReliabilityKind, TopicKind,
            VendorId, USER_DEFINED_WRITER_NO_KEY, USER_DEFINED_WRITER_WITH_KEY,
        },
        RtpsEndpoint, RtpsGroup, RtpsHistoryCache,
    },
};

use crate::{
    dds_impl::data_writer_impl::RtpsWriterFlavor,
    dds_type::DdsType,
    rtps_impl::rtps_writer_history_cache_impl::WriterHistoryCache,
    utils::{
        message_sender::RtpsSubmessageSender,
        shared_object::{
            rtps_shared_downgrade, rtps_shared_new, rtps_shared_write_lock, RtpsShared, RtpsWeak,
        },
        transport::{RtpsSubmessageWrite, TransportWrite},
    },
};

use super::{data_writer_impl::DataWriterImpl, topic_impl::TopicImpl};

pub struct PublisherImpl {
    _qos: PublisherQos,
    rtps_group: RtpsGroup,
    data_writer_impl_list: Mutex<Vec<RtpsShared<DataWriterImpl>>>,
    user_defined_data_writer_counter: AtomicU8,
    default_datawriter_qos: DataWriterQos,
}

impl PublisherImpl {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroup,
        data_writer_impl_list: Vec<RtpsShared<DataWriterImpl>>,
    ) -> Self {
        Self {
            _qos: qos,
            rtps_group,
            data_writer_impl_list: Mutex::new(data_writer_impl_list),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: DataWriterQos::default(),
        }
    }

    pub fn send_data(
        &self,
        protocol_version: &ProtocolVersion,
        vendor_id: &VendorId,
        guid_prefix: &GuidPrefix,
        transport: &mut (impl TransportWrite + ?Sized),
    ) {
        for writer in self.data_writer_impl_list.lock().unwrap().iter() {
            let mut writer_lock = rtps_shared_write_lock(writer);
            let destined_submessages = writer_lock.create_submessages();
            for (dst_locator, submessages) in destined_submessages {
                let header = RtpsMessageHeader {
                    protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
                    version: *protocol_version,
                    vendor_id: *vendor_id,
                    guid_prefix: *guid_prefix,
                };
                let message = RtpsMessage {
                    header,
                    submessages,
                };
                transport.write(&message, &dst_locator);
            }
        }
    }
}

impl<T> DataWriterGAT<'_, '_, T> for PublisherImpl
where
    T: DdsType,
{
    type TopicType = RtpsWeak<TopicImpl>;
    type DataWriterType = RtpsWeak<DataWriterImpl>;

    fn create_datawriter_gat(
        &'_ self,
        _a_topic: &'_ Self::TopicType,
        qos: Option<DataWriterQos>,
        _a_listener: Option<&'static dyn DataWriterListener<DataPIM = T>>,
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
        let rtps_writer_impl = RtpsWriterFlavor::Stateless(RtpsStatelessWriter {
            writer: RtpsWriter {
                endpoint: RtpsEndpoint::new(
                    guid,
                    topic_kind,
                    reliability_level,
                    unicast_locator_list,
                    multicast_locator_list,
                ),
                push_mode,
                heartbeat_period,
                nack_response_delay,
                nack_suppression_duration,
                last_change_sequence_number: 0,
                data_max_size_serialized,
                writer_cache: WriterHistoryCache::new(),
            },
            reader_locators: vec![],
        });
        let data_writer_impl = DataWriterImpl::new(qos, rtps_writer_impl);
        let data_writer_impl_shared = rtps_shared_new(data_writer_impl);
        let data_writer_impl_weak = rtps_shared_downgrade(&data_writer_impl_shared);
        self.data_writer_impl_list
            .lock()
            .unwrap()
            .push(data_writer_impl_shared);
        Some(data_writer_impl_weak)
    }

    fn delete_datawriter_gat(&self, _a_datawriter: &Self::DataWriterType) -> DDSResult<()> {
        todo!()
    }

    fn lookup_datawriter_gat(
        &'_ self,
        _topic: &'_ Self::TopicType,
    ) -> Option<Self::DataWriterType> {
        todo!()
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

impl RtpsSubmessageSender for PublisherImpl {
    fn create_submessages(&mut self) -> Vec<(Locator, Vec<RtpsSubmessageWrite<'_>>)> {
        let combined_submessages = vec![];
        let data_writer_impl_list_lock = self.data_writer_impl_list.lock().unwrap();
        for data_writer in &*data_writer_impl_list_lock {
            let _submessages = rtps_shared_write_lock(data_writer).create_submessages();
            // combined_submessages = submessages;
        }

        combined_submessages
    }
}

#[cfg(test)]
mod tests {

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
        let a_topic_weak = rtps_shared_downgrade(&a_topic_shared);

        let data_writer_counter_before = publisher_impl
            .user_defined_data_writer_counter
            .load(atomic::Ordering::Relaxed);
        let datawriter =
            publisher_impl.create_datawriter::<MockDDSType>(&a_topic_weak, None, None, 0);
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
}
