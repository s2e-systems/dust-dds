use crate::{
    behavior::types::Duration,
    structure::types::{Guid, Locator, ReliabilityKind, TopicKind},
};

pub trait RtpsStatefulWriterAttributes<'a> {
    type ReaderProxyListType;

    fn matched_readers(&'a mut self) -> Self::ReaderProxyListType;
}

pub trait RtpsStatefulWriterConstructor {
    fn new(
        guid: Guid,
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
}

pub trait RtpsStatefulWriterOperations {
    type ReaderProxyType;

    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType);
    fn matched_reader_remove<F>(&mut self, f: F)
    where
        F: FnMut(&Self::ReaderProxyType) -> bool;
    fn matched_reader_lookup(&self, a_reader_guid: Guid) -> Option<&Self::ReaderProxyType>;
    fn is_acked_by_all(&self) -> bool;
}
