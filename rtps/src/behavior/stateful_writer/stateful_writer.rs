use std::ops::{Deref, DerefMut};

use crate::{
    behavior::{types::Duration, Writer},
    types::{Locator, ReliabilityKind, TopicKind, GUID},
};

use super::ReaderProxy;

pub struct StatefulWriter {
    pub matched_readers: Vec<ReaderProxy>,
}



impl StatefulWriter {
    pub fn new() -> Self {       
        Self {
            matched_readers: Vec::new(),
        }
    }

    pub fn matched_reader_add(&mut self, a_reader_proxy: ReaderProxy) {
        self.matched_readers.push(a_reader_proxy);
    }

    pub fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID) {
        self.matched_readers
            .retain(|rp| &rp.remote_reader_guid != reader_proxy_guid);
    }

    pub fn matched_reader_lookup(&self, a_reader_guid: GUID) -> Option<&ReaderProxy> {
        self.matched_readers
            .iter()
            .find(|&rp| &rp.remote_reader_guid == &a_reader_guid)
    }

    pub fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}
