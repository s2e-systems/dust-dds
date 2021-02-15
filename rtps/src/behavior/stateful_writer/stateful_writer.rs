use std::ops::{Deref, DerefMut};

use crate::{
    behavior::{types::Duration, Writer},
    types::{Locator, ReliabilityKind, TopicKind, GUID},
};

use super::ReaderProxy;

pub struct StatefulWriter {
    pub writer: Writer,
    pub matched_readers: Vec<ReaderProxy>,
}

impl Deref for StatefulWriter {
    type Target = Writer;
    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}
impl DerefMut for StatefulWriter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl StatefulWriter {
    pub fn new(
        guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_sized_serialized: Option<i32>,
    ) -> Self {
        let writer = Writer::new(
            guid,
            unicast_locator_list,
            multicast_locator_list,
            topic_kind,
            reliability_level,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_sized_serialized,
        );
        Self {
            writer,
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
