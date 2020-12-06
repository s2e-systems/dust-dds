use crate::behavior::stateful_reader::StatefulReader;
use crate::behavior::types::Duration;
use crate::types::{EntityId, EntityKey, EntityKind, GuidPrefix, ReliabilityKind, GUID};

use rust_dds_api::history_cache::HistoryCache;
use rust_dds_api::protocol::{ProtocolEntity, ProtocolReader};
use rust_dds_api::qos::DataReaderQos;
use rust_dds_api::qos_policy::ReliabilityQosPolicyKind;
use rust_dds_api::types::{InstanceHandle, TopicKind};

pub struct Reader {
    pub stateful_reader: StatefulReader,
}

impl Reader {
    pub fn new(
        guid_prefix: GuidPrefix,
        entity_key: EntityKey,
        topic_kind: TopicKind,
        data_reader_qos: &DataReaderQos,
    ) -> Self {
        let reliability_level = match data_reader_qos.reliability.kind {
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
        };
        let expects_inline_qos = false;
        let heartbeat_response_delay = Duration::from_millis(100);
        let reader_cache = HistoryCache::new(data_reader_qos.resource_limits.clone());
        let entity_kind = match topic_kind {
            TopicKind::NoKey => EntityKind::UserDefinedReaderNoKey,
            TopicKind::WithKey => EntityKind::UserDefinedReaderWithKey,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        let stateful_reader = StatefulReader::new(
            guid,
            topic_kind,
            reliability_level,
            reader_cache,
            expects_inline_qos,
            heartbeat_response_delay,
        );

        Self { stateful_reader }
    }
}

impl ProtocolEntity for Reader {
    fn get_instance_handle(&self) -> InstanceHandle {
        self.stateful_reader.reader.endpoint.entity.guid.into()
    }

    fn enable(&self) {
        todo!()
    }
}

impl ProtocolReader for Reader {}
