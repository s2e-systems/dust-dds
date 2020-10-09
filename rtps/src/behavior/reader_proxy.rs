use std::collections::{BTreeSet, VecDeque};
use std::sync::Mutex;

use crate::types::{Locator, SequenceNumber, GUID, GuidPrefix};

use crate::messages::RtpsSubmessage;

pub struct ChangeForReader {
    highest_sequence_number_sent: SequenceNumber,
    highest_sequence_number_acknowledged: SequenceNumber,
    sequence_numbers_requested: BTreeSet<SequenceNumber>,
}

impl ChangeForReader {
    fn new() -> Self {
        Self {
            highest_sequence_number_sent: 0,
            highest_sequence_number_acknowledged: 0,
            sequence_numbers_requested: BTreeSet::new(),
        }
    }

    fn next_unsent_change(&mut self, last_change_sequence_number: SequenceNumber) -> Option<SequenceNumber> {
        let next_unsent_sequence_number = self.highest_sequence_number_sent + 1;
        if next_unsent_sequence_number > last_change_sequence_number {
            None
        } else {
            self.highest_sequence_number_sent = next_unsent_sequence_number;
            Some(next_unsent_sequence_number)
        }
    }

    fn unsent_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        let mut unsent_changes_set = BTreeSet::new();

        for unsent_sequence_number in
            self.highest_sequence_number_sent + 1..=last_change_sequence_number
        {
            unsent_changes_set.insert(unsent_sequence_number);
        }

        unsent_changes_set
    }

    fn acked_changes(&self) -> SequenceNumber {
        self.highest_sequence_number_acknowledged
    }

    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        self.highest_sequence_number_acknowledged = committed_seq_num;
    }

    fn unacked_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        let mut unacked_changes_set = BTreeSet::new();

        for unsent_sequence_number in
            self.highest_sequence_number_acknowledged + 1..=last_change_sequence_number
        {
            unacked_changes_set.insert(unsent_sequence_number);
        }

        unacked_changes_set
    }

    fn requested_changes_set(&mut self, req_seq_num_set: BTreeSet<SequenceNumber>) {
        let mut new_set = req_seq_num_set;
        self.sequence_numbers_requested.append(&mut new_set);
    }

    fn requested_changes(&self) -> BTreeSet<SequenceNumber> {
        self.sequence_numbers_requested.clone()
    }

    fn next_requested_change(&mut self) -> Option<SequenceNumber> {
        let next_requested_change = *self.sequence_numbers_requested.iter().next()?;

        self.sequence_numbers_requested.remove(&next_requested_change);

        Some(next_requested_change)
    }
}


pub struct ReaderProxy {
    pub remote_reader_guid: GUID,
    // remoteGroupEntityId: EntityId_t,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    changes_for_reader: Mutex<ChangeForReader>,
    expects_inline_qos: bool,
    is_active: bool,

    pub send_messages: Mutex<VecDeque<RtpsSubmessage>>,
    pub received_messages: Mutex<VecDeque<(GuidPrefix, RtpsSubmessage)>>,
}


impl ReaderProxy {
    pub fn new(
        remote_reader_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        expects_inline_qos: bool,
        is_active: bool) -> Self {
            Self {
                remote_reader_guid,
                unicast_locator_list,
                multicast_locator_list,
                expects_inline_qos,
                is_active,
                changes_for_reader: Mutex::new(ChangeForReader::new()),
                send_messages: Mutex::new(VecDeque::new()),
                received_messages: Mutex::new(VecDeque::new()),
        }
    }

    pub fn acked_changes_set(&self, committed_seq_num: SequenceNumber) {
        self.changes_for_reader.lock().unwrap().acked_changes_set(committed_seq_num);
    }

    pub fn next_requested_change(&self) -> Option<SequenceNumber> {
        self.changes_for_reader.lock().unwrap().next_requested_change()
    }

    pub fn next_unsent_change(&self, last_change_sequence_number: SequenceNumber) -> Option<SequenceNumber> {
        self.changes_for_reader.lock().unwrap().next_unsent_change(last_change_sequence_number)
    }

