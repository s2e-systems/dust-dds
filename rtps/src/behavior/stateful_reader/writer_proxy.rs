use std::collections::BTreeSet;
use rust_dds_interface::types::SequenceNumber;
use crate::types::{GUID, Locator};

struct ChangesFromWriter {
    highest_processed_sequence_number: SequenceNumber,
    unknown_changes: BTreeSet<SequenceNumber>,
    lost_changes: BTreeSet<SequenceNumber>,
    missing_changes: BTreeSet<SequenceNumber>,
    irrelevant_changes: BTreeSet<SequenceNumber>,
}

pub struct WriterProxy {
    pub remote_writer_guid: GUID,
    // Groups are not supported yet:
     // remoteGroupEntityId: EntityId_t,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    // data_max_size_serialized: Long,
    changes_from_writer: ChangesFromWriter,
}

impl WriterProxy {
    pub fn new(
        remote_writer_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>, 
        ) -> Self {
            let changes_from_writer = ChangesFromWriter {
                highest_processed_sequence_number: 0,
                unknown_changes: BTreeSet::new(),
                lost_changes: BTreeSet::new(),
                missing_changes: BTreeSet::new(),
                irrelevant_changes: BTreeSet::new(),
            };

            Self {
                remote_writer_guid,
                unicast_locator_list,
                multicast_locator_list,
                changes_from_writer,
        }
    }

    /// This operation returns the maximum SequenceNumber_t among the changes_from_writer changes in the RTPS WriterProxy
    /// that are available for access by the DDS DataReader. The condition to make any CacheChange ‘a_change’ available 
    /// for ‘access’ by the DDS DataReader is that there are no changes from the RTPS Writer with SequenceNumber_t smaller
    /// than or equal to a_change.sequenceNumber that have status MISSING or UNKNOWN. In other words, the available_changes_max
    /// and all previous changes are either RECEIVED or LOST.
    pub fn available_changes_max(&mut self) -> SequenceNumber {

        // The assumption is that all the numbers up to the highest processed sequence number
        // are read (which is typically the scenario). Whatever is not read is
        // marked in its own array as missing, unknown or lost. Therefore, the maximum avaiable
        // change is the minimum value between the missing, unknown and received/lost samples.

        let lowest_before_missing =
            self.changes_from_writer.missing_changes
            .iter().next()
            .map_or(self.changes_from_writer.highest_processed_sequence_number, |x| *x-1);

        
        let lowest_unknown = 
        self.changes_from_writer.unknown_changes
        .iter().next()
        .map_or(self.changes_from_writer.highest_processed_sequence_number, |x| *x-1);

        self.changes_from_writer.highest_processed_sequence_number.min(lowest_unknown).min(lowest_before_missing)
    }

    /// This operation modifies the status of a ChangeFromWriter to indicate that the CacheChange with the
    /// SequenceNumber_t ‘a_seq_num’ is irrelevant to the RTPS Reader.
    pub fn irrelevant_change_set(&mut self, a_seq_num: SequenceNumber) {
        self.received_change_set(a_seq_num);
        self.changes_from_writer.irrelevant_changes.insert(a_seq_num);
    }

    /// This operation modifies the status stored in ChangeFromWriter for any changes in the WriterProxy whose 
    /// status is MISSING or UNKNOWN and have sequence numbers lower than ‘first_available_seq_num.’ The status of 
    /// those changes is modified to LOST indicating that the changes are no longer available in the WriterHistoryCache
    /// of the RTPS Writer represented by the RTPS WriterProxy
    pub fn lost_changes_update(&mut self, first_available_seq_num: SequenceNumber) {
        let remaining_unknown = self.changes_from_writer.unknown_changes.split_off(&first_available_seq_num);
        self.changes_from_writer.lost_changes.append(&mut self.changes_from_writer.unknown_changes);
        self.changes_from_writer.unknown_changes = remaining_unknown;
        
        let remaining_missing = self.changes_from_writer.missing_changes.split_off(&first_available_seq_num);
        self.changes_from_writer.lost_changes.append(&mut self.changes_from_writer.missing_changes);
        self.changes_from_writer.missing_changes = remaining_missing;

        if first_available_seq_num > self.changes_from_writer.highest_processed_sequence_number {
            for seq_num in self.changes_from_writer.highest_processed_sequence_number+1 .. first_available_seq_num {
                self.changes_from_writer.lost_changes.insert(seq_num);
            }
            self.changes_from_writer.highest_processed_sequence_number = first_available_seq_num - 1;
        }
    }

