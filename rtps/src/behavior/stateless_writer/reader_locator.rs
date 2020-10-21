use std::collections::{VecDeque, BTreeSet};
use std::sync::mpsc;

use crate::types::{Locator, SequenceNumber, EntityId};
use crate::types::constants::ENTITYID_UNKNOWN;
use crate::structure::HistoryCache;
use crate::messages::RtpsSubmessage;
use crate::messages::submessages::Gap;
use crate::behavior::{data_from_cache_change, BEHAVIOR_ENDIANNESS};

pub struct ReaderLocator {
    //requested_changes: HashSet<CacheChange>,
    // unsent_changes: SequenceNumber,
    locator: Locator,
    writer_entity_id: EntityId,
    expects_inline_qos: bool,

    highest_sequence_number_sent: SequenceNumber,

    sender: mpsc::Sender<(Vec<Locator>,RtpsSubmessage)>,
}

impl ReaderLocator {
    pub fn new(locator: Locator, writer_entity_id: EntityId, expects_inline_qos: bool, sender: mpsc::Sender<(Vec<Locator>,RtpsSubmessage)>) -> Self {
        Self {
            locator,
            writer_entity_id,
            expects_inline_qos,
            highest_sequence_number_sent:0,
            sender,
        }
    }

    pub fn unsent_changes_reset(&mut self) {
        self.highest_sequence_number_sent = 0;
    }

    pub fn unsent_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        let mut unsent_changes_set = BTreeSet::new();

        // The for loop is made with the underlying sequence number type because it is not possible to implement the Step trait on Stable yet
        for unsent_sequence_number in
            self.highest_sequence_number_sent + 1 ..= last_change_sequence_number
        {
            unsent_changes_set.insert(unsent_sequence_number);
        }

