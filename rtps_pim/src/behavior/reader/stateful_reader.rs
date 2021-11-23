use crate::structure::types::Guid;

use super::{reader::RtpsReader, writer_proxy::RtpsWriterProxy};

pub struct RtpsStatefulReader<'a, L, C, W> {
    pub reader: &'a RtpsReader<L, C>,
    pub matched_writers: W,
}

pub trait RtpsStatefulReaderOperations<L> {
    fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy<L>);
    fn matched_writer_remove(&mut self, writer_proxy_guid: &Guid);
    fn matched_writer_lookup(&self, a_writer_guid: &Guid) -> Option<&RtpsWriterProxy<L>>;
}
