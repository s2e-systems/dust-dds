use std::ops::{Deref, DerefMut};

use rust_rtps_pim::{
    behavior::writer::{
        reader_locator::RtpsReaderLocator,
        stateless_writer::{RtpsStatelessWriter, RtpsStatelessWriterOperations},
    },
    structure::types::Locator,
};

use crate::dds_impl::data_writer_impl::RtpsWriterType;

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
        self.reader_locators.push(reader_locator_impl);
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.retain(|x| &x.locator != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        for reader_locator in &mut self.reader_locators {
            reader_locator.unsent_changes_reset()
        }
    }
}

impl AsRef<RtpsWriterType> for RtpsStatelessWriterImpl {
    fn as_ref(&self) -> &RtpsWriterType {
        self.0.deref()
    }
}

impl AsMut<RtpsWriterType> for RtpsStatelessWriterImpl {
    fn as_mut(&mut self) -> &mut RtpsWriterType {
        self.0.deref_mut()
    }
}
