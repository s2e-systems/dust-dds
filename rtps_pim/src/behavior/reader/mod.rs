pub mod stateful_reader;
pub mod stateless_reader;
pub mod writer_proxy;

use crate::{
    behavior::types::Duration,
    structure::types::{Guid, Locator, ReliabilityKind, TopicKind},
};

pub trait RtpsReaderAttributes {
    type HistoryCacheType;

    fn heartbeat_response_delay(&self) -> Duration;
    fn heartbeat_suppression_duration(&self) -> Duration;
    fn reader_cache(&mut self) -> &mut Self::HistoryCacheType;
    fn expects_inline_qos(&self) -> bool;
}

pub trait RtpsReaderConstructor {
    fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self;
}