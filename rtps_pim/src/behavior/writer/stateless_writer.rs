use crate::{
    behavior::types::Duration,
    structure::types::{Locator, ReliabilityKind, TopicKind, GUID},
};

use super::writer::RtpsWriter;

pub trait RTPSStatelessWriter {
    type ReaderLocatorType;

    fn reader_locators(&mut self) -> &mut [Self::ReaderLocatorType];

    fn writer_cache_and_reader_locators(
        &mut self,
    ) -> (
        &<Self as RtpsWriter>::HistoryCacheType,
        &mut [Self::ReaderLocatorType],
    )
    where
        Self: RtpsWriter;
}

pub trait RTPSStatelessWriterOperations {
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

    fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorType)
    where
        Self: RTPSStatelessWriter;

    fn reader_locator_remove(&mut self, a_locator: &Locator);

    fn unsent_changes_reset(&mut self);
}
