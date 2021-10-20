use std::ops::{Deref, DerefMut};

use rust_rtps_pim::{
    behavior::writer::{
        reader_locator::RtpsReaderLocator,
        stateless_writer::{RtpsStatelessWriter, RtpsStatelessWriterOperations},
    },
    structure::types::Locator,
};

use super::{
    rtps_reader_locator_impl::RtpsReaderLocatorImpl,
    rtps_writer_history_cache_impl::WriterHistoryCache,
};

pub struct RtpsStatelessWriterImpl(
    RtpsStatelessWriter<Vec<Locator>, WriterHistoryCache, Vec<RtpsReaderLocatorImpl>>,
);

impl RtpsStatelessWriterImpl {
    pub fn new(
        stateless_writer: RtpsStatelessWriter<
            Vec<Locator>,
            WriterHistoryCache,
            Vec<RtpsReaderLocatorImpl>,
        >,
    ) -> Self {
        Self(stateless_writer)
    }
}

impl Deref for RtpsStatelessWriterImpl {
    type Target = RtpsStatelessWriter<Vec<Locator>, WriterHistoryCache, Vec<RtpsReaderLocatorImpl>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RtpsStatelessWriterImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RtpsStatelessWriterOperations for RtpsStatelessWriterImpl {
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

    #[test]
    fn reader_locator_add() {
        let stateless_writer = RtpsStatelessWriter::new(
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
        let mut rtps_stateless_writer_impl = RtpsStatelessWriterImpl::new(stateless_writer);

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
        let stateless_writer = RtpsStatelessWriter::new(
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
        let mut rtps_stateless_writer_impl = RtpsStatelessWriterImpl::new(stateless_writer);

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
