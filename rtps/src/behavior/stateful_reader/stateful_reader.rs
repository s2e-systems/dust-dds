use std::ops::{Deref, DerefMut};

use crate::{behavior::Reader, types::GUID};

use super::WriterProxy;

pub trait StatefulReader<T: Reader>: Deref<Target = T> + DerefMut {
    fn matched_writers(&self) -> &[WriterProxy];
    fn matched_writer_add(&mut self, a_writer_proxy: WriterProxy);
    fn matched_writer_remove(&mut self, writer_proxy_guid: &GUID);
    fn matched_writer_lookup(&self, a_writer_guid: GUID) -> Option<&WriterProxy>;
}
