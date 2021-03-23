use crate::{
    behavior::RTPSWriter,
    structure::{RTPSCacheChange, RTPSHistoryCache},
};

pub trait RTPSReaderLocator {
    type CacheChangeRepresentation;
    type CacheChangeRepresentationList: IntoIterator<Item = Self::CacheChangeRepresentation>;
    type Writer: RTPSWriter;

    fn requested_changes(&self) -> Self::CacheChangeRepresentationList;
    fn unsent_changes(&self) -> Self::CacheChangeRepresentationList;
    fn next_requested_change(
        &mut self
    ) -> Option<Self::CacheChangeRepresentation>;
    fn next_unsent_change(
        &mut self
    ) -> Option<&<<Self::Writer as RTPSWriter>::HistoryCache as RTPSHistoryCache>::CacheChange>;
    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[<<<Self::Writer as RTPSWriter>::HistoryCache as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::SequenceNumber]
    );
}
