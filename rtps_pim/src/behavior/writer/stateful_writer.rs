use crate::structure::types::Guid;

use super::reader_proxy::RtpsReaderProxy;

pub trait RtpsStatefulWriter {
    type ReaderProxyType;

    fn matched_readers(&self) -> &[Self::ReaderProxyType];
}

pub trait RtpsStatefulWriterOperations<L> {
    fn matched_reader_add(&mut self, a_reader_proxy: RtpsReaderProxy<L>);

    fn matched_reader_remove(&mut self, reader_proxy_guid: &Guid);

    fn matched_reader_lookup(&self, a_reader_guid: &Guid) -> Option<&RtpsReaderProxy<L>>;

    fn is_acked_by_all(&self) -> bool;
}
