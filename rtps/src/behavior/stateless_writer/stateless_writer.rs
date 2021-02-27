use crate::{behavior::Writer, types::Locator};

use super::ReaderLocator;

pub trait StatelessWriter: Writer {
    fn reader_locators(&self) -> &[ReaderLocator];
    fn reader_locator_add(&mut self, a_locator: ReaderLocator);
    fn reader_locator_remove(&mut self, a_locator: &Locator);
    fn unsent_changes_reset(&mut self);
}
