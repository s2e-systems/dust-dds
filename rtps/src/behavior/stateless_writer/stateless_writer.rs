use std::{collections::HashMap, ops::{Deref, DerefMut}};

use crate::{behavior::Writer, types::Locator};

use super::ReaderLocator;

pub struct StatelessWriter {
    writer: Writer,
    reader_locators: HashMap<Locator, ReaderLocator>,
}

impl Deref for StatelessWriter {
    type Target = Writer;
    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}
impl DerefMut for StatelessWriter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl StatelessWriter {
    pub fn reader_locator_add(&mut self, a_locator: Locator) {
        self.reader_locators
            .insert(a_locator, ReaderLocator::new(a_locator));
    }

    pub fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.remove(a_locator);
    }

    pub fn unsent_changes_reset(&mut self) {
        for (_, rl) in self.reader_locators.iter_mut() {
            rl.unsent_changes_reset();
        }
    }
}
