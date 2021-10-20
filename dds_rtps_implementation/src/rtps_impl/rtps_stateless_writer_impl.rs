use std::ops::{Deref, DerefMut};

use rust_rtps_pim::{
    behavior::writer::stateless_writer::{RtpsStatelessWriter, RtpsStatelessWriterOperations},
    structure::types::Locator,
};

use super::{
    rtps_reader_locator_impl::RtpsReaderLocatorImpl,
    rtps_writer_history_cache_impl::WriterHistoryCache,
};

pub struct RtpsStatelessWriterImpl(
    pub RtpsStatelessWriter<Vec<Locator>, WriterHistoryCache, Vec<RtpsReaderLocatorImpl>>,
);

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
    fn reader_locator_add(&mut self, _a_locator: Locator) {
        todo!()
    }

    fn reader_locator_remove(&mut self, _a_locator: &Locator) {
        todo!()
    }

    fn unsent_changes_reset(&mut self) {
        todo!()
    }
}
