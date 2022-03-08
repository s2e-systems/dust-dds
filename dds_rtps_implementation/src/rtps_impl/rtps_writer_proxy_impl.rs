use rust_rtps_pim::{
    behavior::reader::writer_proxy::{
        RtpsWriterProxyAttributes, RtpsWriterProxyConstructor, RtpsWriterProxyOperations,
    },
    structure::types::{EntityId, Guid, Locator, SequenceNumber},
};

use super::rtps_history_cache_impl::RtpsHistoryCacheImpl;

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

    // Changes for which a GAP message is received are irrelevant
    irrelevant_changes: Vec<SequenceNumber>,

    // Changes which have been received
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

pub struct RtpsWriterProxyOperationsImpl<'a> {
    pub writer_proxy: &'a mut RtpsWriterProxyImpl,
    pub reader_cache: &'a RtpsHistoryCacheImpl,
}

impl RtpsWriterProxyOperations for RtpsWriterProxyOperationsImpl<'_> {
    type SequenceNumberListType = Vec<SequenceNumber>;

    fn available_changes_max(&self) -> SequenceNumber {
        // seq_num := MAX { change.sequenceNumber SUCH-THAT
        // ( change IN this.changes_from_writer
        // AND ( change.status == RECEIVED
        // OR change.status == LOST) ) };
        // return seq_num;

        // Changes below first_available_seq_num are LOST (or RECEIVED, but in any case not MISSING).
        // Change that are IRRELEVANT are not considered
        let max_received_change_seq_num = *self
            .writer_proxy
            .received_changes
            .iter()
            .filter(|&x| !self.writer_proxy.irrelevant_changes.contains(x))
            .max()
            .unwrap_or(&0);
        i64::max(
            self.writer_proxy.first_available_seq_num,
            max_received_change_seq_num,
        )
    }

    fn irrelevant_change_set(&mut self, a_seq_num: SequenceNumber) {
        // This operation modifies the status of a ChangeFromWriter to indicate that the CacheChange with the
        // SequenceNumber_t ‘a_seq_num’ is irrelevant to the RTPS Reader. Logical action in the virtual machine:
        // FIND change FROM this.changes_from_writer SUCH-THAT
        // (change.sequenceNumber == a_seq_num);
        // change.status := RECEIVED; change.is_relevant := FALSE;
        self.writer_proxy.irrelevant_changes.push(a_seq_num)
    }

    fn lost_changes_update(&mut self, first_available_seq_num: SequenceNumber) {
        // FOREACH change IN this.changes_from_writer
        // SUCH-THAT ( change.status == UNKNOWN OR change.status == MISSING
        // AND seq_num < first_available_seq_num ) DO {
        // change.status := LOST;
        // }
        self.writer_proxy.first_available_seq_num = first_available_seq_num
    }

    fn missing_changes(&self) -> Self::SequenceNumberListType {
        // The changes with status ‘MISSING’ represent the set of changes available in the HistoryCache of the RTPS Writer represented by the RTPS WriterProxy that have not been received by the RTPS Reader.
        // return { change IN this.changes_from_writer SUCH-THAT change.status == MISSING};
        let mut missing_changes = Vec::new();

        // Changes below first_available_seq_num are LOST (or RECEIVED, but in any case not MISSING) and above last_available_seq_num are unknown.
        // In between those two numbers, every change that is not RECEIVED or IRRELEVANT is MISSING
        for seq_num in
            self.writer_proxy.first_available_seq_num..=self.writer_proxy.last_available_seq_num
        {
            let received = self.writer_proxy.received_changes.contains(&seq_num);
            let irrelevant = self.writer_proxy.irrelevant_changes.contains(&seq_num);
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
        self.writer_proxy.last_available_seq_num = last_available_seq_num;
    }

    fn received_change_set(&mut self, a_seq_num: SequenceNumber) {
        // FIND change FROM this.changes_from_writer
        //     SUCH-THAT change.sequenceNumber == a_seq_num;
        // change.status := RECEIVED
        self.writer_proxy.received_changes.push(a_seq_num);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writer_proxy_available_changes_max() {
        todo!()
    }
}
