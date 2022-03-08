use rust_rtps_pim::{
    behavior::reader::writer_proxy::{
        RtpsWriterProxyAttributes, RtpsWriterProxyConstructor, RtpsWriterProxyOperations,
    },
    structure::types::{EntityId, Guid, Locator, SequenceNumber},
};

#[derive(Debug, PartialEq)]
pub struct RtpsWriterProxyImpl {
    remote_writer_guid: Guid,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    data_max_size_serialized: Option<i32>,
    remote_group_entity_id: EntityId,

    // Every change below the first_available_seq num is LOST (i.e. no longer available in the RTPS Writer)
    // Every change above the last_available_seq_num is UNKNOWN (i.e. may or may not be available yet at the RTPS Writer.)
    first_available_seq_num: SequenceNumber,
    last_available_seq_num: SequenceNumber,

    // Changes which are IRRELEVANT
    irrelevant_changes: Vec<SequenceNumber>,

    // Changes which have are RECEIVED
    received_changes: Vec<SequenceNumber>,
}

impl RtpsWriterProxyConstructor for RtpsWriterProxyImpl {
    fn new(
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
            first_available_seq_num: 0,
            last_available_seq_num: 0,
            irrelevant_changes: Vec::new(),
            received_changes: Vec::new(),
        }
    }
}

impl RtpsWriterProxyAttributes for RtpsWriterProxyImpl {
    fn remote_writer_guid(&self) -> Guid {
        self.remote_writer_guid
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.as_ref()
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        self.multicast_locator_list.as_ref()
    }

    fn data_max_size_serialized(&self) -> Option<i32> {
        self.data_max_size_serialized
    }

    fn remote_group_entity_id(&self) -> EntityId {
        self.remote_group_entity_id
    }
}

impl RtpsWriterProxyOperations for RtpsWriterProxyImpl {
    type SequenceNumberListType = Vec<SequenceNumber>;

    fn available_changes_max(&self) -> SequenceNumber {
        // The condition to make any CacheChange ‘a_change’ available for ‘access’ by the DDS DataReader is that there are no changes
        // from the RTPS Writer with SequenceNumber_t smaller than or equal to a_change.sequenceNumber that have status MISSING or UNKNOWN.

        let max_received_change_seq_num = *self
            .received_changes
            .iter()
            .filter(|&x| !self.irrelevant_changes.contains(x))
            .max()
            .unwrap_or(&0);
        i64::max(self.first_available_seq_num, max_received_change_seq_num)
    }

    fn irrelevant_change_set(&mut self, a_seq_num: SequenceNumber) {
        // This operation modifies the status of a ChangeFromWriter to indicate that the CacheChange with the
        // SequenceNumber_t ‘a_seq_num’ is irrelevant to the RTPS Reader. Logical action in the virtual machine:
        // FIND change FROM this.changes_from_writer SUCH-THAT
        // (change.sequenceNumber == a_seq_num);
        // change.status := RECEIVED; change.is_relevant := FALSE;
        self.irrelevant_changes.push(a_seq_num)
    }

    fn lost_changes_update(&mut self, first_available_seq_num: SequenceNumber) {
        // FOREACH change IN this.changes_from_writer
        // SUCH-THAT ( change.status == UNKNOWN OR change.status == MISSING
        // AND seq_num < first_available_seq_num ) DO {
        // change.status := LOST;
        // }
        self.first_available_seq_num = first_available_seq_num
    }

    fn missing_changes(&self) -> Self::SequenceNumberListType {
        // The changes with status ‘MISSING’ represent the set of changes available in the HistoryCache of the RTPS Writer represented by the RTPS WriterProxy that have not been received by the RTPS Reader.
        // return { change IN this.changes_from_writer SUCH-THAT change.status == MISSING};
        let mut missing_changes = Vec::new();

        // Changes below first_available_seq_num are LOST (or RECEIVED, but in any case not MISSING) and above last_available_seq_num are unknown.
        // In between those two numbers, every change that is not RECEIVED or IRRELEVANT is MISSING
        for seq_num in self.first_available_seq_num + 1..=self.last_available_seq_num {
            let received = self.received_changes.contains(&seq_num);
            let irrelevant = self.irrelevant_changes.contains(&seq_num);
            if !(irrelevant || received) {
                missing_changes.push(seq_num)
            }
        }
        missing_changes
    }

    fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber) {
        // FOREACH change IN this.changes_from_writer
        // SUCH-THAT ( change.status == UNKNOWN
        // AND seq_num <= last_available_seq_num ) DO {
        // change.status := MISSING;
        // }
        self.last_available_seq_num = last_available_seq_num;
    }

    fn received_change_set(&mut self, a_seq_num: SequenceNumber) {
        // FIND change FROM this.changes_from_writer
        //     SUCH-THAT change.sequenceNumber == a_seq_num;
        // change.status := RECEIVED
        self.received_changes.push(a_seq_num);
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::types::{ENTITYID_UNKNOWN, GUID_UNKNOWN};

    use super::*;

    fn create_test_proxy() -> RtpsWriterProxyImpl {
        RtpsWriterProxyImpl::new(GUID_UNKNOWN, &[], &[], None, ENTITYID_UNKNOWN)
    }

    #[test]
    fn writer_proxy_available_changes_max_empty() {
        let writer_proxy = create_test_proxy();

        assert_eq!(writer_proxy.available_changes_max(), 0);
    }

    #[test]
    fn writer_proxy_available_changes_max_sequential_received_changes() {
        let mut writer_proxy = create_test_proxy();

        writer_proxy.received_change_set(1);
        writer_proxy.received_change_set(2);

        assert_eq!(writer_proxy.available_changes_max(), 2);
    }

    #[test]
    fn writer_proxy_available_changes_max_missing_received_changes() {
        let mut writer_proxy = create_test_proxy();

        writer_proxy.received_change_set(1);
        writer_proxy.received_change_set(2);
        writer_proxy.received_change_set(4);

        assert_eq!(writer_proxy.available_changes_max(), 2);
    }

    #[test]
    fn writer_proxy_missing_changes_without_lost_changes() {
        let mut writer_proxy = create_test_proxy();

        writer_proxy.missing_changes_update(3);

        let expected_missing_changes = vec![1, 2, 3];
        let missing_changes = writer_proxy.missing_changes();
        assert_eq!(missing_changes, expected_missing_changes);
    }

    #[test]
    fn writer_proxy_missing_changes_with_lost_changes() {
        let mut writer_proxy = create_test_proxy();

        writer_proxy.lost_changes_update(2);
        writer_proxy.missing_changes_update(3);

        let expected_missing_changes = vec![3];
        let missing_changes = writer_proxy.missing_changes();
        assert_eq!(missing_changes, expected_missing_changes);
    }

    #[test]
    fn writer_proxy_missing_changes_with_received_changes() {
        let mut writer_proxy = create_test_proxy();

        writer_proxy.missing_changes_update(3);
        writer_proxy.received_change_set(2);

        let expected_missing_changes = vec![1, 3];
        let missing_changes = writer_proxy.missing_changes();
        assert_eq!(missing_changes, expected_missing_changes);
    }
}
