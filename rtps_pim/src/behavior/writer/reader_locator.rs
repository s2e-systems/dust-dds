use crate::structure::types::{Locator, SequenceNumber};

pub trait RtpsReaderLocatorConstructor {
    type CacheChangeType;
    fn new(locator: Locator, expects_inline_qos: bool) -> Self;
}

pub trait RtpsReaderLocatorAttributes {
    // unsent_changes() and requested_changes() functions are not present
    // in Attributes since it's already present in Operations and requires a link to the history cache

    fn locator(&self) -> Locator;
    fn expects_inline_qos(&self) -> bool;
}

pub trait RtpsReaderLocatorOperations {
    type CacheChangeType;
    type CacheChangeListType;

    fn next_requested_change(&mut self) -> Option<Self::CacheChangeType>;
    fn next_unsent_change(&mut self) -> Option<Self::CacheChangeType>;
    fn requested_changes(&self) -> Self::CacheChangeListType;
    fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]);
    fn unsent_changes(&self) -> Self::CacheChangeListType;
}
