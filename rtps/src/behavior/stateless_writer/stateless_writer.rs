use core::ops::{Deref, DerefMut};

use crate::{behavior::Writer, types::Locator};

use super::RTPSReaderLocator;

pub trait StatelessWriter<T: Writer>: Deref<Target = T> + DerefMut {
    type ReaderLocatorType : RTPSReaderLocator;
    fn reader_locators(&self) -> &[Self::ReaderLocatorType];
    fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorType);
    fn reader_locator_remove(&mut self, a_locator: &Locator);
    fn unsent_changes_reset(&mut self);
}
