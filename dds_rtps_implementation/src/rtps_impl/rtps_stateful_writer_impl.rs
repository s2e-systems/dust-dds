use std::ops::{Deref, DerefMut};

use rust_rtps_pim::{
    behavior::writer::{
        reader_proxy::RtpsReaderProxy,
        stateful_writer::{RtpsStatefulWriter, RtpsStatefulWriterOperations},
    },
    structure::types::{Guid, Locator},
};

use super::rtps_writer_history_cache_impl::WriterHistoryCache;

pub struct RtpsStatefulWriterImpl(pub RtpsStatefulWriter<Vec<Locator>, WriterHistoryCache, ()>);

impl Deref for RtpsStatefulWriterImpl {
    type Target = RtpsStatefulWriter<Vec<Locator>, WriterHistoryCache, ()>;

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
    fn matched_reader_add(&mut self, _a_reader_proxy: RtpsReaderProxy<Vec<Locator>>) {
        todo!()
    }

    fn matched_reader_remove(&mut self, _reader_proxy_guid: &Guid) {
        todo!()
    }

    fn matched_reader_lookup(
        &self,
        _a_reader_guid: &Guid,
    ) -> Option<&RtpsReaderProxy<Vec<Locator>>> {
        todo!()
    }

    fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}
