use crate::structure::RtpsEndpoint;
use crate::types::{ReliabilityKind, GUID};

use rust_dds_interface::cache_change::CacheChange;
use rust_dds_interface::history_cache::HistoryCache;
use rust_dds_interface::types::{
    ChangeKind, InstanceHandle, ParameterList, SequenceNumber, TopicKind,
};

pub struct RtpsWriter {
    pub endpoint: RtpsEndpoint,
    pub push_mode: bool,
    pub last_change_sequence_number: SequenceNumber,
    pub writer_cache: HistoryCache,
    pub data_max_sized_serialized: Option<i32>,
}

impl RtpsWriter {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        push_mode: bool,
        writer_cache: HistoryCache,
        data_max_sized_serialized: Option<i32>,
    ) -> Self {
        let endpoint = RtpsEndpoint::new(guid, topic_kind, reliability_level);
        Self {
            endpoint,
            push_mode,
            last_change_sequence_number: 0,
            writer_cache,
            data_max_sized_serialized,
        }
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<ParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        self.last_change_sequence_number += 1;
        CacheChange::new(
            kind,
            self.endpoint.entity.guid.into(),
            handle,
            self.last_change_sequence_number,
            data,
            inline_qos,
        )
    }
}
