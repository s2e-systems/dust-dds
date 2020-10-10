use std::sync::{Mutex, MutexGuard};
use std::collections::{VecDeque, BTreeSet};

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

    highest_sequence_number_sent: Mutex<SequenceNumber>,

    send_messages: Mutex<VecDeque<RtpsSubmessage>>,
}

impl ReaderLocator {
    pub fn new(locator: Locator, writer_entity_id: EntityId, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            writer_entity_id,
            expects_inline_qos,
            highest_sequence_number_sent: Mutex::new(0),
            send_messages: Mutex::new(VecDeque::new()),
        }
    }

    pub fn locator(&self) -> &Locator {
        &self.locator
    }

    fn highest_sequence_number_sent(&self) -> MutexGuard<SequenceNumber> {
        self.highest_sequence_number_sent.lock().unwrap()
    }

    pub fn unsent_changes_reset(&self) {
        *self.highest_sequence_number_sent() = 0;
    }

    pub fn unsent_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        let mut unsent_changes_set = BTreeSet::new();

        // The for loop is made with the underlying sequence number type because it is not possible to implement the Step trait on Stable yet
        for unsent_sequence_number in
            (*self.highest_sequence_number_sent() + 1) ..= last_change_sequence_number
        {
            unsent_changes_set.insert(unsent_sequence_number);
        }

        unsent_changes_set
    }

    pub fn next_unsent_change(&self, last_change_sequence_number: SequenceNumber) -> Option<SequenceNumber> {
        let next_unsent_sequence_number = *self.highest_sequence_number_sent() + 1;
        if next_unsent_sequence_number > last_change_sequence_number {
            None
        } else {
            *self.highest_sequence_number_sent() = next_unsent_sequence_number;
            Some(next_unsent_sequence_number)
        }
    }

    pub fn run(&self, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) {
        if !self.unsent_changes(last_change_sequence_number).is_empty() {
            self.pushing_state(history_cache, last_change_sequence_number);
        }
    }

    fn pushing_state(&self, history_cache: &HistoryCache, last_change_sequence_number: SequenceNumber) {
        // This state is only valid if there are unsent changes
        assert!(!self.unsent_changes(last_change_sequence_number).is_empty());
    
        while let Some(next_unsent_seq_num) = self.next_unsent_change(last_change_sequence_number) {
            self.transition_t4(history_cache, next_unsent_seq_num);
        }
    }

    fn transition_t4(&self, history_cache: &HistoryCache, next_unsent_seq_num: SequenceNumber) {
        if let Some(cache_change) = history_cache
            .changes().iter().find(|cc| cc.sequence_number() == next_unsent_seq_num)
        {
            let data = data_from_cache_change(cache_change, ENTITYID_UNKNOWN);
            self.send_messages.lock().unwrap().push_back(RtpsSubmessage::Data(data));
        } else {
            let gap = Gap::new(
                BEHAVIOR_ENDIANNESS,
                ENTITYID_UNKNOWN, 
                self.writer_entity_id,
                next_unsent_seq_num,
            BTreeSet::new());

            self.send_messages.lock().unwrap().push_back(RtpsSubmessage::Gap(gap));
        }
    }
}