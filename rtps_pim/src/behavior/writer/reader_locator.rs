use crate::structure::types::{Locator, SequenceNumber};

pub trait RtpsReaderLocatorAttributes {
    type CacheChangeListType;

    fn unsent_changes_mut(&mut self) -> &mut Self::CacheChangeListType;
    fn requested_changes_mut(&mut self) -> &mut Self::CacheChangeListType;
    fn locator(&self) -> Locator;
    fn expects_inline_qos(&self) -> bool;
}

pub trait RtpsReaderLocatorConstructor {
    type CacheChangeType;
    fn new(locator: Locator, expects_inline_qos: bool) -> Self;
}

pub trait RtpsReaderLocatorOperations {
    type CacheChangeType;
    type CacheChangeListType;

    fn next_requested_change(&mut self) -> Self::CacheChangeType;
    fn next_unsent_change(&mut self) -> Self::CacheChangeType;
    fn requested_changes(&self) -> Self::CacheChangeListType;
    fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]);
    fn unsent_changes(&self) -> Self::CacheChangeListType;
}
