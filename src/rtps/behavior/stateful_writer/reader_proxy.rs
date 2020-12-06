use std::collections::BTreeSet;

use crate::rtps::types::{Locator, GUID};

use crate::types::SequenceNumber;
struct ChangeForReader {
    highest_sequence_number_sent: SequenceNumber,
    highest_sequence_number_acknowledged: SequenceNumber,
    sequence_numbers_requested: BTreeSet<SequenceNumber>,
}

pub struct ReaderProxy {
    pub remote_reader_guid: GUID,
    // remoteGroupEntityId: EntityId_t,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    changes_for_reader: ChangeForReader,
    pub expects_inline_qos: bool,
    pub is_active: bool,
}

impl ReaderProxy {
    pub fn new(
        remote_reader_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        expects_inline_qos: bool,
        is_active: bool,
    ) -> Self {
        let changes_for_reader = ChangeForReader {
            highest_sequence_number_sent: 0,
            highest_sequence_number_acknowledged: 0,
            sequence_numbers_requested: BTreeSet::new(),
        };

        Self {
            remote_reader_guid,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
            changes_for_reader,
        }
    }

    pub fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        self.changes_for_reader.highest_sequence_number_acknowledged = committed_seq_num;
    }

    pub fn next_requested_change(&mut self) -> Option<SequenceNumber> {
        let next_requested_change = *self
            .changes_for_reader
            .sequence_numbers_requested
            .iter()
            .next()?;

        self.changes_for_reader
            .sequence_numbers_requested
            .remove(&next_requested_change);

        Some(next_requested_change)
    }

    pub fn next_unsent_change(
        &mut self,
        last_change_sequence_number: SequenceNumber,
    ) -> Option<SequenceNumber> {
        let next_unsent_sequence_number = self.changes_for_reader.highest_sequence_number_sent + 1;
        if next_unsent_sequence_number > last_change_sequence_number {
            None
        } else {
            self.changes_for_reader.highest_sequence_number_sent = next_unsent_sequence_number;
            Some(next_unsent_sequence_number)
        }
    }

    pub fn unsent_changes(
        &mut self,
        last_change_sequence_number: SequenceNumber,
    ) -> BTreeSet<SequenceNumber> {
        let mut unsent_changes_set = BTreeSet::new();

        for unsent_sequence_number in
            self.changes_for_reader.highest_sequence_number_sent + 1..=last_change_sequence_number
        {
            unsent_changes_set.insert(unsent_sequence_number);
        }

        unsent_changes_set
    }

    pub fn requested_changes(&mut self) -> BTreeSet<SequenceNumber> {
        self.changes_for_reader.sequence_numbers_requested.clone()
    }

    pub fn requested_changes_set(&mut self, req_seq_num_set: BTreeSet<SequenceNumber>) {
        let mut new_set = req_seq_num_set;
        self.changes_for_reader
            .sequence_numbers_requested
            .append(&mut new_set);
    }

    pub fn unacked_changes(
        &mut self,
        last_change_sequence_number: SequenceNumber,
    ) -> BTreeSet<SequenceNumber> {
        let mut unacked_changes_set = BTreeSet::new();

        for unsent_sequence_number in self.changes_for_reader.highest_sequence_number_acknowledged
            + 1..=last_change_sequence_number
        {
            unacked_changes_set.insert(unsent_sequence_number);
        }

        unacked_changes_set
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps::types::constants::ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR;

    #[test]
    fn unsent_changes() {
        let remote_reader_guid = GUID::new(
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        );
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);
        let unsent_changes_empty = reader_proxy.unsent_changes(0);
        let unsent_changes2 = reader_proxy.unsent_changes(2);

        assert!(unsent_changes_empty.is_empty());
        assert_eq!(unsent_changes2.len(), 2);
        assert!(unsent_changes2.contains(&1));
        assert!(unsent_changes2.contains(&2));
    }

    #[test]
    fn next_unsent_change() {
        let remote_reader_guid = GUID::new(
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        );
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let next_unsent_change1 = reader_proxy.next_unsent_change(2);
        let next_unsent_change2 = reader_proxy.next_unsent_change(2);
        let next_unsent_change_none = reader_proxy.next_unsent_change(2);

        assert_eq!(next_unsent_change1, Some(1));
        assert_eq!(next_unsent_change2, Some(2));
        assert_eq!(next_unsent_change_none, None);
    }

    #[test]
    fn non_compliant_last_change_sequence_number() {
        let remote_reader_guid = GUID::new(
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        );
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        reader_proxy.next_unsent_change(2);
        reader_proxy.next_unsent_change(2);
        reader_proxy.next_unsent_change(2);

        let next_unsent_change_lower_last_seq_num = reader_proxy.next_unsent_change(1);

        assert_eq!(next_unsent_change_lower_last_seq_num, None);
    }

    #[test]
    fn requested_changes_set() {
        let remote_reader_guid = GUID::new(
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        );
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let mut requested_changes = BTreeSet::new();
        requested_changes.insert(2);
        requested_changes.insert(3);
        requested_changes.insert(6);
        reader_proxy.requested_changes_set(requested_changes);

        assert_eq!(reader_proxy.requested_changes().len(), 3);
        assert!(reader_proxy.requested_changes().contains(&2));
        assert!(reader_proxy.requested_changes().contains(&3));
        assert!(reader_proxy.requested_changes().contains(&6));
    }

    #[test]
    fn next_requested_change() {
        let remote_reader_guid = GUID::new(
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        );
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let mut requested_changes = BTreeSet::new();
        requested_changes.insert(2);
        requested_changes.insert(3);
        requested_changes.insert(6);
        reader_proxy.requested_changes_set(requested_changes);

        assert_eq!(reader_proxy.next_requested_change(), Some(2));
        assert_eq!(reader_proxy.next_requested_change(), Some(3));
        assert_eq!(reader_proxy.next_requested_change(), Some(6));
        assert_eq!(reader_proxy.next_requested_change(), None);
    }

    #[test]
    fn unacked_changes() {
        let remote_reader_guid = GUID::new(
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        );
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        let unacked_changes = reader_proxy.unacked_changes(2);
        assert!(unacked_changes.contains(&1));
        assert!(unacked_changes.contains(&2));
    }

    #[test]
    fn unacked_changes_after_ack() {
        let remote_reader_guid = GUID::new(
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        );
        let mut reader_proxy = ReaderProxy::new(remote_reader_guid, vec![], vec![], false, true);

        reader_proxy.acked_changes_set(1);
        let unacked_changes = reader_proxy.unacked_changes(2);
        assert!(!unacked_changes.contains(&1));
        assert!(unacked_changes.contains(&2));
    }
}
