use core::ops::Deref;
use std::ops::DerefMut;

use crate::{behavior::RTPSWriter, types::Locator};

use super::RTPSReaderLocator;

pub trait RTPSStatelessWriter<T: RTPSWriter>: Deref<Target = T> + DerefMut {
    type ReaderLocatorType: RTPSReaderLocator;

    fn new(writer: T) -> Self;
    fn reader_locators(&self) -> &[Self::ReaderLocatorType];
    fn reader_locator_add(&mut self, a_locator: Locator);
    fn reader_locator_remove(&mut self, a_locator: &Locator);
    fn unsent_changes_reset(&mut self);
}
