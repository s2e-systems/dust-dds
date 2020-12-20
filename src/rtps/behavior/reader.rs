use crate::rtps::structure::Endpoint;
use crate::rtps::types::{ReliabilityKind, GUID};
use crate::rtps::structure::HistoryCache;
use crate::types::TopicKind;

pub struct Reader {
    pub endpoint: Endpoint,
    pub reader_cache: HistoryCache,
    pub expects_inline_qos: bool,
}

impl Reader {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        expects_inline_qos: bool,
    ) -> Self {
        let endpoint = Endpoint::new(guid, topic_kind, reliability_level);
        Self {
            endpoint,
            reader_cache: HistoryCache::new(),
            expects_inline_qos,
        }
    }
}
