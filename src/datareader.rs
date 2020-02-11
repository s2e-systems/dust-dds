use crate::cache::{HistoryCache};
use crate::types::{LocatorList};

pub struct StatelessReader {
// From Endpoint:
    // topic_kind: TopicKind,
    // reliability_level: ReliabilityKind_t,
    unicast_locator_list: LocatorList,
    multicast_locator_list: LocatorList,
    // endpoint_id: EntityId,

// From Reader:
    expects_inline_qos: bool,
    // heartbeat_response_delay: Duration,
    reader_cache: HistoryCache,
}

impl StatelessReader
{
    // pub fn new(reader_cache: &HistoryCache) -> Self {
    //     StatelessReader{
    //         reader_cache
    //     }
    // }
}