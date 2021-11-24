use std::ops::{Deref, DerefMut};

use rust_rtps_pim::{
    behavior::writer::{
        reader_proxy::RtpsReaderProxy,
        stateful_writer::{RtpsStatefulWriter, RtpsStatefulWriterOperations},
    },
    structure::types::{Guid, Locator},
};

use crate::dds_impl::data_writer_impl::{GetRtpsWriter, RtpsWriterType};

use super::{
    rtps_reader_proxy_impl::RtpsReaderProxyImpl, rtps_writer_history_cache_impl::WriterHistoryCache,
};

pub struct RtpsStatefulWriterImpl(
    RtpsStatefulWriter<Vec<Locator>, WriterHistoryCache, Vec<RtpsReaderProxyImpl>>,
);

impl RtpsStatefulWriterImpl {
    pub fn new(
        stateful_writer: RtpsStatefulWriter<
            Vec<Locator>,
            WriterHistoryCache,
            Vec<RtpsReaderProxyImpl>,
        >,
    ) -> Self {
        Self(stateful_writer)
    }
}

impl Deref for RtpsStatefulWriterImpl {
    type Target = RtpsStatefulWriter<Vec<Locator>, WriterHistoryCache, Vec<RtpsReaderProxyImpl>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RtpsStatefulWriterImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RtpsStatefulWriterOperations<Vec<Locator>> for RtpsStatefulWriterImpl {
    fn matched_reader_add(&mut self, a_reader_proxy: RtpsReaderProxy<Vec<Locator>>) {
        let reader_proxy = RtpsReaderProxyImpl::new(a_reader_proxy);
        self.0.matched_readers.push(reader_proxy)
    }

    fn matched_reader_remove(&mut self, reader_proxy_guid: &Guid) {
        self.0
            .matched_readers
            .retain(|x| &x.remote_reader_guid != reader_proxy_guid);
    }

    fn matched_reader_lookup(
        &self,
        a_reader_guid: &Guid,
    ) -> Option<&RtpsReaderProxy<Vec<Locator>>> {
        self.0
            .matched_readers
            .iter()
            .find(|&x| &x.remote_reader_guid == a_reader_guid)
            .map(|x| x.deref())
    }

    fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

impl GetRtpsWriter for RtpsStatefulWriterImpl {
    fn rtps_writer(&self) -> &RtpsWriterType {
        self.0.deref()
    }

    fn rtps_writer_mut(&mut self) -> &mut RtpsWriterType {
        self.0.deref_mut()
    }
}
