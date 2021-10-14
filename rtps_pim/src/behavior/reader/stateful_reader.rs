use crate::{
    structure::types::{Guid},
};

use super::writer_proxy::RtpsWriterProxy;

pub trait RtpsStatefulReader {
    type WriterProxyType;

    fn matched_writers(&self) -> &[Self::WriterProxyType];
}

pub trait RtpsStatefulReaderOperations<L> {
    fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy<L>);
    fn matched_writer_remove(&mut self, writer_proxy_guid: &Guid);
    fn matched_writer_lookup(&self, a_writer_guid: &Guid) -> Option<&RtpsWriterProxy<L>>;
}
