use rust_rtps_pim::{
    behavior::reader::stateless_reader::RtpsStatelessReader, structure::types::Locator,
};

use super::rtps_reader_history_cache_impl::ReaderHistoryCache;

pub struct RtpsStatelessReaderImpl<T>(pub RtpsStatelessReader<Vec<Locator>, ReaderHistoryCache<T>>);

impl<T> RtpsStatelessReaderImpl<T> {
    pub fn new(stateless_reader: RtpsStatelessReader<Vec<Locator>, ReaderHistoryCache<T>>) -> Self {
        Self(stateless_reader)
    }
}
