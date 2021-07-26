use crate::structure::types::{EntityId, Locator, SequenceNumber, GUID};

pub trait RTPSReaderProxy {
    fn remote_reader_guid(&self) -> &GUID;
    fn remote_group_entity_id(&self) -> &EntityId;
    fn unicast_locator_list(&self) -> &[Locator];
    fn multicast_locator_list(&self) -> &[Locator];
    fn expects_inline_qos(&self) -> bool;
    fn is_active(&self) -> bool;
}

pub trait RTPSReaderProxyOperations {
    type SequenceNumberVector;

    fn new<L>(
        remote_reader_guid: GUID,
        remote_group_entity_id: EntityId,
        unicast_locator_list: L,
        multicast_locator_list: L,
        expects_inline_qos: bool,
        is_active: bool,
    ) -> Self
    where
        L: IntoIterator<Item = Locator>;
    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber);
    fn next_requested_change(&mut self) -> Option<SequenceNumber>;
    fn next_unsent_change(&mut self) -> Option<SequenceNumber>;
    fn unsent_changes(&self) -> Self::SequenceNumberVector;
    fn requested_changes(&self) -> Self::SequenceNumberVector;
    fn requested_changes_set(&mut self, req_seq_num_set: Self::SequenceNumberVector);
    fn unacked_changes(&self) -> Self::SequenceNumberVector;
}
