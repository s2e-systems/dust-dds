use crate::structure::types::{Locator, SequenceNumber};

pub trait RtpsReaderLocatorConstructor {
    fn new(locator: Locator, expects_inline_qos: bool) -> Self;
}

pub trait RtpsReaderLocatorAttributes {
    type CacheChangeType;

    fn requested_changes(&self) -> &[Self::CacheChangeType];
    fn unsent_changes(&self) -> &[Self::CacheChangeType];
    fn locator(&self) -> &Locator;
    fn expects_inline_qos(&self) -> &bool;
}

pub trait RtpsReaderLocatorOperations {
    type CacheChangeType;

    fn next_requested_change(&mut self) -> Option<Self::CacheChangeType>;
    fn next_unsent_change(&mut self) -> Option<Self::CacheChangeType>;
    fn requested_changes_set(&mut self, req_seq_num_set: &[Self::CacheChangeType]);

    fn unsent_changes_add(&mut self, unsent_seq_num_set: &Self::CacheChangeType);
    // fn last_sent_change
}
