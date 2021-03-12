use std::ops::{Deref, DerefMut};

use rust_rtps::behavior::{RTPSReaderProxy, RTPSStatefulWriter, RTPSWriter};

pub struct StatefulWriter<W: RTPSWriter, R: RTPSReaderProxy> {
    writer: W,
    matched_reades: Vec<R>,
}

impl<W: RTPSWriter, R: RTPSReaderProxy> Deref for StatefulWriter<W, R> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<W: RTPSWriter, R: RTPSReaderProxy> DerefMut for StatefulWriter<W, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl<W: RTPSWriter, R: RTPSReaderProxy> RTPSStatefulWriter<W> for StatefulWriter<W, R> {
    type ReaderProxyType = R;

    fn matched_readers(&self) -> &[Self::ReaderProxyType] {
        todo!()
    }

    fn matched_reader_add(&mut self, _a_reader_proxy: Self::ReaderProxyType) {
        todo!()
    }

    fn matched_reader_remove(&mut self, _reader_proxy_guid: &rust_rtps::types::GUID) {
        todo!()
    }

    fn matched_reader_lookup(
        &self,
        _a_reader_guid: rust_rtps::types::GUID,
    ) -> Option<&Self::ReaderProxyType> {
        todo!()
    }

    fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn matched_reader_add() {
        // let stateful_writer = StatefulWriter::new();
    }
}
