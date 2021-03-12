use crate::{
    behavior::types::ChangeForReaderStatusKind,
    types::{EntityId, Locator, SequenceNumber, GUID},
};

pub trait RTPSChangeForReader {
    type CacheChangeRepresentation;

    fn change(&self) -> &Self::CacheChangeRepresentation;
    fn status(&self) -> ChangeForReaderStatusKind;
    fn is_relevant(&self) -> bool;
}

pub trait RTPSReaderProxy {
    type ChangeForReaderType: RTPSChangeForReader;

    fn remote_reader_guid(&self) -> GUID;
    fn remote_group_entity_id(&self) -> EntityId;
    fn unicast_locator_list(&self) -> &[Locator];
    fn multicast_locator_list(&self) -> &[Locator];
    fn changes_for_reader(
        &self,
    ) -> &[<Self::ChangeForReaderType as RTPSChangeForReader>::CacheChangeRepresentation];
    fn expects_inline_qos(&self) -> bool;
    fn is_active(&self) -> bool;

    fn new(
        remote_reader_guid: GUID,
        remote_group_entity_id: EntityId,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        expects_inline_qos: bool,
        is_active: bool,
    ) -> Self;

    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber);
    fn next_requested_change(&mut self) -> Option<&Self::ChangeForReaderType>;
    fn next_unsent_change(&mut self) -> Option<&Self::ChangeForReaderType>;
    fn unsent_changes(&self) -> &[Self::ChangeForReaderType];
    fn requested_changes(&self) -> &[Self::ChangeForReaderType];
    fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]);
    fn unacked_changes(&self) -> &[Self::ChangeForReaderType];
}
