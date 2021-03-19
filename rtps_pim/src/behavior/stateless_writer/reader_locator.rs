use crate::{
    behavior::RTPSWriter,
    structure::{RTPSCacheChange, RTPSEndpoint, RTPSHistoryCache},
};

pub trait RTPSReaderLocator {
    type CacheChangeRepresentation;
    type CacheChangeRepresentationList: IntoIterator<Item = Self::CacheChangeRepresentation>;
    type Writer: RTPSWriter;

    // Attributes
    fn locator(&self) -> &<Self::Writer as RTPSEndpoint>::Locator;
    fn expects_inline_qos(&self) -> bool;

    // Operations
    fn new(locator: <Self::Writer as RTPSEndpoint>::Locator, expects_inline_qos: bool) -> Self;
    fn requested_changes(&self, writer: &Self::Writer) -> Self::CacheChangeRepresentationList;
    fn unsent_changes(&self, writer: &Self::Writer) -> Self::CacheChangeRepresentationList;
    fn next_requested_change(
        &mut self,
        writer: &Self::Writer,
    ) -> Option<Self::CacheChangeRepresentation>;
    fn next_unsent_change(
        &mut self,
        writer: &Self::Writer,
    ) -> Option<Self::CacheChangeRepresentation>;
    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[<<<Self::Writer as RTPSWriter>::HistoryCache as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::SequenceNumber],
        writer: &Self::Writer,
    );
}
