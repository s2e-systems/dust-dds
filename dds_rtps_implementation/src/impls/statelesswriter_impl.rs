use std::ops::{Deref, DerefMut};

use rust_rtps::{
    behavior::{stateless_writer::ReaderLocator, StatelessWriter},
    types::{Locator, SequenceNumber},
};

use super::writer_impl::WriterImpl;

pub struct ReaderLocatorImpl;

impl ReaderLocator for ReaderLocatorImpl {
    type CacheChangeRepresentation = SequenceNumber;

    fn requested_changes(&self) -> &[Self::CacheChangeRepresentation] {
        todo!()
    }

    fn unsent_changes(&self) -> &[Self::CacheChangeRepresentation] {
        todo!()
    }

    fn locator(&self) -> Locator {
        todo!()
    }

    fn expects_inline_qos(&self) -> bool {
        todo!()
    }

    fn new(locator: Locator) -> Self {
        todo!()
    }

    fn next_requested_change(&mut self) -> Option<&Self::CacheChangeRepresentation> {
        todo!()
    }

    fn next_unsent_change(&mut self) -> Option<&Self::CacheChangeRepresentation> {
        todo!()
    }

    fn requested_changes_set(&mut self, req_seq_num_set: &[rust_rtps::types::SequenceNumber]) {
        todo!()
    }
}
pub struct StatelessWriterImpl {
    writer: WriterImpl,
    reader_locators: Vec<ReaderLocatorImpl>,
}

impl Deref for StatelessWriterImpl {
    type Target = WriterImpl;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl DerefMut for StatelessWriterImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl StatelessWriter<WriterImpl> for StatelessWriterImpl {
    type ReaderLocatorType = ReaderLocatorImpl;

    fn reader_locators(&self) -> &[Self::ReaderLocatorType] {
        &self.reader_locators
    }

    fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorType) {
        self.reader_locators.push(a_locator)
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.retain(|x| &x.locator() != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        todo!()
    }
}
