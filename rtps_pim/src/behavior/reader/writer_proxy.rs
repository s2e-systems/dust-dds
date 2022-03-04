use crate::structure::types::{EntityId, Guid, Locator, SequenceNumber};

pub trait RtpsWriterProxyConstructor {
    fn new(
        remote_writer_guid: Guid,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        data_max_size_serialized: Option<i32>,
        remote_group_entity_id: EntityId,
    ) -> Self;
}

pub trait RtpsWriterProxyAttributes {
    fn remote_writer_guid(&self) -> Guid;
    fn unicast_locator_list(&self) -> &[Locator];
    fn multicast_locator_list(&self) -> &[Locator];
    fn data_max_size_serialized(&self) -> Option<i32>;
    fn remote_group_entity_id(&self) -> EntityId;
}

pub trait RtpsWriterProxyOperations {
    type SequenceNumberListType;

    fn available_changes_max(&self) -> SequenceNumber;
    fn irrelevant_change_set(&mut self, a_seq_num: SequenceNumber);
    fn lost_changes_update(&mut self, first_available_seq_num: SequenceNumber);
    fn missing_changes(&self) -> Self::SequenceNumberListType;
    fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber);
    fn received_change_set(&mut self, a_seq_num: SequenceNumber);
}
