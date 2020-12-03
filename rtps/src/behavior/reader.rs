use crate::structure::RtpsEndpoint;
use crate::types::{ReliabilityKind, GUID};
use crate::structure::HistoryCache;
use rust_dds_api::types::TopicKind;

pub struct RtpsReader {
    pub endpoint: RtpsEndpoint,
    pub reader_cache: HistoryCache,
    pub expects_inline_qos: bool,
}

impl RtpsReader {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        reader_cache: HistoryCache,
        expects_inline_qos: bool,
    ) -> Self {
        let endpoint = RtpsEndpoint::new(guid, topic_kind, reliability_level);
        Self {
            endpoint,
            reader_cache,
            expects_inline_qos,
        }
    }
}
