use crate::structure::types::{Locator, SequenceNumber};

pub trait RtpsReaderLocatorConstructor {
    fn new(locator: Locator, expects_inline_qos: bool) -> Self;
}

pub trait RtpsReaderLocatorAttributes {
    type CacheChangeType;
    fn requested_changes(&self) -> &[Self::CacheChangeType];
    fn locator(&self) -> &Locator;
    fn expects_inline_qos(&self) -> &bool;
}

pub trait RtpsReaderLocatorOperations {
    fn next_requested_change(&mut self) -> Option<SequenceNumber>;

    fn next_unsent_change(
        &mut self,
        last_change_sequence_number: &SequenceNumber,
    ) -> Option<SequenceNumber>;

    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[SequenceNumber],
        last_change_sequence_number: &SequenceNumber,
    );

    fn unsent_changes(&self) -> &[SequenceNumber];
}
