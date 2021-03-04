use core::ops::{Deref, DerefMut};

use crate::{behavior::Writer, types::GUID};

use super::ReaderProxy;

pub trait StatefulWriter<T: Writer>: Deref<Target = T> + DerefMut {
    type ReaderProxyType: ReaderProxy;
    fn matched_readers(&self) -> &[Self::ReaderProxyType];
    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType);
    fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID);
    fn matched_reader_lookup(&self, a_reader_guid: GUID) -> Option<&Self::ReaderProxyType>;
    fn is_acked_by_all(&self) -> bool;
}
