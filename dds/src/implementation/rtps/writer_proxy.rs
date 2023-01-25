use std::{
    cmp::{max, min},
    collections::HashMap,
};

use super::{
    messages::types::FragmentNumber,
    types::{Count, EntityId, Guid, Locator, SequenceNumber},
};

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsWriterProxy {
    remote_writer_guid: Guid,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    data_max_size_serialized: Option<i32>,
    remote_group_entity_id: EntityId,
    first_available_seq_num: SequenceNumber,
    last_available_seq_num: SequenceNumber,
    irrelevant_changes: Vec<SequenceNumber>,
    received_changes: Vec<SequenceNumber>,
    must_send_acknacks: bool,
    last_received_heartbeat_count: Count,
    acknack_count: Count,

    frag_buffer: HashMap<SequenceNumber, HashMap<FragmentNumber, Vec<u8>>>,
}

impl RtpsWriterProxy {
    pub fn new(
        remote_writer_guid: Guid,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        data_max_size_serialized: Option<i32>,
        remote_group_entity_id: EntityId,
    ) -> Self {
        Self {
            remote_writer_guid,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
            data_max_size_serialized,
            remote_group_entity_id,
            first_available_seq_num: SequenceNumber::new(1),
            last_available_seq_num: SequenceNumber::new(0),
            irrelevant_changes: Vec::new(),
            received_changes: Vec::new(),
            must_send_acknacks: false,
            last_received_heartbeat_count: Count::new(0),
            acknack_count: Count::new(0),
            frag_buffer: HashMap::new(),
        }
    }

    pub fn push_data_frag(
        &mut self,
        sequence_number: SequenceNumber,
        fragment_number: FragmentNumber,
        data: Vec<u8>,
    ) {
        self.frag_buffer
            .entry(sequence_number)
            .or_insert(HashMap::new())
            .insert(fragment_number, data);
    }

    pub fn frag_buffer(&self) -> &HashMap<SequenceNumber, HashMap<FragmentNumber, Vec<u8>>> {
        &self.frag_buffer
    }

    pub fn extract_frag(&mut self, data_size: usize, seq_num: SequenceNumber) -> Option<Vec<u8>> {
        let mut data = Vec::new();
        if let Some(m) = self.frag_buffer.get(&seq_num) {
            for fragment_number in 1..m.len() as u32 {
                if let Some(mut data_frag) = m.get(&FragmentNumber::new(fragment_number)).cloned() {
                    data.append(&mut data_frag);
                } else {
                    break;
                }
            }
        }

        if data.len() >= data_size {
            Some(data)
        } else {
            None
        }
    }

    pub fn remote_writer_guid(&self) -> Guid {
        self.remote_writer_guid
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.as_ref()
    }

    pub fn available_changes_max(&self) -> SequenceNumber {
        // The condition to make any CacheChange ‘a_change’ available for ‘access’ by the DDS DataReader is that there are no changes
        // from the RTPS Writer with SequenceNumber_t smaller than or equal to a_change.sequenceNumber that have status MISSING or UNKNOWN.

        // Any number below first_available_seq_num is missing so that is the minimum
        // If there are missing changes, the minimum will be one above the maximum
        if let Some(&minimum_missing_changes) = self.missing_changes().iter().min() {
            minimum_missing_changes - 1
        } else {
            // If there are no missing changes then the highest received sequence number
            // with a lower limit of the first_available_seq_num
            let minimum_available_changes_max =
                min(self.first_available_seq_num, self.last_available_seq_num);
            let highest_received_seq_num = *self
                .received_changes
                .iter()
                .filter(|&x| !self.irrelevant_changes.contains(x))
                .max()
                .unwrap_or(&SequenceNumber::new(0));
            max(highest_received_seq_num, minimum_available_changes_max)
        }
    }

    pub fn _irrelevant_change_set(&mut self, a_seq_num: SequenceNumber) {
        // This operation modifies the status of a ChangeFromWriter to indicate that the CacheChange with the
        // SequenceNumber_t ‘a_seq_num’ is irrelevant to the RTPS Reader. Logical action in the virtual machine:
        // FIND change FROM this.changes_from_writer SUCH-THAT
        // (change.sequenceNumber == a_seq_num);
        // change.status := RECEIVED; change.is_relevant := FALSE;
        self.irrelevant_changes.push(a_seq_num);
    }

    pub fn lost_changes_update(&mut self, first_available_seq_num: SequenceNumber) {
        // FOREACH change IN this.changes_from_writer
        // SUCH-THAT ( change.status == UNKNOWN OR change.status == MISSING
        // AND seq_num < first_available_seq_num ) DO {
        // change.status := LOST;
        // }
        self.first_available_seq_num = first_available_seq_num;
    }

