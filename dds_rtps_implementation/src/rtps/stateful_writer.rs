use std::{marker::PhantomData, ops::{Deref, DerefMut}};

use rust_rtps::behavior::{RTPSReaderProxy, RTPSStatefulWriter, RTPSWriter};

pub struct StatefulWriter<'a, W: RTPSWriter, R: RTPSReaderProxy<'a>> {
    writer: W,
    matched_reades: Vec<R>,
    phantom: PhantomData<&'a ()>,
}

impl<'a, W: RTPSWriter, R: RTPSReaderProxy<'a>> Deref for StatefulWriter<'a, W, R> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<'a, W: RTPSWriter, R: RTPSReaderProxy<'a>> DerefMut for StatefulWriter<'a, W, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl<'a, W: RTPSWriter, R: RTPSReaderProxy<'a>> RTPSStatefulWriter<'a, W> for StatefulWriter<'a, W, R> {
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
