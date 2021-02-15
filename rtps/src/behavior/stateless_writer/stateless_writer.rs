use std::ops::{Deref, DerefMut};

use crate::{
    behavior::{types::Duration, Writer},
    types::{Locator, ReliabilityKind, TopicKind, GUID},
};

use super::ReaderLocator;

pub struct StatelessWriter {
    pub writer: Writer,
    pub reader_locators: Vec<ReaderLocator>,
}

impl Deref for StatelessWriter {
    type Target = Writer;
    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}
impl DerefMut for StatelessWriter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl StatelessWriter {
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
            reader_locators: Vec::new(),
        }
    }
    pub fn reader_locator_add(&mut self, a_locator: ReaderLocator) {
        self.reader_locators.push(a_locator);
    }

    pub fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.retain(|rl| &rl.locator != a_locator);
    }

    pub fn unsent_changes_reset(&mut self) {
        for rl in self.reader_locators.iter_mut() {
            rl.unsent_changes_reset();
        }
    }
}
