use crate::{
    behavior::types::Duration,
    structure::types::{Locator, ReliabilityKind, TopicKind, GUID},
};

pub trait RTPSStatefulWriter {
    type ReaderProxyType;

    fn matched_readers(&self) -> &[Self::ReaderProxyType];
}

pub trait RTPSStatefulWriterOperations {
    fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
    ) -> Self;

    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType)
    where
        Self: RTPSStatefulWriter;

    fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID);

    fn matched_reader_lookup(&self, a_reader_guid: &GUID) -> Option<&Self::ReaderProxyType>
    where
        Self: RTPSStatefulWriter;

    fn is_acked_by_all(&self) -> bool;
}
