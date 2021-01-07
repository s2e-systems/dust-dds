use std::{collections::HashMap, ops::{Deref, DerefMut}};

use super::best_effort_reader_locator::BestEffortReaderLocator;
use crate::rtps::behavior::endpoint_traits::{CacheChangeSender, DestinedMessages};
use crate::rtps::behavior::Writer;
use crate::rtps::types::{Locator, ReliabilityKind, GUID};
use crate::types::TopicKind;

pub struct StatelessWriter {
    pub writer: Writer,
    reader_locators: HashMap<Locator, BestEffortReaderLocator>,
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
            data_max_sized_serialized,
        );

        Self {
            writer,
            reader_locators: HashMap::new(),
        }
    }

    pub fn reader_locator_add(&mut self, a_locator: Locator) {
        self.reader_locators
            .insert(a_locator, BestEffortReaderLocator::new(a_locator));
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

impl CacheChangeSender for StatelessWriter {
    fn produce_messages(&mut self) -> Vec<DestinedMessages> {
        let mut output = Vec::new();
        for (&locator, reader_locator) in self.reader_locators.iter_mut() {
            let messages = reader_locator.produce_messages(
                &self.writer.writer_cache,
                self.writer.endpoint.entity.guid.entity_id(),
                self.writer.last_change_sequence_number,
            );
            if !messages.is_empty() {
                output.push(DestinedMessages::SingleDestination { locator, messages });
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps::behavior::data_from_cache_change;
    use crate::rtps::messages::RtpsSubmessage;
    use crate::rtps::types::constants::{ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER, ENTITYID_UNKNOWN};

    use crate::types::ChangeKind;
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
            data_max_sized_serialized,
        );

        let locator_1 = Locator::new_udpv4(1000, [10, 11, 12, 13]);
        stateless_writer.reader_locator_add(locator_1);
        stateless_writer.reader_locator_remove(&locator_1);

        assert_eq!(stateless_writer.reader_locators.len(), 0);
    }

    #[test]
    fn produce_messages_single_locator() {
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
            data_max_sized_serialized,
        );

        let locator_1 = Locator::new_udpv4(1000, [10, 11, 12, 13]);
        stateless_writer.reader_locator_add(locator_1);

        let cache_change1 = stateless_writer.writer.new_change(
            ChangeKind::Alive,
            Some(vec![1, 2, 3]),
            None,
            [1; 16],
        );
        stateless_writer
            .writer
            .writer_cache
            .add_change(cache_change1.clone());

        let cache_change2 = stateless_writer.writer.new_change(
            ChangeKind::Alive,
            Some(vec![3, 4, 5]),
            None,
            [2; 16],
        );
        stateless_writer
            .writer
            .writer_cache
            .add_change(cache_change2.clone());

        let destined_messages_vec = stateless_writer.produce_messages();
        let expected_destined_message = DestinedMessages::SingleDestination {
            locator: Locator::new_udpv4(1000, [10, 11, 12, 13]),
            messages: vec![
                RtpsSubmessage::Data(data_from_cache_change(&cache_change1, ENTITYID_UNKNOWN)),
                RtpsSubmessage::Data(data_from_cache_change(&cache_change2, ENTITYID_UNKNOWN)),
            ],
        };
        assert_eq!(destined_messages_vec.len(), 1);
        assert!(destined_messages_vec.contains(&expected_destined_message));
    }

    #[test]
    fn produce_messages_multiple_locators() {
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
            data_max_sized_serialized,
        );

        let locator_1 = Locator::new_udpv4(1000, [10, 11, 12, 13]);
        stateless_writer.reader_locator_add(locator_1);

        let cache_change1 = stateless_writer.writer.new_change(
            ChangeKind::Alive,
            Some(vec![1, 2, 3]),
            None,
            [1; 16],
        );
        stateless_writer
            .writer
            .writer_cache
            .add_change(cache_change1.clone());

        stateless_writer.produce_messages();

        let locator_2 = Locator::new_udpv4(21000, [10, 11, 12, 13]);
        stateless_writer.reader_locator_add(locator_2);

        let cache_change2 = stateless_writer.writer.new_change(
            ChangeKind::Alive,
            Some(vec![4, 5, 6]),
            None,
            [1; 16],
        );
        stateless_writer
            .writer
            .writer_cache
            .add_change(cache_change2.clone());

        let destined_messages_vec = stateless_writer.produce_messages();

        let expected_destined_message1 = DestinedMessages::SingleDestination {
            locator: Locator::new_udpv4(1000, [10, 11, 12, 13]),
            messages: vec![RtpsSubmessage::Data(data_from_cache_change(
                &cache_change2,
                ENTITYID_UNKNOWN,
            ))],
        };
        let expected_destined_message2 = DestinedMessages::SingleDestination {
            locator: Locator::new_udpv4(21000, [10, 11, 12, 13]),
            messages: vec![
                RtpsSubmessage::Data(data_from_cache_change(&cache_change1, ENTITYID_UNKNOWN)),
                RtpsSubmessage::Data(data_from_cache_change(&cache_change2, ENTITYID_UNKNOWN)),
            ],
        };
        assert_eq!(destined_messages_vec.len(), 2);
        assert!(destined_messages_vec.contains(&expected_destined_message1));
        assert!(destined_messages_vec.contains(&expected_destined_message2));
    }

    #[test]
    fn unsent_changes_reset_multiple_locators() {
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
            data_max_sized_serialized,
        );

        let locator_1 = Locator::new_udpv4(1000, [10, 11, 12, 13]);
        stateless_writer.reader_locator_add(locator_1);

        let locator_2 = Locator::new_udpv4(21000, [10, 11, 12, 13]);
        stateless_writer.reader_locator_add(locator_2);

        let cache_change1 = stateless_writer.writer.new_change(
            ChangeKind::Alive,
            Some(vec![1, 2, 3]),
            None,
            [1; 16],
        );
        stateless_writer
            .writer
            .writer_cache
            .add_change(cache_change1.clone());

        let cache_change2 = stateless_writer.writer.new_change(
            ChangeKind::Alive,
            Some(vec![4, 5, 6]),
            None,
            [1; 16],
        );
        stateless_writer
            .writer
            .writer_cache
            .add_change(cache_change2.clone());

        stateless_writer.produce_messages();

        let destined_messages_vec_before_reset = stateless_writer.produce_messages();

        stateless_writer.unsent_changes_reset();

        let destined_messages_vec_after_reset = stateless_writer.produce_messages();

        let expected_destined_message1 = DestinedMessages::SingleDestination {
            locator: Locator::new_udpv4(1000, [10, 11, 12, 13]),
            messages: vec![
                RtpsSubmessage::Data(data_from_cache_change(&cache_change1, ENTITYID_UNKNOWN)),
                RtpsSubmessage::Data(data_from_cache_change(&cache_change2, ENTITYID_UNKNOWN)),
            ],
        };
        let expected_destined_message2 = DestinedMessages::SingleDestination {
            locator: Locator::new_udpv4(21000, [10, 11, 12, 13]),
            messages: vec![
                RtpsSubmessage::Data(data_from_cache_change(&cache_change1, ENTITYID_UNKNOWN)),
                RtpsSubmessage::Data(data_from_cache_change(&cache_change2, ENTITYID_UNKNOWN)),
            ],
        };
        assert_eq!(destined_messages_vec_before_reset.len(), 0);
        assert_eq!(destined_messages_vec_after_reset.len(), 2);
        assert!(destined_messages_vec_after_reset.contains(&expected_destined_message1));
        assert!(destined_messages_vec_after_reset.contains(&expected_destined_message2));
    }
}
