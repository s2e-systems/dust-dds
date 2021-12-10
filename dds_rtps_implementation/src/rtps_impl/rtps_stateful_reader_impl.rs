use rust_rtps_pim::{
    behavior::reader::{
        stateful_reader::{RtpsStatefulReader, RtpsStatefulReaderOperations},
        writer_proxy::RtpsWriterProxy,
    },
    messages::submessage_elements::Parameter,
    structure::{
        history_cache::RtpsHistoryCacheGetChange,
        types::{Guid, Locator},
    },
};

use super::{
    rtps_reader_history_cache_impl::{ReaderHistoryCache, ReaderHistoryCacheGetChange},
    rtps_writer_proxy_impl::RtpsWriterProxyImpl,
};

pub struct RtpsStatefulReaderImpl<T>(
    pub RtpsStatefulReader<Vec<Locator>, ReaderHistoryCache<T>, Vec<RtpsWriterProxyImpl>>,
);

impl<T> RtpsStatefulReaderImpl<T> {
    pub fn new(
        stateful_reader: RtpsStatefulReader<
            Vec<Locator>,
            ReaderHistoryCache<T>,
            Vec<RtpsWriterProxyImpl>,
        >,
    ) -> Self {
        Self(stateful_reader)
    }
}

impl<T> RtpsStatefulReaderOperations<Vec<Locator>> for RtpsStatefulReaderImpl<T> {
    fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy<Vec<Locator>>) {
        let writer_proxy = RtpsWriterProxyImpl::new(a_writer_proxy);
        self.0.matched_writers.push(writer_proxy);
    }

    fn matched_writer_remove(&mut self, writer_proxy_guid: &Guid) {
        self.0
            .matched_writers
            .retain(|x| &x.as_ref().remote_writer_guid != writer_proxy_guid)
    }

    fn matched_writer_lookup(
        &self,
        a_writer_guid: &Guid,
    ) -> Option<&RtpsWriterProxy<Vec<Locator>>> {
        self.0
            .matched_writers
            .iter()
            .find(|&x| &x.as_ref().remote_writer_guid == a_writer_guid)
            .map(|x| x.as_ref())
    }
}

impl<'a, T> ReaderHistoryCacheGetChange<'a, T> for RtpsStatefulReaderImpl<T> {
    fn get_reader_history_cache_get_change(
        &'a self,
    ) -> &dyn RtpsHistoryCacheGetChange<&'a [Parameter<&'a [u8]>], &'a T> {
        &self.0.reader.reader_cache
    }
}
