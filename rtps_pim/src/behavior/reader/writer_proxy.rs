use crate::structure::types::{EntityId, Guid, SequenceNumber};

pub struct RtpsWriterProxy<L> {
    pub remote_writer_guid: Guid,
    pub unicast_locator_list: L,
    pub multicast_locator_list: L,
    pub data_max_size_serialized: Option<i32>,
    pub remote_group_entity_id: EntityId,
}

pub trait RtpsWriterProxyOperations {
    type SequenceNumberVector;

    fn available_changes_max(&self) -> &SequenceNumber;
    fn irrelevant_change_set(&mut self, a_seq_num: &SequenceNumber);
    fn lost_changes_update(&mut self, first_available_seq_num: &SequenceNumber);
    fn missing_changes(&self) -> Self::SequenceNumberVector;
    fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber);
    fn received_change_set(&mut self, a_seq_num: SequenceNumber);
}
