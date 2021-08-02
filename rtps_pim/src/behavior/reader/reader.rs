use crate::{
    behavior::types::Duration,
    structure::types::{Locator, ReliabilityKind, TopicKind, GUID},
};

pub trait RtpsReader {
    type HistoryCacheType;

    fn heartbeat_response_delay(&self) -> &Duration;
    fn heartbeat_supression_duration(&self) -> &Duration;
    fn reader_cache(&self) -> &Self::HistoryCacheType;
    fn reader_cache_mut(&mut self) -> &mut Self::HistoryCacheType;
    fn expects_inline_qos(&self) -> bool;
}

pub trait RTPSReaderOperations {
    fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        heartbeat_response_delay: Duration,
        heartbeat_supression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self;
}
