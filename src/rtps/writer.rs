use crate::types::{GUID, EntityKind, GuidPrefix, EntityKey, EntityId, ReliabilityKind};
use crate::behavior::StatefulWriter;
use crate::behavior::types::Duration;

use crate::structure::{CacheChange, HistoryCache};
use rust_dds_api::qos::DataWriterQos;
use rust_dds_api::qos_policy::ReliabilityQosPolicyKind;
use rust_dds_api::types::{ChangeKind, Data, InstanceHandle, ParameterList, TopicKind};

pub struct Writer {
    stateful_writer: StatefulWriter,
}

impl Writer {
    pub fn new(
        guid_prefix: GuidPrefix,
        entity_key: EntityKey,
        topic_kind: TopicKind,
        data_writer_qos: &DataWriterQos,
    ) -> Self {
        let entity_kind = match topic_kind {
            TopicKind::NoKey => EntityKind::UserDefinedWriterNoKey,
            TopicKind::WithKey => EntityKind::UserDefinedWriterWithKey,
        };

        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);

         let reliability_level = match data_writer_qos.reliability.kind {
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
        };

        let writer_cache = HistoryCache::new(data_writer_qos.resource_limits.clone());

        let push_mode = true;
        let data_max_size_serialized = None;
        let heartbeat_period = Duration::from_millis(500);
        let nack_response_delay = Duration::from_millis(0);
        let nack_supression_duration = Duration::from_millis(0);

        let stateful_writer = StatefulWriter::new(
                guid,
                topic_kind,
                reliability_level,
                push_mode,
                writer_cache,
                data_max_size_serialized,
                heartbeat_period,
                nack_response_delay,
                nack_supression_duration,
            );

        Self { stateful_writer }
    }
}

impl ProtocolEntity for Writer {
    fn get_instance_handle(&self) -> InstanceHandle {
        self.stateful_writer.writer.endpoint.entity.guid.into()
    }

    fn enable(&self) {
        todo!()
    }
}

impl ProtocolWriter for Writer {
    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<Data>,
        inline_qos: Option<ParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        self.stateful_writer.writer.new_change(kind, data, inline_qos, handle)
    }

    fn writer_cache(&mut self) -> &mut HistoryCache {
        &mut self.stateful_writer.writer.writer_cache
    }
}
