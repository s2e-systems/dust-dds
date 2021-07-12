use crate::{
    behavior::types::Duration,
    structure::types::{Locator, ReliabilityKind, TopicKind, GUID},
};

pub trait RTPSStatefulReader {
    type WriterProxyType;

    fn matched_writers(&self) -> &[Self::WriterProxyType];
}

pub trait RTPSStatefulReaderOperations {
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
    fn matched_writer_add(&mut self, a_writer_proxy: Self::WriterProxyType)
    where
        Self: RTPSStatefulReader;
    fn matched_writer_remove(&mut self, writer_proxy_guid: &GUID);
    fn matched_writer_lookup(&self, a_writer_guid: &GUID) -> Option<&Self::WriterProxyType>
    where
        Self: RTPSStatefulReader;
}
