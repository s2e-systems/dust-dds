use crate::{structure::{Endpoint, HistoryCache}, types::{GUID, Locator, ReliabilityKind, TopicKind}};

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
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        expects_inline_qos: bool,
        heartbeat_response_delay: Duration,
        heartbeat_supression_duration: Duration,
    ) -> Self {
        let endpoint = Endpoint::new(guid, unicast_locator_list, multicast_locator_list, topic_kind, reliability_level);
        Self {
            endpoint,
            reader_cache: HistoryCache::new(),
            expects_inline_qos,
            heartbeat_response_delay,
            heartbeat_supression_duration,
        }
    }
}
