use rust_dds_api::{
    dcps_psm::StatusMask,
    infrastructure::{
        qos::{DataWriterQos, PublisherQos},
        qos_policy::ReliabilityQosPolicyKind,
    },
    publication::data_writer_listener::DataWriterListener,
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

use super::data_writer_impl::DataWriterImpl;

pub struct PublisherImpl {
    qos: PublisherQos,
    rtps_group: RtpsGroupImpl,
    data_writer_impl_list: Vec<RtpsShared<DataWriterImpl>>,
    user_defined_data_writer_counter: u8,
    default_datawriter_qos: DataWriterQos,
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
            data_writer_impl_list,
            user_defined_data_writer_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
        }
    }

    /// Get a reference to the publisher storage's data writer storage list.
    pub fn data_writer_storage_list(&self) -> &[RtpsShared<DataWriterImpl>] {
        self.data_writer_impl_list.as_slice()
    }

    pub fn create_datawriter<T: DDSType + 'static>(
        &mut self,
        _a_topic: (),
        qos: Option<DataWriterQos>,
        _a_listener: Option<&'static dyn DataWriterListener<DataPIM = T>>,
        _mask: StatusMask,
    ) -> Option<RtpsWeak<DataWriterImpl>> {
        let qos = qos.unwrap_or(self.default_datawriter_qos.clone());
        self.user_defined_data_writer_counter += 1;
        let (entity_kind, topic_kind) = match T::has_key() {
            true => (EntityKind::UserDefinedWriterWithKey, TopicKind::WithKey),
            false => (EntityKind::UserDefinedWriterNoKey, TopicKind::NoKey),
        };
        let entity_id = EntityId::new(
            [
                self.rtps_group.guid().entity_id().entity_key()[0],
                self.user_defined_data_writer_counter,
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
        let rtps_writer = RtpsWriterImpl::new(
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
        let data_writer_storage = DataWriterImpl::new(qos, rtps_writer);
        let data_writer_storage_shared = RtpsShared::new(data_writer_storage);
        let data_writer_storage_weak = data_writer_storage_shared.downgrade();
        self.data_writer_impl_list.push(data_writer_storage_shared);
        Some(data_writer_storage_weak)
    }

    pub fn set_qos(&mut self, qos: Option<PublisherQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        self.qos = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> &PublisherQos {
        &self.qos
    }
}
