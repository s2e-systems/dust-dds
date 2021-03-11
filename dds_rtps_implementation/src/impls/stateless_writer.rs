use std::ops::{Deref, DerefMut};

use rust_rtps::{behavior::RTPSStatelessWriter, types::Locator};

use super::{reader_locator::ReaderLocator, writer_impl::WriterImpl};

pub struct StatelessWriter {
    writer: WriterImpl,
    // reader_locators: Vec<ReaderLocator<'a>>,
}

impl Deref for StatelessWriter {
    type Target = WriterImpl;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl DerefMut for StatelessWriter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

// impl RTPSStatelessWriter<WriterImpl> for StatelessWriter {
//     type ReaderLocatorType = ReaderLocator;

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
