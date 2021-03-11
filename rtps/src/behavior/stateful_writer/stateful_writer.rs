use core::ops::{Deref, DerefMut};

use crate::{behavior::RTPSWriter, types::GUID};

use super::RTPSReaderProxy;

pub trait RTPSStatefulWriter<T: RTPSWriter>: Deref<Target = T> + DerefMut {
    type ReaderProxyType: RTPSReaderProxy;
    fn matched_readers(&self) -> &[Self::ReaderProxyType];
    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType);
    fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID);
    fn matched_reader_lookup(&self, a_reader_guid: GUID) -> Option<&Self::ReaderProxyType>;
    fn is_acked_by_all(&self) -> bool;
}
