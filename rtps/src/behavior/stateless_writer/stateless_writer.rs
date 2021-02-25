use crate::types::Locator;

use super::ReaderLocator;

pub struct StatelessWriter {
    pub reader_locators: Vec<ReaderLocator>,
}


impl StatelessWriter {
    pub fn new() -> Self {
        Self {
            reader_locators: Vec::new(),
        }
    }
    pub fn reader_locator_add(&mut self, a_locator: ReaderLocator) {
        self.reader_locators.push(a_locator);
    }

    pub fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.retain(|rl| &rl.locator != a_locator);
    }

    pub fn unsent_changes_reset(&mut self) {
        for rl in self.reader_locators.iter_mut() {
            rl.unsent_changes_reset();
        }
    }
}
