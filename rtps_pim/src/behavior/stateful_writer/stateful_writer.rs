use core::ops::{Deref, DerefMut};

use crate::behavior::RTPSWriter;

use super::RTPSReaderProxy;

pub trait RTPSStatefulWriter<T: RTPSWriter>: Deref<Target = T> + DerefMut {
    type ReaderProxyType: RTPSReaderProxy<Writer = T>;
    fn matched_readers(&self) -> &[Self::ReaderProxyType];
    fn matched_reader_add(&mut self, guid: T::GUID);
    fn matched_reader_remove(&mut self, reader_proxy_guid: &T::GUID);
    fn matched_reader_lookup(&self, a_reader_guid: T::GUID) -> Option<&Self::ReaderProxyType>;
    fn is_acked_by_all(&self) -> bool;
}
