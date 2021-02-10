use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{
    behavior::{types::Duration, Writer},
    types::{Locator, ReliabilityKind, TopicKind, GUID},
};

use super::reader_locator::ReaderLocator;

pub struct StatelessWriter {
    pub writer: Writer,
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
        assert!(
            reliability_level == ReliabilityKind::BestEffort,
            "Only BestEffort is supported on stateless writer"
        );

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TopicKind;
    use crate::{
        behavior::types::constants::DURATION_ZERO,
        types::constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
    };

    #[test]
    fn reader_locator_add() {
        let guid = GUID::new([1; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let push_mode = true;
        let data_max_sized_serialized = None;
        let mut stateless_writer = StatelessWriter::new(
            guid,
            topic_kind,
            reliability_level,
            push_mode,
            DURATION_ZERO,
            Duration::from_millis(200),
            DURATION_ZERO,
            data_max_sized_serialized,
        );

        let locator_1 = Locator::new_udpv4(1000, [10, 11, 12, 13]);
        stateless_writer.reader_locator_add(locator_1);

        assert_eq!(stateless_writer.reader_locators.len(), 1);
        assert_eq!(
            stateless_writer.reader_locators[&locator_1].locator,
            locator_1
        );
    }

    #[test]
    fn reader_locator_remove() {
        let guid = GUID::new([1; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER);
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let push_mode = true;
        let data_max_sized_serialized = None;
        let mut stateless_writer = StatelessWriter::new(
            guid,
            topic_kind,
            reliability_level,
            push_mode,
            DURATION_ZERO,
            Duration::from_millis(200),
            DURATION_ZERO,
            data_max_sized_serialized,
        );

        let locator_1 = Locator::new_udpv4(1000, [10, 11, 12, 13]);
        stateless_writer.reader_locator_add(locator_1);
        stateless_writer.reader_locator_remove(&locator_1);

        assert_eq!(stateless_writer.reader_locators.len(), 0);
    }
}
