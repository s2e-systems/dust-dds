use super::reader_locator::RtpsReaderLocator;
use crate::transport::types::{Guid, Locator};
use alloc::vec::Vec;

pub struct RtpsStatelessWriter {
    guid: Guid,
    reader_locators: Vec<RtpsReaderLocator>,
}

impl RtpsStatelessWriter {
    pub fn new(guid: Guid) -> Self {
        Self {
            guid,
            reader_locators: Vec::new(),
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn reader_locator_add(&mut self, locator: Locator) {
        self.reader_locators
            .push(RtpsReaderLocator::new(locator, false));
    }

    pub fn reader_locator_remove(&mut self, locator: Locator) {
        self.reader_locators.retain(|x| x.locator() != locator);
    }

    pub fn reader_locator_list(&mut self) -> &mut [RtpsReaderLocator] {
        &mut self.reader_locators
    }
}