    pub fn unsent_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        self.changes_for_reader.lock().unwrap().unsent_changes(last_change_sequence_number)
    }

    pub fn requested_changes(&self) -> BTreeSet<SequenceNumber> {
        self.changes_for_reader.lock().unwrap().requested_changes()
    }

    pub fn requested_changes_set(&self, req_seq_num_set: BTreeSet<SequenceNumber>) {
        self.changes_for_reader.lock().unwrap().requested_changes_set(req_seq_num_set);
    }

    pub fn unacked_changes(&self, last_change_sequence_number: SequenceNumber) -> BTreeSet<SequenceNumber> {
        self.changes_for_reader.lock().unwrap().unacked_changes(last_change_sequence_number)
    }

    pub fn unicast_locator_list(&self) -> &Vec<Locator> {
        &self.unicast_locator_list
    }

    pub fn multicast_locator_list(&self) -> &Vec<Locator> {
        &self.multicast_locator_list
    }

    pub fn remote_reader_guid(&self) -> &GUID {
        &self.remote_reader_guid
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR;


    #[test]
    fn reader_proxy_unsent_changes_operations() {
        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        // Check that a reader proxy that has no changes marked as sent doesn't reports no changes
        let no_change_in_writer_sequence_number = 0;
        assert_eq!(reader_proxy.next_unsent_change(no_change_in_writer_sequence_number), None);
        assert!(reader_proxy.unsent_changes(no_change_in_writer_sequence_number).is_empty());

        // Check the behaviour for a reader proxy starting with no changes sent and two changes in writer
        let two_changes_in_writer_sequence_number = 2;
        assert_eq!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).len(), 2);
        assert!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).contains(&1));
        assert!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).contains(&2));

        assert_eq!(reader_proxy.next_unsent_change(two_changes_in_writer_sequence_number), Some(1));
        assert_eq!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).len(), 1);
        assert!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).contains(&2));

        assert_eq!(reader_proxy.next_unsent_change(two_changes_in_writer_sequence_number), Some(2));
        assert!(reader_proxy.unsent_changes(two_changes_in_writer_sequence_number).is_empty());

        assert_eq!(reader_proxy.next_unsent_change(two_changes_in_writer_sequence_number), None);
    }

    #[test]
    fn reader_proxy_requested_changes_operations() {
        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        // Check that a reader proxy that has no changes marked as sent doesn't reports no changes
        assert!(reader_proxy.requested_changes().is_empty());
        assert_eq!(reader_proxy.next_requested_change(), None);

        // Insert some requested changes
        let mut requested_changes = BTreeSet::new();
        requested_changes.insert(2);
        requested_changes.insert(3);
        requested_changes.insert(6);
        reader_proxy.requested_changes_set(requested_changes);

        // Verify that the changes were correctly inserted and are removed in the correct order
        assert_eq!(reader_proxy.requested_changes().len(), 3);
        assert!(reader_proxy.requested_changes().contains(&2));
        assert!(reader_proxy.requested_changes().contains(&3));
        assert!(reader_proxy.requested_changes().contains(&6));

        assert_eq!(reader_proxy.next_requested_change(), Some(2));
        assert_eq!(reader_proxy.next_requested_change(), Some(3));
        assert_eq!(reader_proxy.requested_changes().len(), 1);
        assert!(reader_proxy.requested_changes().contains(&6));
        assert_eq!(reader_proxy.next_requested_change(), Some(6));
        assert_eq!(reader_proxy.next_requested_change(), None);


        // Verify that if requested changes are inserted when there are already requested changes
        // that the sets are not replaced
        let mut requested_changes_1 = BTreeSet::new();
        requested_changes_1.insert(2);
        requested_changes_1.insert(3);
        reader_proxy.requested_changes_set(requested_changes_1);

        let mut requested_changes_2 = BTreeSet::new();
        requested_changes_2.insert(2); // Repeated number
        requested_changes_2.insert(7);
        requested_changes_2.insert(9);
        reader_proxy.requested_changes_set(requested_changes_2);
        
        assert_eq!(reader_proxy.requested_changes().len(), 4);
        assert!(reader_proxy.requested_changes().contains(&2));
        assert!(reader_proxy.requested_changes().contains(&3));
        assert!(reader_proxy.requested_changes().contains(&7));
        assert!(reader_proxy.requested_changes().contains(&9));
    }

    #[test]
    fn reader_proxy_unacked_changes_operations() {
        let remote_reader_guid = GUID::new([1,2,3,4,5,6,7,8,9,10,11,12], ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR);
        let reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let no_change_in_writer = 0;
        assert!(reader_proxy.unacked_changes(no_change_in_writer).is_empty());

        let two_changes_in_writer = 2;
        assert_eq!(reader_proxy.unacked_changes(two_changes_in_writer).len(), 2);
        assert!(reader_proxy.unacked_changes(two_changes_in_writer).contains(&1));
        assert!(reader_proxy.unacked_changes(two_changes_in_writer).contains(&2));

        reader_proxy.acked_changes_set(1);
        assert_eq!(reader_proxy.unacked_changes(two_changes_in_writer).len(), 1);
        assert!(reader_proxy.unacked_changes(two_changes_in_writer).contains(&2));
    }
}