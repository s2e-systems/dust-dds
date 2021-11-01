use crate::structure::types::{Locator, SequenceNumber};

#[derive(Debug)]
pub struct RtpsReaderLocator {
    pub locator: Locator,
    pub expects_inline_qos: bool,
}

impl RtpsReaderLocator {
    pub fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
        }
    }
}

pub trait RtpsReaderLocatorOperations {
    type SequenceNumberVector;

    fn next_requested_change(&mut self) -> Option<SequenceNumber>;

    fn next_unsent_change(
        &mut self,
        last_change_sequence_number: &SequenceNumber,
    ) -> Option<SequenceNumber>;

    fn requested_changes(&self) -> Self::SequenceNumberVector;

    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[SequenceNumber],
        last_change_sequence_number: &SequenceNumber,
    );

    fn unsent_changes(
        &self,
        last_change_sequence_number: SequenceNumber,
    ) -> Self::SequenceNumberVector;
}
