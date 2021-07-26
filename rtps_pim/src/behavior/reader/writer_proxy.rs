use crate::structure::types::{EntityId, Locator, SequenceNumber, GUID};

pub trait RTPSWriterProxy {
    fn remote_writer_guid(&self) -> &GUID;
    fn unicast_locator_list(&self) -> &[Locator];
    fn multicast_locator_list(&self) -> &[Locator];
    fn data_max_size_serialized(&self) -> &Option<i32>;
    fn remote_group_entity_id(&self) -> &EntityId;
}
pub trait RTPSWriterProxyOperations {
    type SequenceNumberVector;
    fn new<L>(
        remote_writer_guid: GUID,
        remote_group_entity_id: EntityId,
        unicast_locator_list: L,
        multicast_locator_list: L,
        data_max_size_serialized: Option<i32>,
    ) -> Self
    where
        L: IntoIterator<Item = Locator>;
    fn available_changes_max(&self) -> &SequenceNumber;
    fn irrelevant_change_set(&mut self, a_seq_num: &SequenceNumber);
    fn lost_changes_update(&mut self, first_available_seq_num: &SequenceNumber);
    fn missing_changes(&self) -> Self::SequenceNumberVector;
    fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber);
    fn received_change_set(&mut self, a_seq_num: SequenceNumber);
}
