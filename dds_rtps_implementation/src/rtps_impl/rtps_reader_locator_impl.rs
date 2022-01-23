use rust_rtps_pim::{
    behavior::writer::reader_locator::{
        RtpsReaderLocatorAttributes, RtpsReaderLocatorConstructor, RtpsReaderLocatorOperations,
    },
    structure::types::{Locator, SequenceNumber},
};
pub struct RtpsReaderLocatorImpl {
    locator: Locator,
    expects_inline_qos: bool,
    last_sent_sequence_number: SequenceNumber,
    requested_changes: Vec<SequenceNumber>,
    unsent_changes: Vec<SequenceNumber>,
}

impl RtpsReaderLocatorImpl {
    pub fn unsent_changes_reset(&mut self) {
        // self.last_sent_sequence_number = 0;
        self.unsent_changes = vec![];
    }
}

impl RtpsReaderLocatorConstructor for RtpsReaderLocatorImpl {
    fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
            last_sent_sequence_number: 0,
            requested_changes: vec![],
            unsent_changes: vec![],
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

    fn next_unsent_change(
        &mut self,
    ) -> Option<Self::CacheChangeType> {
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
        self.unsent_changes.push(*seq_num)
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::types::LOCATOR_INVALID;

    use super::*;

    // #[test]
    // fn reader_locator_next_unsent_change() {
    //     let mut reader_locator =
    //         RtpsReaderLocatorImpl::new(LOCATOR_INVALID, false);

    //     assert_eq!(reader_locator.next_unsent_change(&2), Some(1));
    //     assert_eq!(reader_locator.next_unsent_change(&2), Some(2));
    //     assert_eq!(reader_locator.next_unsent_change(&2), None);
    // }

    // #[test]
    // fn reader_locator_next_unsent_change_non_compliant_last_change_sequence_number() {
    //     let mut reader_locator =
    //         RtpsReaderLocatorImpl::new(LOCATOR_INVALID, false);

    //     assert_eq!(reader_locator.next_unsent_change(&2), Some(1));
    //     assert_eq!(reader_locator.next_unsent_change(&2), Some(2));
    //     assert_eq!(reader_locator.next_unsent_change(&2), None);
    //     assert_eq!(reader_locator.next_unsent_change(&0), None);
    //     assert_eq!(reader_locator.next_unsent_change(&-10), None);
    //     assert_eq!(reader_locator.next_unsent_change(&3), Some(3));
    // }

    // #[test]
    // fn reader_locator_requested_changes_set() {
    //     let mut reader_locator =
    //         RtpsReaderLocatorImpl::new(LOCATOR_INVALID, false);

    //     let req_seq_num_set = vec![1, 2, 3];
    //     reader_locator.requested_changes_set(&req_seq_num_set, &3);

    //     let expected_requested_changes = vec![1, 2, 3];
    //     assert_eq!(
    //         reader_locator.requested_changes(),
    //         expected_requested_changes
    //     )
    // }

    // #[test]
    // fn reader_locator_requested_changes_set_above_last_change_sequence_number() {
    //     let mut reader_locator =
    //         RtpsReaderLocatorImpl::new(LOCATOR_INVALID, false);

    //     let req_seq_num_set = vec![1, 2, 3];
    //     reader_locator.requested_changes_set(&req_seq_num_set, &1);

    //     let expected_requested_changes = vec![1];
    //     assert_eq!(
    //         reader_locator.requested_changes(),
    //         expected_requested_changes
    //     )
    // }

    // #[test]
    // fn reader_locator_unsent_changes() {
    //     let reader_locator =
    //         RtpsReaderLocatorImpl::new(LOCATOR_INVALID, false);

    //     let unsent_changes = reader_locator.unsent_changes(&3);
    //     let expected_unsent_changes = vec![1, 2, 3];

    //     assert_eq!(unsent_changes, expected_unsent_changes);
    // }

    // #[test]
    // fn reader_locator_unsent_changes_after_next_unsent_change() {
    //     let mut reader_locator =
    //         RtpsReaderLocatorImpl::new(LOCATOR_INVALID, false);

    //     let last_change_sequence_number = 3;
    //     reader_locator.next_unsent_change(&last_change_sequence_number);
    //     let unsent_changes = reader_locator.unsent_changes();

    //     let expected_unsent_changes = vec![2, 3];

    //     assert_eq!(unsent_changes, expected_unsent_changes);
    // }
}
