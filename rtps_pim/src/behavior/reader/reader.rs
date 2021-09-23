use crate::{
    behavior::types::Duration,
    structure::{
        types::{Guid, Locator, ReliabilityKind, TopicKind},
        RtpsEndpoint,
    },
};

pub struct RtpsReader<L, C> {
    pub endpoint: RtpsEndpoint<L>,
    pub heartbeat_response_delay: Duration,
    pub heartbeat_supression_duration: Duration,
    pub reader_cache: C,
    pub expects_inline_qos: bool,
}

pub trait RtpsReaderOperations {
    fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        heartbeat_response_delay: Duration,
        heartbeat_supression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self;
}
