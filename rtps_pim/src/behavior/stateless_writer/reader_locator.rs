use crate::{
    behavior::RTPSWriter,
    types::{Locator, SequenceNumber},
};

pub trait RTPSReaderLocator {
    type CacheChangeRepresentation;
    type CacheChangeRepresentationList: IntoIterator<Item = Self::CacheChangeRepresentation>;

    // Attributes
    fn locator(&self) -> Locator;
    fn expects_inline_qos(&self) -> bool;

    // Operations
    fn new(locator: Locator, expects_inline_qos: bool) -> Self;
    fn requested_changes(&self, writer: &impl RTPSWriter) -> Self::CacheChangeRepresentationList;
    fn unsent_changes(&self, writer: &impl RTPSWriter) -> Self::CacheChangeRepresentationList;
    fn next_requested_change(
        &mut self,
        writer: &impl RTPSWriter,
    ) -> Option<Self::CacheChangeRepresentation>;
    fn next_unsent_change(
        &mut self,
        writer: &impl RTPSWriter,
    ) -> Option<Self::CacheChangeRepresentation>;
    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[SequenceNumber],
        writer: &impl RTPSWriter,
    );
}