    pub fn missing_changes(&self) -> Vec<SequenceNumber> {
        // The changes with status ‘MISSING’ represent the set of changes available in the HistoryCache of the RTPS Writer represented by the RTPS WriterProxy that have not been received by the RTPS Reader.
        // return { change IN this.changes_from_writer SUCH-THAT change.status == MISSING};
        let mut missing_changes = Vec::new();

        let highest_received_seq_num = self
            .received_changes
            .iter()
            .max()
            .cloned()
            .unwrap_or(SequenceNumber::new(0));
        let highest_irrelevant_seq_num = self
            .irrelevant_changes
            .iter()
            .max()
            .cloned()
            .unwrap_or(SequenceNumber::new(0));
        // The highest sequence number of all present
        let highest_number = max(
            self.last_available_seq_num,
            max(highest_received_seq_num, highest_irrelevant_seq_num),
        );
        // Changes below first_available_seq_num are LOST (or RECEIVED, but in any case not MISSING) and above last_available_seq_num are unknown.
        // In between those two numbers, every change that is not RECEIVED or IRRELEVANT is MISSING
        let mut seq_num = self.first_available_seq_num;
        while seq_num <= highest_number {
            let received = self.received_changes.contains(&seq_num);
            let irrelevant = self.irrelevant_changes.contains(&seq_num);
            if !(irrelevant || received) {
                missing_changes.push(seq_num)
            }
            seq_num += 1;
        }
        missing_changes
    }

    pub fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber) {
        // FOREACH change IN this.changes_from_writer
        // SUCH-THAT ( change.status == UNKNOWN
        // AND seq_num <= last_available_seq_num ) DO {
        // change.status := MISSING;
        // }
        self.last_available_seq_num = last_available_seq_num;
    }

    pub fn received_change_set(&mut self, a_seq_num: SequenceNumber) {
        // FIND change FROM this.changes_from_writer
        //     SUCH-THAT change.sequenceNumber == a_seq_num;
        // change.status := RECEIVED
        self.received_changes.push(a_seq_num);
    }

    pub fn set_must_send_acknacks(&mut self, must_send_acknacks: bool) {
        self.must_send_acknacks = must_send_acknacks;
    }

    pub fn must_send_acknacks(&self) -> bool {
        self.must_send_acknacks
    }

    pub fn last_received_heartbeat_count(&self) -> Count {
        self.last_received_heartbeat_count
    }

    pub fn set_last_received_heartbeat_count(&mut self, last_received_heartbeat_count: Count) {
        self.last_received_heartbeat_count = last_received_heartbeat_count;
    }

    pub fn acknack_count(&self) -> Count {
        self.acknack_count
    }

    pub fn increment_acknack_count(&mut self) {
        self.acknack_count = self.acknack_count.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::rtps::types::{ENTITYID_UNKNOWN, GUID_UNKNOWN};

    use super::*;

    fn create_test_proxy() -> RtpsWriterProxy {
        RtpsWriterProxy::new(GUID_UNKNOWN, &[], &[], None, ENTITYID_UNKNOWN)
    }

    #[test]
    fn writer_proxy_available_changes_max_empty() {
        let writer_proxy = create_test_proxy();

        assert_eq!(writer_proxy.available_changes_max(), SequenceNumber::new(0));
    }

    #[test]
    fn writer_proxy_available_changes_max_sequential_received_changes() {
        let mut writer_proxy = create_test_proxy();

        writer_proxy.received_change_set(SequenceNumber::new(1));
        writer_proxy.received_change_set(SequenceNumber::new(2));

        assert_eq!(writer_proxy.available_changes_max(), SequenceNumber::new(2));
    }

    #[test]
    fn writer_proxy_available_changes_max_missing_received_changes() {
        let mut writer_proxy = create_test_proxy();

        writer_proxy.received_change_set(SequenceNumber::new(1));
        writer_proxy.received_change_set(SequenceNumber::new(2));
        writer_proxy.received_change_set(SequenceNumber::new(4));

        assert_eq!(writer_proxy.available_changes_max(), SequenceNumber::new(2));
    }

    #[test]
    fn writer_proxy_missing_changes_without_lost_changes() {
        let mut writer_proxy = create_test_proxy();

        writer_proxy.missing_changes_update(SequenceNumber::new(3));

        let expected_missing_changes = vec![
            SequenceNumber::new(1),
            SequenceNumber::new(2),
            SequenceNumber::new(3),
        ];
        let missing_changes = writer_proxy.missing_changes();
        assert_eq!(missing_changes, expected_missing_changes);
    }

    #[test]
    fn writer_proxy_missing_changes_with_lost_changes() {
        let mut writer_proxy = create_test_proxy();

        writer_proxy.lost_changes_update(SequenceNumber::new(2));
        writer_proxy.missing_changes_update(SequenceNumber::new(3));

        let expected_missing_changes = vec![SequenceNumber::new(2), SequenceNumber::new(3)];
        let missing_changes = writer_proxy.missing_changes();
        assert_eq!(missing_changes, expected_missing_changes);
    }

    #[test]
    fn writer_proxy_missing_changes_with_received_changes() {
        let mut writer_proxy = create_test_proxy();

        writer_proxy.missing_changes_update(SequenceNumber::new(3));
        writer_proxy.received_change_set(SequenceNumber::new(2));

        let expected_missing_changes = vec![SequenceNumber::new(1), SequenceNumber::new(3)];
        let missing_changes = writer_proxy.missing_changes();
        assert_eq!(missing_changes, expected_missing_changes);
    }

    #[test]
    fn writer_proxy_missing_changes_with_only_received_changes() {
        let mut writer_proxy = create_test_proxy();

        writer_proxy.received_change_set(SequenceNumber::new(1));
        writer_proxy.received_change_set(SequenceNumber::new(2));
        writer_proxy.received_change_set(SequenceNumber::new(4));

        let expected_missing_changes = vec![SequenceNumber::new(3)];
        let missing_changes = writer_proxy.missing_changes();
        assert_eq!(missing_changes, expected_missing_changes);
    }
}
