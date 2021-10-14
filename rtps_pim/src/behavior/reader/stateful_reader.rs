use core::ops::{Deref, DerefMut};

use crate::structure::types::Guid;

use super::{reader::RtpsReader, writer_proxy::RtpsWriterProxy};

pub struct RtpsStatefulReader<L, C, P> {
    pub rtps_reader: RtpsReader<L, C>,
    pub matched_writers: P,
}

impl<L, C, P> Deref for RtpsStatefulReader<L, C, P> {
    type Target = RtpsReader<L, C>;

    fn deref(&self) -> &Self::Target {
        &self.rtps_reader
    }
}

impl<L, C, P> DerefMut for RtpsStatefulReader<L, C, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rtps_reader
    }
}

pub trait RtpsStatefulReaderOperations<L> {
    fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy<L>);
    fn matched_writer_remove(&mut self, writer_proxy_guid: &Guid);
    fn matched_writer_lookup(&self, a_writer_guid: &Guid) -> Option<&RtpsWriterProxy<L>>;
}
