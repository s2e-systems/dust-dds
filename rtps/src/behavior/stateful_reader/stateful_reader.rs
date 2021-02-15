use crate::{
    behavior::{types::Duration, Reader},
    types::{Locator, ReliabilityKind, TopicKind, GUID},
};
use std::ops::{Deref, DerefMut};

use super::WriterProxy;

pub struct StatefulReader {
    pub reader: Reader,
    matched_writers: Vec<WriterProxy>,
}

impl Deref for StatefulReader {
    type Target = Reader;
    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}
impl DerefMut for StatefulReader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}

impl StatefulReader {
    pub fn new(
        guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        expects_inline_qos: bool,
        heartbeat_response_delay: Duration,
        heartbeat_supression_duration: Duration,
    ) -> Self {
        let reader = Reader::new(
            guid,
            unicast_locator_list,
            multicast_locator_list,
            topic_kind,
            reliability_level,
            expects_inline_qos,
            heartbeat_response_delay,
            heartbeat_supression_duration,
        );
        Self {
            reader,
            matched_writers: Vec::new(),
        }
    }

    pub fn matched_writer_add(&mut self, a_writer_proxy: WriterProxy) {
        self.matched_writers.push(a_writer_proxy);
    }

    pub fn matched_writer_remove(&mut self, writer_proxy_guid: &GUID) {
        self.matched_writers
            .retain(|wp| &wp.remote_writer_guid != writer_proxy_guid);
    }

    pub fn matched_writer_lookup(&self, a_writer_guid: GUID) -> Option<&WriterProxy> {
        self.matched_writers
            .iter()
            .find(|&wp| &wp.remote_writer_guid == &a_writer_guid)
    }
}
