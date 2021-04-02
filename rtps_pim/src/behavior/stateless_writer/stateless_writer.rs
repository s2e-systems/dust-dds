use core::{
    iter::FromIterator,
    ops::{Index, IndexMut},
};

use super::RTPSReaderLocator;
use crate::{
    behavior::{self, RTPSWriter},
    structure::{self, RTPSHistoryCache},
};

pub struct RTPSStatelessWriter<PSM: structure::Types + behavior::Types, HistoryCache: RTPSHistoryCache<PSM = PSM>, ReaderLocatorList> {
    writer: RTPSWriter<PSM, HistoryCache>,
    reader_locators: ReaderLocatorList,
}

impl<
        PSM: structure::Types + behavior::Types,
        HistoryCache: RTPSHistoryCache<PSM = PSM>,
        ReaderLocatorList: IntoIterator<Item = RTPSReaderLocator<PSM>>
            + Extend<RTPSReaderLocator<PSM>>
            + FromIterator<RTPSReaderLocator<PSM>>
            + Clone,
    > RTPSStatelessWriter<PSM, HistoryCache, ReaderLocatorList>
    where PSM::Locator: PartialEq
{
    pub fn reader_locator_add(&mut self, a_locator: <PSM as structure::Types>::Locator) {
        self.reader_locators
            .extend(Some(RTPSReaderLocator::new(a_locator, false)));
    }

    pub fn reader_locator_remove(&mut self, a_locator: <PSM as structure::Types>::Locator) {
        let reader_locator = &RTPSReaderLocator::new(a_locator, false);
        self.reader_locators = self
            .reader_locators
            .clone()
            .into_iter()
            .filter(|x| x != reader_locator)
            .collect();
    }

    pub fn unsent_changes_reset(&mut self) {}

    pub fn behavior(&mut self, _a_locator: <PSM as structure::Types>::Locator) {
        // let reader_locator = &mut self.reader_locators[0];
        //if let Some(_change) = self.reader_locators[0].next_unsent_change(&self.writer) {}
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn reader_locator_remove() {
        let v = vec![1];
        
    }
}