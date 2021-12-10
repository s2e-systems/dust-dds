use rust_rtps_pim::{
    behavior::reader::stateless_reader::RtpsStatelessReader,
    messages::submessage_elements::Parameter,
    structure::{history_cache::RtpsHistoryCacheGetChange, types::Locator},
};

use super::rtps_reader_history_cache_impl::{ReaderHistoryCache, ReaderHistoryCacheGetChange};

pub struct RtpsStatelessReaderImpl<T>(pub RtpsStatelessReader<Vec<Locator>, ReaderHistoryCache<T>>);

impl<T> RtpsStatelessReaderImpl<T> {
    pub fn new(stateless_reader: RtpsStatelessReader<Vec<Locator>, ReaderHistoryCache<T>>) -> Self {
        Self(stateless_reader)
    }
}

impl<'a, T> ReaderHistoryCacheGetChange<'a, T> for RtpsStatelessReaderImpl<T> {
    fn get_reader_history_cache_get_change(
        &'a self,
    ) -> &dyn RtpsHistoryCacheGetChange<&'a [Parameter<&'a [u8]>], &'a T> {
        &self.0.reader.reader_cache
    }
}