    /// This operation returns the subset of changes for the WriterProxy that have status ‘MISSING.’
    /// The changes with status ‘MISSING’ represent the set of changes available in the HistoryCache of the RTPS Writer
    /// represented by the RTPS WriterProxy that have not been received by the RTPS Reader.
    pub fn missing_changes(&mut self) -> BTreeSet<SequenceNumber> {
        self.changes_from_writer.missing_changes.clone()
    }

    /// This operation modifies the status stored in ChangeFromWriter for any changes in the WriterProxy whose status is UNKNOWN 
    /// and have sequence numbers smaller or equal to ‘last_available_seq_num.’ The status of those changes is modified 
    /// from UNKNOWN to MISSING indicating that the changes are available at the WriterHistoryCache of the RTPS Writer represented 
    /// by the RTPS WriterProxy but have not been received by the RTPS Reader
    pub fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber) {
        let remaining_unknown = self.changes_from_writer.unknown_changes.split_off(&last_available_seq_num);
        self.changes_from_writer.missing_changes.append(&mut self.changes_from_writer.unknown_changes);
        self.changes_from_writer.unknown_changes = remaining_unknown;

        if last_available_seq_num > self.changes_from_writer.highest_processed_sequence_number {
            for seq_num in self.changes_from_writer.highest_processed_sequence_number+1 ..= last_available_seq_num {
                self.changes_from_writer.missing_changes.insert(seq_num);
            }
            self.changes_from_writer.highest_processed_sequence_number = last_available_seq_num;
        }
    }

    /// This operation modifies the status of the ChangeFromWriter that refers to the CacheChange with the
    /// SequenceNumber_t ‘a_seq_num.’ The status of the change is set to ‘RECEIVED,’ indicating it has been received.
    pub fn received_change_set(&mut self, a_seq_num: SequenceNumber) {
        if a_seq_num > self.changes_from_writer.highest_processed_sequence_number {
            for seq_num in  self.changes_from_writer.highest_processed_sequence_number+1 .. a_seq_num {
                self.changes_from_writer.unknown_changes.insert(seq_num);
            }
            self.changes_from_writer.highest_processed_sequence_number = a_seq_num;
        } else {
            self.changes_from_writer.unknown_changes.remove(&a_seq_num);
            self.changes_from_writer.missing_changes.remove(&a_seq_num);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;

    #[test]
    fn received_changes_set_sequential_ordered() {
        let remote_writer_guid = GUID::new([1;12], ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER );
        let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

        // Originally the writer proxy doesn't know of any available changes
        assert_eq!(writer_proxy.available_changes_max(), 0);

        // If the next sequence number is received the unknown changes are empty
        writer_proxy.received_change_set(1);
        assert_eq!(writer_proxy.available_changes_max(), 1);
        assert!(writer_proxy.changes_from_writer.unknown_changes.is_empty());

        // If the next sequence number is received the unknown changes are empty
        writer_proxy.received_change_set(2);
        assert_eq!(writer_proxy.available_changes_max(), 2);
        assert!(writer_proxy.changes_from_writer.unknown_changes.is_empty());
    }

    #[test]
    fn received_changes_set_unordered() {
        let remote_writer_guid = GUID::new([1;12], ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER );
        let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

        // Originally the writer proxy doesn't know of any available changes
        assert_eq!(writer_proxy.available_changes_max(), 0);

        // If the next sequence number jumps two number then change 1 and 2 are unknown
        writer_proxy.received_change_set(3);
        assert_eq!(writer_proxy.available_changes_max(), 0);
        assert_eq!(writer_proxy.changes_from_writer.unknown_changes, [1, 2].iter().cloned().collect());

        // If sequence number 2 is received the only unknown change is 1
        writer_proxy.received_change_set(1);
        assert_eq!(writer_proxy.available_changes_max(), 1);
        assert_eq!(writer_proxy.changes_from_writer.unknown_changes, [2].iter().cloned().collect());

        // If sequence number 1 is received after there are no more unknown changes
        writer_proxy.received_change_set(2);
        assert_eq!(writer_proxy.available_changes_max(), 3);
        assert!(writer_proxy.changes_from_writer.unknown_changes.is_empty());
    }

    #[test]
    fn lost_changes_only() {
        let remote_writer_guid = GUID::new([1;12], ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER );
        let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

        writer_proxy.lost_changes_update(3);

        assert_eq!(writer_proxy.available_changes_max(), 2);
        assert_eq!(writer_proxy.changes_from_writer.lost_changes, [1, 2].iter().cloned().collect());

        writer_proxy.lost_changes_update(5);
        assert_eq!(writer_proxy.available_changes_max(), 4);
        assert_eq!(writer_proxy.changes_from_writer.lost_changes, [1, 2, 3, 4].iter().cloned().collect());
    }

    #[test]
    fn received_and_lost_changes() {
        let remote_writer_guid = GUID::new([1;12], ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER );
        let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

        writer_proxy.received_change_set(3);
        writer_proxy.lost_changes_update(3);

        assert_eq!(writer_proxy.available_changes_max(), 3);
        assert_eq!(writer_proxy.changes_from_writer.lost_changes, [1, 2].iter().cloned().collect());

        writer_proxy.lost_changes_update(5);
        assert_eq!(writer_proxy.available_changes_max(), 4);
        assert_eq!(writer_proxy.changes_from_writer.lost_changes, [1, 2, 4].iter().cloned().collect());
    }

    #[test]
    fn missing_changes_only() {
        let remote_writer_guid = GUID::new([1;12], ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER );
        let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

        writer_proxy.missing_changes_update(3);

        assert_eq!(writer_proxy.available_changes_max(), 0);
        assert_eq!(writer_proxy.changes_from_writer.missing_changes, [1, 2, 3].iter().cloned().collect());

        writer_proxy.missing_changes_update(5);
        assert_eq!(writer_proxy.available_changes_max(), 0);
        assert_eq!(writer_proxy.changes_from_writer.missing_changes, [1, 2, 3, 4, 5].iter().cloned().collect());
    }

    #[test]
    fn received_lost_and_missing_changes() {
        let remote_writer_guid = GUID::new([1;12], ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER );
        let mut writer_proxy = WriterProxy::new(remote_writer_guid, vec![], vec![]);

        // Received sample number 4. Verify that all sample numbers before are unknown
        writer_proxy.received_change_set(4);
        assert_eq!(writer_proxy.available_changes_max(), 0);
        assert_eq!(writer_proxy.changes_from_writer.unknown_changes, [1, 2, 3].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.missing_changes, [].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.lost_changes, [].iter().cloned().collect());

        // Mark sample 1 as lost and verify that now samples 2 and 3 are unknown
        writer_proxy.lost_changes_update(2);
        assert_eq!(writer_proxy.available_changes_max(), 1);
        assert_eq!(writer_proxy.changes_from_writer.unknown_changes, [2, 3].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.missing_changes, [].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.lost_changes, [1].iter().cloned().collect());

        // Mark sample 2 as missing
        writer_proxy.missing_changes_update(3);
        assert_eq!(writer_proxy.available_changes_max(), 1);
        assert_eq!(writer_proxy.changes_from_writer.unknown_changes, [3].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.missing_changes, [2].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.lost_changes, [1].iter().cloned().collect());

        // Mark sample 2 as received
        writer_proxy.received_change_set(2);
        assert_eq!(writer_proxy.available_changes_max(), 2);
        assert_eq!(writer_proxy.changes_from_writer.unknown_changes, [3].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.missing_changes, [].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.lost_changes, [1].iter().cloned().collect());

        // Mark samples 1 and 2 as lost and check that nothing happens
        writer_proxy.lost_changes_update(3);
        assert_eq!(writer_proxy.available_changes_max(), 2);
        assert_eq!(writer_proxy.changes_from_writer.unknown_changes, [3].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.missing_changes, [].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.lost_changes, [1].iter().cloned().collect());

        // Mark sample 3 as missing
        writer_proxy.missing_changes_update(4);
        assert_eq!(writer_proxy.available_changes_max(), 2);
        assert_eq!(writer_proxy.changes_from_writer.unknown_changes, [].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.missing_changes, [3].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.lost_changes, [1].iter().cloned().collect());

        // Mark sample 3 as lost
        writer_proxy.lost_changes_update(4);
        assert_eq!(writer_proxy.available_changes_max(), 4);
        assert_eq!(writer_proxy.changes_from_writer.unknown_changes, [].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.missing_changes, [].iter().cloned().collect());
        assert_eq!(writer_proxy.changes_from_writer.lost_changes, [1, 3].iter().cloned().collect());
    }
}
