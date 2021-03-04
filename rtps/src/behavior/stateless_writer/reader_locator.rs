use crate::types::{Locator, SequenceNumber};

pub trait ReaderLocator {
    type CacheChangeRepresentation;

    fn requested_changes(&self) -> &[Self::CacheChangeRepresentation];
    fn unsent_changes(&self) -> &[Self::CacheChangeRepresentation];

    fn locator(&self) -> Locator;
    fn expects_inline_qos(&self) -> bool;

    fn new(locator: Locator) -> Self;
    fn next_requested_change(&mut self) -> Option<&Self::CacheChangeRepresentation>;
    fn next_unsent_change(&mut self) -> Option<&Self::CacheChangeRepresentation>;
    fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]);
}