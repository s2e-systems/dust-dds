use crate::{
    behavior::types::Duration,
    structure::types::{Guid, Locator, ReliabilityKind, TopicKind},
};

pub trait RtpsStatefulReaderAttributes {
    type WriterProxyType;
    fn matched_writers(&self) -> &[Self::WriterProxyType];
}

pub trait RtpsStatefulReaderConstructor {
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

pub trait RtpsStatefulReaderOperations {
    type WriterProxyType;

    fn matched_writer_add(&mut self, a_writer_proxy: Self::WriterProxyType);
    // Note: Guid instead of Self::WriterProxyType
    fn matched_writer_remove(&mut self, writer_proxy_guid: Guid);
    fn matched_writer_lookup(&self, a_writer_guid: Guid) -> Option<&Self::WriterProxyType>;
}
