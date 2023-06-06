use std::{
    cmp::{max, min},
    collections::HashMap,
};

use super::{
    messages::{
        overall_structure::{RtpsMessageHeader, RtpsMessageWrite, RtpsSubmessageWriteKind},
        submessage_elements::{Data, FragmentNumberSet, SequenceNumberSet},
        submessages::{
            ack_nack::AckNackSubmessageWrite, data_frag::DataFragSubmessageRead,
            info_destination::InfoDestinationSubmessageWrite, nack_frag::NackFragSubmessageWrite,
        },
        types::{Count, FragmentNumber},
    },
    transport::TransportWrite,
    types::{EntityId, Guid, Locator, SequenceNumber},
};

#[derive(Debug, PartialEq, Eq)]
pub struct OwningDataFragSubmessage {
    fragment_starting_num: FragmentNumber,
    data_size: u32,
    fragment_size: u16,
    fragments_in_submessage: u16,
    serialized_payload: Data,
}

impl From<&DataFragSubmessageRead<'_>> for OwningDataFragSubmessage {
    fn from(x: &DataFragSubmessageRead<'_>) -> Self {
        Self {
            fragment_starting_num: x.fragment_starting_num(),
            data_size: x.data_size(),
            fragment_size: x.fragment_size(),
            fragments_in_submessage: x.fragments_in_submessage(),
            serialized_payload: x.serialized_payload(),
        }
    }
}

fn total_fragments_expected(data_frag_submessage: &OwningDataFragSubmessage) -> u32 {
    let data_size = data_frag_submessage.data_size;
    let fragment_size = data_frag_submessage.fragment_size as u32;
    let total_fragments_correction = if data_size % fragment_size == 0 { 0 } else { 1 };
    data_size / fragment_size + total_fragments_correction
}

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
    last_received_heartbeat_frag_count: Count,
    acknack_count: Count,
    nack_frag_count: Count,
    frag_buffer: HashMap<SequenceNumber, Vec<OwningDataFragSubmessage>>,
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
            last_received_heartbeat_frag_count: Count::new(0),
            acknack_count: Count::new(0),
            nack_frag_count: Count::new(0),
            frag_buffer: HashMap::new(),
        }
    }

    pub fn push_data_frag(&mut self, submessage: &DataFragSubmessageRead) {
        let owning_data_frag = submessage.into();
        let frag_bug_seq_num = self.frag_buffer.entry(submessage.writer_sn()).or_default();
        if !frag_bug_seq_num.contains(&owning_data_frag) {
            frag_bug_seq_num.push(owning_data_frag);
        }
    }

    pub fn extract_frag(&mut self, seq_num: SequenceNumber) -> Option<Vec<u8>> {
        if let Some(seq_num_frag) = self.frag_buffer.get(&seq_num) {
            let total_fragments_expected = total_fragments_expected(&seq_num_frag[0]);

            let mut total_fragments = 0;
            for frag_seq_num in seq_num_frag {
                total_fragments += frag_seq_num.fragments_in_submessage as u32;
            }

            if total_fragments == total_fragments_expected {
                let mut frag_seq_num_list = self.frag_buffer.remove(&seq_num).expect("Must exist");
                frag_seq_num_list.sort_by_key(|k| k.fragment_starting_num);

                let mut data = Vec::new();
                for frag in frag_seq_num_list {
                    data.append(&mut frag.serialized_payload.as_ref().to_vec());
                }
                return Some(data);
            }
        }
        None
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

    pub fn irrelevant_change_set(&mut self, a_seq_num: SequenceNumber) {
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

    pub fn _last_received_heartbeat_frag_count(&self) -> Count {
        self.last_received_heartbeat_frag_count
    }

    pub fn set_last_received_heartbeat_count(&mut self, last_received_heartbeat_count: Count) {
        self.last_received_heartbeat_count = last_received_heartbeat_count;
    }

    pub fn set_last_received_heartbeat_frag_count(
        &mut self,
        last_received_heartbeat_frag_count: Count,
    ) {
        self.last_received_heartbeat_frag_count = last_received_heartbeat_frag_count;
    }

    pub fn acknack_count(&self) -> Count {
        self.acknack_count
    }

    pub fn increment_acknack_count(&mut self) {
        self.acknack_count = self.acknack_count.wrapping_add(1);
    }

    pub fn send_message(
        &mut self,
        reader_guid: &Guid,
        header: RtpsMessageHeader,
        transport: &mut impl TransportWrite,
    ) {
        if self.must_send_acknacks() || !self.missing_changes().is_empty() {
            self.set_must_send_acknacks(false);
            self.increment_acknack_count();

            let info_dst_submessage =
                InfoDestinationSubmessageWrite::new(self.remote_writer_guid().prefix());

            let acknack_submessage = AckNackSubmessageWrite::new(
                true,
                reader_guid.entity_id(),
                self.remote_writer_guid().entity_id(),
                SequenceNumberSet {
                    base: self.available_changes_max() + 1,
                    set: self.missing_changes(),
                },
                self.acknack_count(),
            );

            let mut submessages = vec![
                RtpsSubmessageWriteKind::InfoDestination(info_dst_submessage),
                RtpsSubmessageWriteKind::AckNack(acknack_submessage),
            ];

            for (seq_num, owning_data_frag_list) in self.frag_buffer.iter() {
                let total_fragments_expected = total_fragments_expected(&owning_data_frag_list[0]);
                let mut missing_fragment_number = Vec::new();
                for fragment_number in 1..=total_fragments_expected {
                    if !owning_data_frag_list.iter().any(|x| {
                        fragment_number >= u32::from(x.fragment_starting_num)
                            && fragment_number
                                < u32::from(x.fragment_starting_num)
                                    + (x.fragments_in_submessage as u32)
                    }) {
                        missing_fragment_number.push(FragmentNumber::new(fragment_number))
                    }
                }

                if !missing_fragment_number.is_empty() {
                    self.nack_frag_count = self.nack_frag_count.wrapping_add(1);
                    let nack_frag_submessage =
                        RtpsSubmessageWriteKind::NackFrag(NackFragSubmessageWrite::new(
                            reader_guid.entity_id(),
                            self.remote_writer_guid().entity_id(),
                            *seq_num,
                            FragmentNumberSet {
                                base: missing_fragment_number[0],
                                set: missing_fragment_number,
                            },
                            self.nack_frag_count,
                        ));

                    submessages.push(nack_frag_submessage)
                }
            }

            transport.write(
                &RtpsMessageWrite::new(header, submessages),
                self.unicast_locator_list(),
            );
        }
    }

    pub fn is_historical_data_received(&self) -> bool {
        let at_least_one_heartbeat_received = self.last_received_heartbeat_count > Count::new(0);
        at_least_one_heartbeat_received && self.missing_changes().is_empty()
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
