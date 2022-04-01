use crate::{
    behavior::types::Duration,
    structure::types::{Guid, Locator, ReliabilityKind, TopicKind},
};

pub trait RtpsStatelessWriterAttributes<'a> {
    type ReaderLocatorListType;

    fn reader_locators(&'a mut self) -> Self::ReaderLocatorListType;
}

pub trait RtpsStatelessWriterConstructor {
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

pub trait RtpsStatelessWriterOperations {
    type ReaderLocatorType;

    fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorType);
    fn reader_locator_remove<F>(&mut self, f: F)
    where
        F: FnMut(&Self::ReaderLocatorType) -> bool;
    fn unsent_changes_reset(&mut self);
}
