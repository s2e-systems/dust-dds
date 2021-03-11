use crate::{
    structure::RTPSHistoryCache,
    types::{Locator, SequenceNumber},
};

pub trait RTPSReaderLocator {
    type CacheChangeRepresentation;
    type CacheChangeRepresentationList: IntoIterator<Item = Self::CacheChangeRepresentation>;

    fn requested_changes(&self) -> Self::CacheChangeRepresentationList;
    fn unsent_changes(&self, writer_cache: &impl RTPSHistoryCache) -> Self::CacheChangeRepresentationList;

    fn new(locator: Locator, expects_inline_qos: bool) -> Self;
    fn locator(&self) -> Locator;
    fn expects_inline_qos(&self) -> bool;

    fn next_requested_change(&mut self) -> Option<Self::CacheChangeRepresentation>;
    fn next_unsent_change(&mut self, writer_cache: &impl RTPSHistoryCache) -> Option<Self::CacheChangeRepresentation>;
    fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber], writer_cache: &impl RTPSHistoryCache);
}
