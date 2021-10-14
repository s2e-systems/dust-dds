use rust_rtps_pim::{
    behavior::reader::{
        stateful_reader::{RtpsStatefulReader, RtpsStatefulReaderOperations},
        writer_proxy::RtpsWriterProxy,
    },
    structure::types::{Guid, Locator},
};

use super::rtps_reader_history_cache_impl::ReaderHistoryCache;

pub struct RtpsStatefulReaderImpl(RtpsStatefulReader<Vec<Locator>, ReaderHistoryCache, ()>);

impl<L> RtpsStatefulReaderOperations<L> for RtpsStatefulReaderImpl {
    fn matched_writer_add(&mut self, _a_writer_proxy: RtpsWriterProxy<L>) {
        todo!()
    }

    fn matched_writer_remove(&mut self, _writer_proxy_guid: &Guid) {
        todo!()
    }

    fn matched_writer_lookup(&self, _a_writer_guid: &Guid) -> Option<&RtpsWriterProxy<L>> {
        todo!()
    }
}
