use std::ops::{Deref, DerefMut};

use rust_rtps::types::Locator;

use super::{reader_locator::ReaderLocator, writer_impl::WriterImpl};

pub struct StatelessWriterImpl {
    writer: WriterImpl,
    // reader_locators: Vec<ReaderLocator<'a>>,
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

// impl rust_rtps::behavior::StatelessWriter<WriterImpl> for StatelessWriterImpl {
//     type ReaderLocatorType = ReaderLocator<'a>;

//     fn reader_locators(&self) -> &[Self::ReaderLocatorType] {
//         &self.reader_locators
//     }

//     fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorType) {
//         self.reader_locators.push(a_locator)
//     }

//     fn reader_locator_remove(&mut self, a_locator: &Locator) {
//         todo!()
//         // self.reader_locators.retain(|x| &x.locator() != a_locator)
//     }

//     fn unsent_changes_reset(&mut self) {
//         todo!()
//     }
// }
