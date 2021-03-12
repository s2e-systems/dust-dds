use rust_rtps::{
    behavior::{
        stateful_writer::reader_proxy::RTPSChangeForReader, types::ChangeForReaderStatusKind,
        RTPSReaderProxy,
    },
    types::SequenceNumber,
};

pub struct ChangeForReader;

impl RTPSChangeForReader for ChangeForReader {
    type CacheChangeRepresentation = SequenceNumber;

    fn change(&self) -> &Self::CacheChangeRepresentation {
        todo!()
    }

    fn status(&self) -> ChangeForReaderStatusKind {
        todo!()
    }

    fn is_relevant(&self) -> bool {
        todo!()
    }
}

pub struct ReaderProxy {}

impl RTPSReaderProxy for ReaderProxy {
    type ChangeForReaderType = ChangeForReader;

    fn remote_reader_guid(&self) -> rust_rtps::types::GUID {
        todo!()
    }

    fn remote_group_entity_id(&self) -> rust_rtps::types::EntityId {
        todo!()
    }

    fn unicast_locator_list(&self) -> &[rust_rtps::types::Locator] {
        todo!()
    }

    fn multicast_locator_list(&self) -> &[rust_rtps::types::Locator] {
        todo!()
    }

    fn changes_for_reader(
        &self,
    ) -> &[<Self::ChangeForReaderType as RTPSChangeForReader>::CacheChangeRepresentation] {
        todo!()
    }

    fn expects_inline_qos(&self) -> bool {
        todo!()
    }

    fn is_active(&self) -> bool {
        todo!()
    }

    fn new(
        _remote_reader_guid: rust_rtps::types::GUID,
        _unicast_locator_list: &[rust_rtps::types::Locator],
        _multicast_locator_list: &[rust_rtps::types::Locator],
        _expects_inline_qos: bool,
        _is_active: bool,
    ) -> Self {
        todo!()
    }

    fn acked_changes_set(&mut self, _committed_seq_num: rust_rtps::types::SequenceNumber) {
        todo!()
    }

    fn next_requested_change(&mut self) -> Option<&Self::ChangeForReaderType> {
        todo!()
    }

    fn next_unsent_change(&mut self) -> Option<&Self::ChangeForReaderType> {
        todo!()
    }

    fn unsent_changes(&self) -> &[Self::ChangeForReaderType] {
        todo!()
    }

    fn requested_changes(&self) -> &[Self::ChangeForReaderType] {
        todo!()
    }

    fn requested_changes_set(&mut self, _req_seq_num_set: &[rust_rtps::types::SequenceNumber]) {
        todo!()
    }

    fn unacked_changes(&self) -> &[Self::ChangeForReaderType] {
        todo!()
    }
}
