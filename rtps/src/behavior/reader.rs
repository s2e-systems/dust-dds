use crate::{
    structure::{Endpoint, HistoryCache},
    types::{ReliabilityKind, TopicKind, GUID},
};

use super::types::Duration;

pub struct Reader {
    pub endpoint: Endpoint,
    pub heartbeat_response_delay: Duration,
    pub heartbeat_supression_duration: Duration,
    pub reader_cache: HistoryCache,
    pub expects_inline_qos: bool,
}

impl Reader {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        expects_inline_qos: bool,
        heartbeat_response_delay: Duration,
        heartbeat_supression_duration: Duration,
    ) -> Self {
        let endpoint = Endpoint::new(guid, topic_kind, reliability_level);
        Self {
            endpoint,
            reader_cache: HistoryCache::new(),
            expects_inline_qos,
            heartbeat_response_delay,
            heartbeat_supression_duration,
        }
    }
}
