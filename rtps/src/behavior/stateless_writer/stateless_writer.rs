use crate::types::Locator;

pub trait StatelessWriter {
    fn reader_locator_add(&self, a_locator: Locator);

    fn reader_locator_remove(&self, a_locator: &Locator);

    fn unsent_changes_reset(&self);
}