        unsent_changes_set
    }

    pub fn next_unsent_change(&mut self, last_change_sequence_number: SequenceNumber) -> Option<SequenceNumber> {
        let next_unsent_sequence_number = self.highest_sequence_number_sent + 1;
        if next_unsent_sequence_number > last_change_sequence_number {
            None
        } else {
            self.highest_sequence_number_sent = next_unsent_sequence_number;
            Some(next_unsent_sequence_number)
        }
    }

    pub fn run(&mut self, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) {
        if !self.unsent_changes(last_change_sequence_number).is_empty() {
            self.pushing_state(history_cache, last_change_sequence_number);
        }
    }

    fn pushing_state(&mut self, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) {
        // This state is only valid if there are unsent changes
        debug_assert!(!self.unsent_changes(last_change_sequence_number).is_empty());
    
        while let Some(next_unsent_seq_num) = self.next_unsent_change(last_change_sequence_number) {
            self.transition_t4(history_cache, next_unsent_seq_num);
        }
    }

    fn transition_t4(&mut self, history_cache: &HistoryCache, next_unsent_seq_num: SequenceNumber) {
        if let Some(cache_change) = history_cache
            .changes().iter().find(|cc| cc.sequence_number() == next_unsent_seq_num)
        {
            let data = data_from_cache_change(cache_change, ENTITYID_UNKNOWN);
            self.sender.send((vec![self.locator], RtpsSubmessage::Data(data))).unwrap();
        } else {
            let gap = Gap::new(
                BEHAVIOR_ENDIANNESS,
                ENTITYID_UNKNOWN, 
                self.writer_entity_id,
                next_unsent_seq_num,
            BTreeSet::new());

            self.sender.send((vec![self.locator], RtpsSubmessage::Gap(gap))).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GUID, ChangeKind};
    use crate::types::constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
    use crate::structure::CacheChange;
    use rust_dds_interface::qos_policy::ResourceLimitsQosPolicy;

    #[test]
    fn unsent_change_operations() {
        let (sender, receiver) = mpsc::channel();
        let locator = Locator::new_udpv4(7400, [127,0,0,1]);
        let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
        let expects_inline_qos = false;
        let mut reader_locator = ReaderLocator::new(locator, writer_entity_id, expects_inline_qos, sender);

        let unsent_changes = reader_locator.unsent_changes(0);
        assert!(unsent_changes.is_empty());

        let unsent_changes = reader_locator.unsent_changes(2);
        assert_eq!(unsent_changes.len(), 2);
        assert!(unsent_changes.contains(&1));
        assert!(unsent_changes.contains(&2));

        let next_unsent_change = reader_locator.next_unsent_change(2).unwrap();
        assert_eq!(next_unsent_change, 1);
        let next_unsent_change = reader_locator.next_unsent_change(2).unwrap();
        assert_eq!(next_unsent_change, 2);
        let next_unsent_change = reader_locator.next_unsent_change(2);
        assert!(next_unsent_change.is_none());

        // Test also that the system is robust if the last_change_sequence_number input does not follow the precondition
        // of being a constantly increasing number
        let next_unsent_change = reader_locator.next_unsent_change(1);
        assert!(next_unsent_change.is_none());
    }

    #[test]
    fn unsent_changes_reset() {
        let (sender, receiver) = mpsc::channel();
        let locator = Locator::new_udpv4(7400, [127,0,0,1]);
        let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
        let expects_inline_qos = false;
        let mut reader_locator = ReaderLocator::new(locator, writer_entity_id, expects_inline_qos, sender);

        let next_unsent_change = reader_locator.next_unsent_change(2).unwrap();
        assert_eq!(next_unsent_change, 1);
        let next_unsent_change = reader_locator.next_unsent_change(2).unwrap();
        assert_eq!(next_unsent_change, 2);
        let next_unsent_change = reader_locator.next_unsent_change(2);
        assert!(next_unsent_change.is_none());

        reader_locator.unsent_changes_reset();

        let next_unsent_change = reader_locator.next_unsent_change(2).unwrap();
        assert_eq!(next_unsent_change, 1);
        let next_unsent_change = reader_locator.next_unsent_change(2).unwrap();
        assert_eq!(next_unsent_change, 2);
        let next_unsent_change = reader_locator.next_unsent_change(2);
        assert!(next_unsent_change.is_none());
    }

    #[test]
    fn run() {
        let (sender, receiver) = mpsc::channel();
        let locator = Locator::new_udpv4(7400, [127,0,0,1]);
        let writer_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER;
        let expects_inline_qos = false;
        let mut reader_locator = ReaderLocator::new(locator, writer_entity_id, expects_inline_qos, sender);

        let history_cache = HistoryCache::new(&ResourceLimitsQosPolicy::default());

        // Run without any change being created or added in the cache. No message should be sent
        let last_change_sequence_number = 0;
        reader_locator.run(&history_cache, last_change_sequence_number);

        assert!(receiver.try_recv().is_err());

        // Add one change to the history cache and run with that change as the last one. One Data submessage should be sent
        let writer_guid = GUID::new([5;12], writer_entity_id);
        let instance_handle = [1;16];
        let cache_change_seq1 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 1, Some(vec![1,2,3]), None);
        let expected_data_submessage = data_from_cache_change(&cache_change_seq1, ENTITYID_UNKNOWN);
        history_cache.add_change(cache_change_seq1).unwrap();

        let last_change_sequence_number = 1;
        reader_locator.run(&history_cache, last_change_sequence_number);

        let expected_submessage = RtpsSubmessage::Data(expected_data_submessage);
        let (locator_list, sent_message) = receiver.try_recv().unwrap();
        assert!(receiver.try_recv().is_err());
        assert_eq!(sent_message, expected_submessage);

        // Run with the next sequence number without adding any change to the history cache. One Gap submessage should be sent
        let last_change_sequence_number = 2;
        reader_locator.run(&history_cache, last_change_sequence_number);

        let expected_submessage = RtpsSubmessage::Gap(Gap::new(BEHAVIOR_ENDIANNESS, ENTITYID_UNKNOWN, writer_entity_id, 2, BTreeSet::new()));
        let (locator_list, sent_message) = receiver.try_recv().unwrap();
        assert!(receiver.try_recv().is_err());
        assert_eq!(sent_message, expected_submessage);

        // Add one change to the history cache skipping one sequence number. One Gap and one Data submessage should be sent
        let cache_change_seq4 = CacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, 4, Some(vec![4,5,6]), None);
        let expected_data_submessage = data_from_cache_change(&cache_change_seq4, ENTITYID_UNKNOWN);
        history_cache.add_change(cache_change_seq4).unwrap();

        let last_change_sequence_number = 4;
        reader_locator.run(&history_cache, last_change_sequence_number);

        let expected_gap_submessage = RtpsSubmessage::Gap(Gap::new(BEHAVIOR_ENDIANNESS, ENTITYID_UNKNOWN, writer_entity_id, 3, BTreeSet::new()));
        let expected_data_submessage = RtpsSubmessage::Data(expected_data_submessage);

        let (locator_list, sent_message_1) = receiver.try_recv().unwrap();
        let (locator_list, sent_message_2) = receiver.try_recv().unwrap();
        assert_eq!(sent_message_1, expected_gap_submessage);
        assert_eq!(sent_message_2, expected_data_submessage);

    }
}