use crate::{
    behavior::types::Duration,
    structure::types::{Guid, Locator, ReliabilityKind, TopicKind},
};

use super::reader_locator::RtpsReaderLocator;

pub trait StatelessWriterAttributes {
    fn reader_locators(&self);
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
    fn reader_locator_add(&mut self, a_locator: RtpsReaderLocator);

    fn reader_locator_remove(&mut self, a_locator: &Locator);

    fn unsent_changes_reset(&mut self);
}
