use std::ops::{Deref, DerefMut};

use rust_rtps_pim::{
    behavior::{
        types::Duration,
        writer::{
            reader_locator::RtpsReaderLocator,
            stateless_writer::{RtpsStatelessWriter, RtpsStatelessWriterOperations},
        },
    },
    structure::{
        history_cache::RtpsHistoryCacheConstructor,
        types::{Guid, Locator, ReliabilityKind, TopicKind},
    },
};

use super::rtps_reader_locator_impl::RtpsReaderLocatorImpl;

pub struct RtpsStatelessWriterImpl<C>(
    RtpsStatelessWriter<Vec<Locator>, C, Vec<RtpsReaderLocatorImpl>>,
);

impl<C> RtpsStatelessWriterImpl<C> {
    pub fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
    ) -> Self
    where
        C: RtpsHistoryCacheConstructor,
    {
        Self(RtpsStatelessWriter::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        ))
    }
}

impl<C> Deref for RtpsStatelessWriterImpl<C> {
    type Target = RtpsStatelessWriter<Vec<Locator>, C, Vec<RtpsReaderLocatorImpl>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> DerefMut for RtpsStatelessWriterImpl<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<C> RtpsStatelessWriterOperations for RtpsStatelessWriterImpl<C> {
    fn reader_locator_add(&mut self, a_locator: RtpsReaderLocator) {
        let reader_locator_impl = RtpsReaderLocatorImpl::new(a_locator);
        self.0.reader_locators.push(reader_locator_impl);
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.0.reader_locators.retain(|x| &x.locator != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        for reader_locator in &mut self.0.reader_locators {
            reader_locator.unsent_changes_reset()
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::{behavior::types::DURATION_ZERO, structure::types::GUID_UNKNOWN};

    use super::*;

    struct MockHistoryCache;

    impl RtpsHistoryCacheConstructor for MockHistoryCache {
        fn new() -> Self {
            Self
        }
    }

    #[test]
    fn reader_locator_add() {
        let mut rtps_stateless_writer_impl: RtpsStatelessWriterImpl<MockHistoryCache> =
            RtpsStatelessWriterImpl::new(
                GUID_UNKNOWN,
                rust_rtps_pim::structure::types::TopicKind::WithKey,
                rust_rtps_pim::structure::types::ReliabilityKind::BestEffort,
                vec![],
                vec![],
                true,
                DURATION_ZERO,
                DURATION_ZERO,
                DURATION_ZERO,
                None,
            );

        let locator1 = Locator::new(1, 1, [1; 16]);
        let locator2 = Locator::new(2, 2, [2; 16]);
        let a_locator1 = RtpsReaderLocator::new(locator1, false);
        let a_locator2 = RtpsReaderLocator::new(locator2, false);
        rtps_stateless_writer_impl.reader_locator_add(a_locator1);
        rtps_stateless_writer_impl.reader_locator_add(a_locator2);

        assert_eq!(rtps_stateless_writer_impl.reader_locators.len(), 2);
    }

    #[test]
    fn reader_locator_remove() {
        let mut rtps_stateless_writer_impl: RtpsStatelessWriterImpl<MockHistoryCache> =
            RtpsStatelessWriterImpl::new(
                GUID_UNKNOWN,
                rust_rtps_pim::structure::types::TopicKind::WithKey,
                rust_rtps_pim::structure::types::ReliabilityKind::BestEffort,
                vec![],
                vec![],
                true,
                DURATION_ZERO,
                DURATION_ZERO,
                DURATION_ZERO,
                None,
            );

        let locator1 = Locator::new(1, 1, [1; 16]);
        let locator2 = Locator::new(2, 2, [2; 16]);
        let a_locator1 = RtpsReaderLocator::new(locator1, false);
        let a_locator2 = RtpsReaderLocator::new(locator2, false);
        rtps_stateless_writer_impl.reader_locator_add(a_locator1);
        rtps_stateless_writer_impl.reader_locator_add(a_locator2);

        rtps_stateless_writer_impl.reader_locator_remove(&locator2);

        assert_eq!(rtps_stateless_writer_impl.reader_locators.len(), 1);
    }
}
