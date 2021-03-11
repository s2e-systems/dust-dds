use std::ops::{Deref, DerefMut};

use rust_rtps::{
    behavior::{RTPSStatelessWriter, RTPSWriter},
    types::Locator,
};

use super::reader_locator::ReaderLocator;

pub struct StatelessWriter<W: RTPSWriter> {
    writer: W,
    // reader_locators: Vec<ReaderLocator<'a>>,
}

impl<W: RTPSWriter> Deref for StatelessWriter<W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<W: RTPSWriter> DerefMut for StatelessWriter<W> {
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
