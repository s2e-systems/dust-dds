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