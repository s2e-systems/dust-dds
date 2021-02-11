use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use behavior::reader;
use rust_dds_api::{
    dcps_psm::StatusMask,
    dds_type::DDSType,
    infrastructure::{qos::DataWriterQos, qos_policy::ReliabilityQosPolicyKind},
    publication::data_writer_listener::DataWriterListener,
};
use rust_rtps::{
    behavior::{self, stateless_writer::BestEffortReaderLocatorBehavior, StatelessWriter},
    types::{
        constants::{
            ENTITY_KIND_BUILT_IN_WRITER_NO_KEY, ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
            ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY, ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY,
        },
        EntityId, GuidPrefix, ReliabilityKind, TopicKind, GUID,
    },
};

use super::{
    rtps_datawriter_inner::{AnyRtpsDataWriterInner, RtpsDataWriterInner},
    rtps_topic_inner::RtpsTopicInner,
};

pub struct RtpsStatelessDataWriterInner {
    pub stateless_writer: StatelessWriter,
    pub inner: Box<dyn AnyRtpsDataWriterInner>,
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
    pub fn new_builtin<T: DDSType>(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        topic: &Arc<RtpsTopicInner>,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match topic.topic_kind() {
            TopicKind::NoKey => ENTITY_KIND_BUILT_IN_WRITER_NO_KEY,
            TopicKind::WithKey => ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        Self::new(guid_prefix, entity_id, topic, qos, listener, status_mask)
    }

    pub fn new_user_defined<T: DDSType>(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        topic: &Arc<RtpsTopicInner>,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match topic.topic_kind() {
            TopicKind::NoKey => ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY,
            TopicKind::WithKey => ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        Self::new(guid_prefix, entity_id, topic, qos, listener, status_mask)
    }

    fn new<T: DDSType>(
        guid_prefix: GuidPrefix,
        entity_id: EntityId,
        topic: &Arc<RtpsTopicInner>,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
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

        let inner = Box::new(RtpsDataWriterInner::new(topic, qos, listener, status_mask));

        Self {
            stateless_writer,
            inner,
        }
    }

    pub fn produce_messages(&mut self) {
        // let mut output = Vec::new();
        let reader_locators = &mut self.stateless_writer.reader_locators;
        let writer = &self.stateless_writer.writer;
        for (&locator, reader_locator) in reader_locators.iter_mut() {
            BestEffortReaderLocatorBehavior::produce_messages(
                reader_locator,
                &writer.writer_cache,
                writer.endpoint.entity.guid.entity_id(),
                writer.last_change_sequence_number,
            );
            // let messages = reader_locator.produce_messages(
            //     &self.writer.writer_cache,
            //     self.writer.endpoint.entity.guid.entity_id(),
            //     self.writer.last_change_sequence_number,
            // );
            // if !messages.is_empty() {
            //     output.push(DestinedMessages::SingleDestination { locator, messages });
            // }
        }
        // output
    }
}
