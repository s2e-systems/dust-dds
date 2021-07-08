use crate::structure::types::{EntityId, Locator, SequenceNumber, GUID};

pub trait RTPSReaderProxy {
    type SequenceNumberVector;

    fn remote_reader_guid(&self) -> &GUID;
    fn remote_group_entity_id(&self) -> &EntityId;
    fn unicast_locator_list(&self) -> &[Locator];
    fn multicast_locator_list(&self) -> &[Locator];
    fn expects_inline_qos(&self) -> bool;
    fn is_active(&self) -> bool;

    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber);
    fn next_requested_change(&mut self) -> Option<SequenceNumber>;
    fn next_unsent_change(&mut self) -> Option<SequenceNumber>;
    fn unsent_changes(&self) -> Self::SequenceNumberVector;
    fn requested_changes(&self) -> Self::SequenceNumberVector;
    fn requested_changes_set(&mut self, req_seq_num_set: Self::SequenceNumberVector);
    fn unacked_changes(&self) -> Self::SequenceNumberVector;
}

pub trait RTPSStatefulWriter {
    type ReaderProxyType;

    fn matched_readers(&self) -> &[Self::ReaderProxyType];

    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType);

    fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID);

    fn matched_reader_lookup(&self, a_reader_guid: &GUID) -> Option<&Self::ReaderProxyType>;

    fn is_acked_by_all(&self) -> bool;
}
