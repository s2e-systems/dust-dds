use std::sync::{
    atomic::{self, AtomicU8},
    Mutex, RwLock,
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
    behavior::writer::stateful_writer::RtpsStatefulWriterOperations,
    structure::{
        types::{EntityId, EntityKind, Guid, ReliabilityKind, TopicKind},
        RtpsEntity,
    },
};

use crate::{
    dds_type::DDSType,
    rtps_impl::{rtps_group_impl::RtpsGroupImpl, rtps_writer_impl::RtpsWriterImpl},
    utils::shared_object::{RtpsShared, RtpsWeak},
};

use super::{data_writer_impl::DataWriterImpl, topic_impl::TopicImpl};

pub struct PublisherImpl {
    qos: PublisherQos,
    rtps_group: RtpsGroupImpl,
    data_writer_impl_list: Mutex<Vec<RtpsShared<DataWriterImpl>>>,
    user_defined_data_writer_counter: AtomicU8,
    default_datawriter_qos: RwLock<DataWriterQos>,
}

impl PublisherImpl {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroupImpl,
        data_writer_impl_list: Vec<RtpsShared<DataWriterImpl>>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_writer_impl_list: Mutex::new(data_writer_impl_list),
            user_defined_data_writer_counter: AtomicU8::new(0),
            default_datawriter_qos: RwLock::new(DataWriterQos::default()),
        }
    }
}

impl<T> DataWriterGAT<'_, '_, T> for PublisherImpl
where
    T: DDSType,
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
        let qos = qos.unwrap_or(self.default_datawriter_qos.read().unwrap().clone());
        let user_defined_data_writer_counter = self
            .user_defined_data_writer_counter
            .fetch_add(1, atomic::Ordering::SeqCst);
        let (entity_kind, topic_kind) = match T::has_key() {
            true => (EntityKind::UserDefinedWriterWithKey, TopicKind::WithKey),
            false => (EntityKind::UserDefinedWriterNoKey, TopicKind::NoKey),
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
        let rtps_writer_impl = RtpsWriterImpl::new(
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
        let data_writer_impl_shared = RtpsShared::new(data_writer_impl);
        let data_writer_impl_weak = data_writer_impl_shared.downgrade();
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

    fn set_default_datawriter_qos(&self, qos: Option<DataWriterQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        *self.default_datawriter_qos.write().unwrap() = qos;
        Ok(())
    }

    fn get_default_datawriter_qos(&self) -> DataWriterQos {
        self.default_datawriter_qos.read().unwrap().clone()
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

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
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
    use rust_dds_api::infrastructure::qos::TopicQos;
    use rust_rtps_pim::structure::types::GUID_UNKNOWN;

    struct MockDDSType;

    impl DDSType for MockDDSType {
        fn type_name() -> &'static str {
            todo!()
        }

        fn has_key() -> bool {
            true
        }
    }

    #[test]
    fn set_default_datawriter_qos_some_value() {
        let rtps_group_impl = RtpsGroupImpl::new(GUID_UNKNOWN);
        let publisher_impl = PublisherImpl::new(PublisherQos::default(), rtps_group_impl, vec![]);

        let mut qos = DataWriterQos::default();
        qos.user_data.value = &[1, 2, 3, 4];
        publisher_impl
            .set_default_datawriter_qos(Some(qos.clone()))
            .unwrap();

        assert_eq!(publisher_impl.get_default_datawriter_qos(), qos);
    }

    #[test]
    fn set_default_datawriter_qos_none() {
        let rtps_group_impl = RtpsGroupImpl::new(GUID_UNKNOWN);
        let publisher_impl = PublisherImpl::new(PublisherQos::default(), rtps_group_impl, vec![]);

        let mut qos = DataWriterQos::default();
        qos.user_data.value = &[1, 2, 3, 4];
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
        let publisher_impl = PublisherImpl::new(PublisherQos::default(), rtps_group_impl, vec![]);
        let a_topic_shared = RtpsShared::new(TopicImpl::new(TopicQos::default()));
        let a_topic_weak = a_topic_shared.downgrade();

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
