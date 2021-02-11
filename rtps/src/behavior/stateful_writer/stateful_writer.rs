use crate::types::GUID;

use super::ReaderProxy;

pub trait StatefulWriter {
    fn matched_reader_add(&self, a_reader_proxy: ReaderProxy);

    fn matched_reader_remove(&self, reader_proxy_guid: &GUID);

    fn matched_reader_lookup(&self, a_reader_guid: GUID) -> Option<&ReaderProxy>;

    fn is_acked_by_all(&self) -> bool;
}
