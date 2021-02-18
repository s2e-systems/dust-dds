use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use rust_dds_api::{
    dcps_psm::StatusMask,
    dds_type::DDSType,
    infrastructure::{qos::DataReaderQos, qos_policy::ReliabilityQosPolicyKind},
    subscription::data_reader_listener::DataReaderListener,
};
use rust_rtps::{
    behavior::{self, types::Duration, StatelessReader},
    types::{
        constants::ENTITY_KIND_BUILT_IN_READER_NO_KEY, EntityId, GuidPrefix, Locator,
        ReliabilityKind, TopicKind, GUID,
    },
};

use super::{
    rtps_datareader_inner::{AnyRtpsDataReaderInner, RtpsDataReaderInner},
    rtps_topic_impl::RtpsTopicInner,
};

pub struct RtpsStatelessDataReaderInner {
    pub stateless_reader: StatelessReader,
    pub inner: Box<dyn AnyRtpsDataReaderInner>,
}

impl Deref for RtpsStatelessDataReaderInner {
    type Target = StatelessReader;

    fn deref(&self) -> &Self::Target {
        &self.stateless_reader
    }
}

impl DerefMut for RtpsStatelessDataReaderInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stateless_reader
    }
}

impl RtpsStatelessDataReaderInner {
    pub fn new_builtin<T: DDSType>(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        topic: &Arc<RtpsTopicInner>,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match topic.topic_kind() {
            TopicKind::NoKey => ENTITY_KIND_BUILT_IN_READER_NO_KEY,
            TopicKind::WithKey => ENTITY_KIND_BUILT_IN_READER_NO_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        Self::new(
            guid_prefix,
            entity_id,
            unicast_locator_list,
            multicast_locator_list,
            topic,
            qos,
            listener,
            status_mask,
        )
    }

    pub fn new_user_defined<T: DDSType>(
        guid_prefix: GuidPrefix,
        entity_key: [u8; 3],
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        topic: &Arc<RtpsTopicInner>,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match topic.topic_kind() {
            TopicKind::NoKey => ENTITY_KIND_BUILT_IN_READER_NO_KEY,
            TopicKind::WithKey => ENTITY_KIND_BUILT_IN_READER_NO_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        Self::new(
            guid_prefix,
            entity_id,
            unicast_locator_list,
            multicast_locator_list,
            topic,
            qos,
            listener,
            status_mask,
        )
    }

    fn new<T: DDSType>(
        guid_prefix: GuidPrefix,
        entity_id: EntityId,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        topic: &Arc<RtpsTopicInner>,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        assert!(
            qos.is_consistent().is_ok(),
            "RtpsDataReader can only be created with consistent QoS"
        );
        let guid = GUID::new(guid_prefix, entity_id);
        let topic_kind = topic.topic_kind();
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let expects_inline_qos = false;
        let heartbeat_response_delay = Duration::from_millis(500);
        let heartbeat_supression_duration = behavior::types::constants::DURATION_ZERO;
        let stateless_reader = StatelessReader::new(
            guid,
            unicast_locator_list,
            multicast_locator_list,
            topic_kind,
            reliability_level,
            expects_inline_qos,
            heartbeat_response_delay,
            heartbeat_supression_duration,
        );

        let inner = Box::new(RtpsDataReaderInner::new(topic, qos, listener, status_mask));

        Self {
            stateless_reader,
            inner,
        }
    }
}
