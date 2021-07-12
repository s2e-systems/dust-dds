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
    fn available_changes_max(&self) -> &SequenceNumber;
    fn irrelevant_change_set(&mut self, a_seq_num: &SequenceNumber);
    fn lost_changes_update(&mut self, first_available_seq_num: &SequenceNumber);
    fn missing_changes(&self) -> Self::SequenceNumberVector;
    fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber);
    fn received_change_set(&mut self, a_seq_num: SequenceNumber);
}
