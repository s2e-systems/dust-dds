use crate::{
    behavior::{types::ChangeForReaderStatusKind, RTPSWriter},
    structure::{entity::RTPSGUID, RTPSCacheChange, RTPSEndpoint, RTPSEntity, RTPSHistoryCache},
};

pub trait RTPSChangeForReader {
    type CacheChangeRepresentation;
    fn new(
        change: Self::CacheChangeRepresentation,
        status: ChangeForReaderStatusKind,
        is_relevant: bool,
    ) -> Self;
    fn change(&self) -> Self::CacheChangeRepresentation;
    fn status(&self) -> ChangeForReaderStatusKind;
    fn is_relevant(&self) -> bool;
}

pub trait RTPSReaderProxy {
    type ChangeForReaderType: RTPSChangeForReader;
    type ChangeForReaderTypeList: IntoIterator<Item = Self::ChangeForReaderType>;
    type Writer: RTPSWriter;

    // Attributes
    fn remote_reader_guid(&self) -> <Self::Writer as RTPSEntity>::GUID;
    fn remote_group_entity_id(&self) -> <<Self::Writer as RTPSEntity>::GUID as RTPSGUID>::EntityId;
    fn unicast_locator_list(&self) -> &[<Self::Writer as RTPSEndpoint>::Locator];
    fn multicast_locator_list(&self) -> &[<Self::Writer as RTPSEndpoint>::Locator];
    fn expects_inline_qos(&self) -> bool;
    fn is_active(&self) -> bool;

    // Operations
    fn new(
        remote_reader_guid: <Self::Writer as RTPSEntity>::GUID,
        remote_group_entity_id: <<Self::Writer as RTPSEntity>::GUID as RTPSGUID>::EntityId,
        unicast_locator_list: &[<Self::Writer as RTPSEndpoint>::Locator],
        multicast_locator_list: &[<Self::Writer as RTPSEndpoint>::Locator],
        expects_inline_qos: bool,
        is_active: bool,
    ) -> Self;

    fn acked_changes_set(
        &mut self,
        committed_seq_num: <<<Self::Writer as RTPSWriter>::HistoryCache as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::SequenceNumber,
        writer: &Self::Writer,
    );
    fn next_requested_change(&mut self, writer: &Self::Writer)
        -> Option<Self::ChangeForReaderType>;
    fn next_unsent_change(&mut self, writer: &Self::Writer) -> Option<Self::ChangeForReaderType>;
    fn unsent_changes(&self, writer: &Self::Writer) -> Self::ChangeForReaderTypeList;
    fn requested_changes(&self, writer: &Self::Writer) -> Self::ChangeForReaderTypeList;
    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[<<<Self::Writer as RTPSWriter>::HistoryCache as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::SequenceNumber],
        writer: &Self::Writer,
    );
    fn unacked_changes(&self, writer: &Self::Writer) -> Self::ChangeForReaderTypeList;
    fn changes_for_reader(&self, writer: &Self::Writer) -> Self::ChangeForReaderTypeList;
}
