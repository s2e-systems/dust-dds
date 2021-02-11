use std::{ops::{Deref, DerefMut}, sync::Arc};

use rust_dds_api::{dcps_psm::StatusMask, infrastructure::{qos::DataWriterQos, qos_policy::ReliabilityQosPolicyKind}};
use rust_rtps::{behavior::{self, StatelessWriter}, types::{EntityId, GUID, GuidPrefix, ReliabilityKind}};

use super::{rtps_datawriter_inner::{AnyDataWriterListener, RtpsDataWriterInner}, rtps_topic_inner::RtpsTopicInner};

pub struct RtpsStatelessDataWriterInner {
    pub stateless_writer: StatelessWriter,
    pub inner: RtpsDataWriterInner,
}

impl Deref for RtpsStatelessDataWriterInner {
    type Target = StatelessWriter;

    fn deref(&self) -> &Self::Target {
        &self.stateless_writer
    }
}

impl DerefMut for RtpsStatelessDataWriterInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stateless_writer
    }
}

impl RtpsStatelessDataWriterInner {
    pub fn new_builtin() -> Self {
        todo!()
    }

    pub fn new(
        guid_prefix: GuidPrefix,
        entity_id: EntityId,
        topic: &Arc<RtpsTopicInner>,
        qos: DataWriterQos,
        listener: Option<Box<dyn AnyDataWriterListener>>,
        status_mask: StatusMask,
    ) -> Self {
        assert!(
            qos.is_consistent().is_ok(),
            "RtpsDataWriter can only be created with consistent QoS"
        );
        let guid = GUID::new(guid_prefix, entity_id);
        let topic_kind = topic.topic_kind();
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let push_mode = true;
        let data_max_sized_serialized = None;
        let heartbeat_period = behavior::types::Duration::from_millis(500);
        let nack_response_delay = behavior::types::constants::DURATION_ZERO;
        let nack_supression_duration = behavior::types::constants::DURATION_ZERO;
        let stateless_writer = StatelessWriter::new(
            guid,
            topic_kind,
            reliability_level,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_supression_duration,
            data_max_sized_serialized,
        );

        let inner = RtpsDataWriterInner::new(topic, qos, listener, status_mask);

        Self {
            stateless_writer,
            inner,
        }
    }
}