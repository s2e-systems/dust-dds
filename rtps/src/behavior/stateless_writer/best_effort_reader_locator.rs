use super::reader_locator::ReaderLocator;
use crate::behavior::{data_from_cache_change, BEHAVIOR_ENDIANNESS};
use crate::messages::submessages::Gap;
use crate::messages::RtpsSubmessage;
use crate::types::constants::ENTITYID_UNKNOWN;
use crate::types::{EntityId, Locator};
use std::collections::BTreeSet;

use rust_dds_interface::history_cache::HistoryCache;
use rust_dds_interface::types::SequenceNumber;

pub struct BestEffortReaderLocator(ReaderLocator);

impl std::ops::Deref for BestEffortReaderLocator {
    type Target = ReaderLocator;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for BestEffortReaderLocator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BestEffortReaderLocator {
    pub fn new(locator: Locator) -> Self {
        Self(ReaderLocator::new(locator))
    }

    pub fn produce_messages(
        &mut self,
        history_cache: &HistoryCache,
        writer_entity_id: EntityId,
        last_change_sequence_number: SequenceNumber,
    ) -> Vec<RtpsSubmessage> {
        let mut message_queue = Vec::new();
        if !self.unsent_changes(last_change_sequence_number).is_empty() {
            self.pushing_state(
                history_cache,
                writer_entity_id,
                last_change_sequence_number,
                &mut message_queue,
            );
        }
        message_queue
    }

    fn pushing_state(
        &mut self,
        history_cache: &HistoryCache,
        writer_entity_id: EntityId,
        last_change_sequence_number: SequenceNumber,
        message_queue: &mut Vec<RtpsSubmessage>,
    ) {
        while let Some(next_unsent_seq_num) = self.next_unsent_change(last_change_sequence_number) {
            self.transition_t4(
                history_cache,
                writer_entity_id,
                next_unsent_seq_num,
                message_queue,
            );
        }
    }

    fn transition_t4(
        &mut self,
        history_cache: &HistoryCache,
        writer_entity_id: EntityId,
        next_unsent_seq_num: SequenceNumber,
        message_queue: &mut Vec<RtpsSubmessage>,
    ) {
        if let Some(cache_change) = history_cache.get_change(next_unsent_seq_num) {
            let data = data_from_cache_change(cache_change, ENTITYID_UNKNOWN);
            message_queue.push(RtpsSubmessage::Data(data));
        } else {
            let gap = Gap::new(
                BEHAVIOR_ENDIANNESS,
                ENTITYID_UNKNOWN,
                writer_entity_id,
                next_unsent_seq_num,
                BTreeSet::new(),
            );

            message_queue.push(RtpsSubmessage::Gap(gap));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
    use crate::types::GUID;

    use rust_dds_interface::cache_change::CacheChange;
    use rust_dds_interface::types::ChangeKind;

    #[test]
    fn produce_data_and_gap_messages() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);

        let mut best_effort_reader_locator = BestEffortReaderLocator::new(locator);

        let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
        let mut history_cache = HistoryCache::default();
        // Run without any change being created or added in the cache. No message should be sent
        let last_change_sequence_number = 0;
        let messages_vec = best_effort_reader_locator.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
        );

        assert!(messages_vec.is_empty());

        // Add one change to the history cache and run with that change as the last one. One Data submessage should be sent
        let writer_guid = GUID::new([5; 12], writer_entity_id);
        let instance_handle = [1; 16];
        let cache_change_seq1 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid.into(),
            instance_handle,
            1,
            Some(vec![1, 2, 3]),
            None,
        );
        let expected_data_submessage = data_from_cache_change(&cache_change_seq1, ENTITYID_UNKNOWN);
        history_cache.add_change(cache_change_seq1).unwrap();

        let last_change_sequence_number = 1;
        let mut messages_vec = best_effort_reader_locator.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
        );

        let expected_submessage = RtpsSubmessage::Data(expected_data_submessage);
        let sent_message = messages_vec.pop().unwrap();
        assert!(messages_vec.is_empty());
        assert_eq!(sent_message, expected_submessage);

        // Run with the next sequence number without adding any change to the history cache. One Gap submessage should be sent
        let last_change_sequence_number = 2;
        let mut messages_vec = best_effort_reader_locator.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
        );

        let expected_submessage = RtpsSubmessage::Gap(Gap::new(
            BEHAVIOR_ENDIANNESS,
            ENTITYID_UNKNOWN,
            writer_entity_id,
            2,
            BTreeSet::new(),
        ));
        let sent_message = messages_vec.pop().unwrap();
        assert!(messages_vec.is_empty());
        assert_eq!(sent_message, expected_submessage);

        // Add one change to the history cache skipping one sequence number. One Gap and one Data submessage should be sent
        let cache_change_seq4 = CacheChange::new(
            ChangeKind::Alive,
            writer_guid.into(),
            instance_handle,
            4,
            Some(vec![4, 5, 6]),
            None,
        );
        let expected_data_submessage = data_from_cache_change(&cache_change_seq4, ENTITYID_UNKNOWN);
        history_cache.add_change(cache_change_seq4).unwrap();

        let last_change_sequence_number = 4;
        let mut messages_vec = best_effort_reader_locator.produce_messages(
            &history_cache,
            writer_entity_id,
            last_change_sequence_number,
        );

        let expected_gap_submessage = RtpsSubmessage::Gap(Gap::new(
            BEHAVIOR_ENDIANNESS,
            ENTITYID_UNKNOWN,
            writer_entity_id,
            3,
            BTreeSet::new(),
        ));
        let expected_data_submessage = RtpsSubmessage::Data(expected_data_submessage);

        let sent_message_2 = messages_vec.pop().unwrap();
        let sent_message_1 = messages_vec.pop().unwrap();
        assert!(messages_vec.is_empty());
        assert_eq!(sent_message_1, expected_gap_submessage);
        assert_eq!(sent_message_2, expected_data_submessage);
    }
}
