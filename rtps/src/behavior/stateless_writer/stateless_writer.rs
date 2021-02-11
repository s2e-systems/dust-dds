use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{
    behavior::{types::Duration, Writer},
    types::{Locator, ReliabilityKind, TopicKind, GUID},
};

use super::ReaderLocator;

pub struct StatelessWriter {
    writer: Writer,
    reader_locators: HashMap<Locator, ReaderLocator>,
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
            reader_locators: HashMap::new(),
        }
    }
    pub fn reader_locator_add(&mut self, a_locator: Locator) {
        self.reader_locators
            .insert(a_locator, ReaderLocator::new(a_locator));
    }

    pub fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.remove(a_locator);
    }

    pub fn unsent_changes_reset(&mut self) {
        for (_, rl) in self.reader_locators.iter_mut() {
            rl.unsent_changes_reset();
        }
    }
}
