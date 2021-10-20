use std::ops::{Deref, DerefMut};

use rust_rtps_pim::{
    behavior::reader::{
        stateful_reader::{RtpsStatefulReader, RtpsStatefulReaderOperations},
        writer_proxy::RtpsWriterProxy,
    },
    structure::types::{Guid, Locator},
};

use super::rtps_reader_history_cache_impl::ReaderHistoryCache;

pub struct RtpsStatefulReaderImpl(RtpsStatefulReader<Vec<Locator>, ReaderHistoryCache, ()>);

impl RtpsStatefulReaderImpl {
    pub fn new(stateful_reader: RtpsStatefulReader<Vec<Locator>, ReaderHistoryCache, ()>) -> Self {
        Self(stateful_reader)
    }
}

impl Deref for RtpsStatefulReaderImpl {
    type Target = RtpsStatefulReader<Vec<Locator>, ReaderHistoryCache, ()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RtpsStatefulReaderImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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
