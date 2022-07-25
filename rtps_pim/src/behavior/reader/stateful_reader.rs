use crate::{
    behavior::types::Duration,
    structure::types::{Guid, Locator, ReliabilityKind, TopicKind},
};

pub trait RtpsStatefulReaderAttributes<'a> {
    type WriterProxyListType;

    fn matched_writers(&'a mut self) -> Self::WriterProxyListType;
}

pub trait RtpsStatefulReaderConstructor {
    #[allow(clippy::too_many_arguments)]
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
    fn matched_writer_remove<F>(&mut self, f: F)
    where
        F: FnMut(&Self::WriterProxyType) -> bool;
    fn matched_writer_lookup(&mut self, a_writer_guid: Guid) -> Option<&mut Self::WriterProxyType>;
}
