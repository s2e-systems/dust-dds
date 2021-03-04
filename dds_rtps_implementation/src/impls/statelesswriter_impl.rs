use std::ops::{Deref, DerefMut};

use rust_rtps::{
    behavior::{stateless_writer::ReaderLocator, StatelessWriter},
    types::Locator,
};

use super::writer_impl::WriterImpl;

pub struct StatelessWriterImpl {
    writer: WriterImpl,
    reader_locators: Vec<ReaderLocator>,
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
    fn reader_locators(&self) -> &[ReaderLocator] {
        &self.reader_locators
    }

    fn reader_locator_add(&mut self, a_locator: ReaderLocator) {
        self.reader_locators.push(a_locator)
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.retain(|x| &x.locator != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        todo!()
    }
}
