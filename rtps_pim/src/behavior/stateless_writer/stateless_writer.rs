use core::ops::{Deref, DerefMut};

use crate::behavior::RTPSWriter;

use super::RTPSReaderLocator;

pub trait RTPSStatelessWriter<T: RTPSWriter>: Deref<Target = T> + DerefMut {
    type ReaderLocatorType: RTPSReaderLocator<Writer = T>;

    fn new(writer: T) -> Self;
    fn reader_locators(&self) -> &[Self::ReaderLocatorType];
    fn reader_locator_add(&mut self, a_locator: T::Locator);
    fn reader_locator_remove(&mut self, a_locator: &T::Locator);
    fn unsent_changes_reset(&mut self);
}
