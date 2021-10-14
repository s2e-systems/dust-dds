use std::ops::{Deref, DerefMut};

use rust_rtps_pim::{
    behavior::reader::stateless_reader::RtpsStatelessReader, structure::types::Locator,
};

use super::rtps_reader_history_cache_impl::ReaderHistoryCache;

pub struct RtpsStatelessReaderImpl(pub RtpsStatelessReader<Vec<Locator>, ReaderHistoryCache>);

impl Deref for RtpsStatelessReaderImpl {
    type Target = RtpsStatelessReader<Vec<Locator>, ReaderHistoryCache>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RtpsStatelessReaderImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
