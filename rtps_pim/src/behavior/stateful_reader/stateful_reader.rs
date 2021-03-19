use core::ops::{Deref, DerefMut};

use crate::{behavior::RTPSReader, types::GUID};

use super::RTPSWriterProxy;

pub trait RTPSStatefulReader<T: RTPSReader>: Deref<Target = T> + DerefMut {
    type WriterProxyType: RTPSWriterProxy;
    fn matched_writers(&self) -> &[Self::WriterProxyType];
    fn matched_writer_add(&mut self, a_writer_proxy: Self::WriterProxyType);
    fn matched_writer_remove(&mut self, writer_proxy_guid: &GUID);
    fn matched_writer_lookup(&self, a_writer_guid: GUID) -> Option<&Self::WriterProxyType>;
}
