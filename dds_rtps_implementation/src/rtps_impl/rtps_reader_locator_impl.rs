use rust_rtps_pim::{
    behavior::writer::reader_locator::{
        RtpsReaderLocatorAttributes, RtpsReaderLocatorConstructor, RtpsReaderLocatorOperations,
    },
    structure::types::{Locator, SequenceNumber},
};
pub struct RtpsReaderLocatorImpl {
    requested_changes: Vec<SequenceNumber>,
    unsent_changes: Vec<SequenceNumber>,
    locator: Locator,
    expects_inline_qos: bool,
}

impl RtpsReaderLocatorImpl {
    pub fn unsent_changes_reset(&mut self) {
        self.unsent_changes = vec![];
    }
}

impl RtpsReaderLocatorConstructor for RtpsReaderLocatorImpl {
    type CacheChangeType = SequenceNumber;
    fn new(
        unsent_changes: &[Self::CacheChangeType],
        locator: Locator,
        expects_inline_qos: bool,
    ) -> Self {
        Self {
            locator,
            expects_inline_qos,
            requested_changes: vec![],
            unsent_changes: unsent_changes.to_vec(),
        }
    }
}

impl RtpsReaderLocatorAttributes for RtpsReaderLocatorImpl {
    type CacheChangeType = SequenceNumber;

    fn requested_changes(&self) -> &[Self::CacheChangeType] {
        &self.requested_changes
    }
    fn unsent_changes(&self) -> &[Self::CacheChangeType] {
        &self.unsent_changes
    }
    fn locator(&self) -> &Locator {
        &self.locator
    }
    fn expects_inline_qos(&self) -> &bool {
        &self.expects_inline_qos
    }
}

impl RtpsReaderLocatorOperations for RtpsReaderLocatorImpl {
    type CacheChangeType = SequenceNumber;

    fn next_requested_change(&mut self) -> Option<Self::CacheChangeType> {
        if self.requested_changes.is_empty() {
            None
        } else {
            Some(self.requested_changes.remove(0))
        }
    }

    fn next_unsent_change(&mut self) -> Option<Self::CacheChangeType> {
        if self.unsent_changes.is_empty() {
            None
        } else {
            Some(self.unsent_changes.remove(0))
        }
    }

    fn requested_changes_set(&mut self, req_seq_num_set: &[Self::CacheChangeType]) {
        self.requested_changes = req_seq_num_set.to_vec();
    }

    fn unsent_changes_add(&mut self, seq_num: &Self::CacheChangeType) {
        // add only changes that are not yet present
        if self.unsent_changes.iter().all(|c| c != seq_num) {
            self.unsent_changes.push(*seq_num);
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::types::LOCATOR_INVALID;

    use super::*;

    #[test]
    fn reader_locator_next_unsent_change() {
        let mut reader_locator = RtpsReaderLocatorImpl::new(&[1, 2], LOCATOR_INVALID, false);

        assert_eq!(reader_locator.next_unsent_change(), Some(1));
        assert_eq!(reader_locator.next_unsent_change(), Some(2));
        assert_eq!(reader_locator.next_unsent_change(), None);
    }

    #[test]
    fn reader_locator_requested_changes_set() {
        let mut reader_locator = RtpsReaderLocatorImpl::new(&[], LOCATOR_INVALID, false);

        let req_seq_num_set = vec![1, 2, 3];
        reader_locator.requested_changes_set(&req_seq_num_set);

        let expected_requested_changes = vec![1, 2, 3];
        assert_eq!(
            reader_locator.requested_changes(),
            expected_requested_changes
        )
    }

    #[test]
    fn reader_locator_unsent_changes() {
        let reader_locator = RtpsReaderLocatorImpl::new(&[1, 2, 3], LOCATOR_INVALID, false);

        let unsent_changes = reader_locator.unsent_changes();
        let expected_unsent_changes = vec![1, 2, 3];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }
}
