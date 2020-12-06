use crate::rtps::structure::RtpsEndpoint;
use crate::rtps::types::{ReliabilityKind, GUID};
use crate::rtps::structure::HistoryCache;
use crate::types::TopicKind;

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
