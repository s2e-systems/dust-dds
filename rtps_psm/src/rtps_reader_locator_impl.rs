use std::ops::{Deref, DerefMut};

use rust_rtps_pim::{
    behavior::writer::reader_locator::{RtpsReaderLocator, RtpsReaderLocatorOperations},
    structure::types::SequenceNumber,
};
pub struct RtpsReaderLocatorImpl {
    reader_locator: RtpsReaderLocator,
    last_sent_sequence_number: SequenceNumber,
    requested_changes: Vec<SequenceNumber>,
}

impl Deref for RtpsReaderLocatorImpl {
    type Target = RtpsReaderLocator;

    fn deref(&self) -> &Self::Target {
        &self.reader_locator
    }
}

impl DerefMut for RtpsReaderLocatorImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader_locator
    }
}

impl RtpsReaderLocatorImpl {
    pub fn new(reader_locator: RtpsReaderLocator) -> Self {
        Self {
            reader_locator,
            last_sent_sequence_number: 0,
            requested_changes: Vec::new(),
        }
    }

    pub fn unsent_changes_reset(&mut self) {
        self.last_sent_sequence_number = 0;
    }
}

impl RtpsReaderLocatorOperations for RtpsReaderLocatorImpl {
    type SequenceNumberVector = Vec<SequenceNumber>;

    fn next_requested_change(&mut self) -> Option<SequenceNumber> {
        if let Some(requested_change) = self.requested_changes.iter().min().cloned() {
            self.requested_changes.retain(|x| x != &requested_change);
            Some(requested_change.clone())
        } else {
            None
        }
    }

    fn next_unsent_change(
        &mut self,
        last_change_sequence_number: &SequenceNumber,
    ) -> Option<SequenceNumber> {
        if &self.last_sent_sequence_number < last_change_sequence_number {
            self.last_sent_sequence_number = self.last_sent_sequence_number + 1;
            Some(self.last_sent_sequence_number.clone())
        } else {
            None
        }
    }

    fn requested_changes(&self) -> Self::SequenceNumberVector {
        self.requested_changes.clone()
    }

    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[SequenceNumber],
        last_change_sequence_number: &SequenceNumber,
    ) {
        for requested_change in req_seq_num_set.as_ref() {
            if requested_change <= last_change_sequence_number {
                self.requested_changes.push(requested_change.clone())
            }
        }
    }

    fn unsent_changes(
        &self,
        last_change_sequence_number: &SequenceNumber,
    ) -> Self::SequenceNumberVector {
        let mut unsent_changes = Vec::new();
        for unsent_change_seq_num in
            self.last_sent_sequence_number + 1..=*last_change_sequence_number
        {
            unsent_changes.push(unsent_change_seq_num)
        }
        unsent_changes
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::types::LOCATOR_INVALID;

    use super::*;

    #[test]
    fn reader_locator_next_unsent_change() {
        let mut reader_locator =
            RtpsReaderLocatorImpl::new(RtpsReaderLocator::new(LOCATOR_INVALID, false));

        assert_eq!(reader_locator.next_unsent_change(&2), Some(1));
        assert_eq!(reader_locator.next_unsent_change(&2), Some(2));
        assert_eq!(reader_locator.next_unsent_change(&2), None);
    }

    #[test]
    fn reader_locator_next_unsent_change_non_compliant_last_change_sequence_number() {
        let mut reader_locator =
            RtpsReaderLocatorImpl::new(RtpsReaderLocator::new(LOCATOR_INVALID, false));

        assert_eq!(reader_locator.next_unsent_change(&2), Some(1));
        assert_eq!(reader_locator.next_unsent_change(&2), Some(2));
        assert_eq!(reader_locator.next_unsent_change(&2), None);
        assert_eq!(reader_locator.next_unsent_change(&0), None);
        assert_eq!(reader_locator.next_unsent_change(&-10), None);
        assert_eq!(reader_locator.next_unsent_change(&3), Some(3));
    }

    #[test]
    fn reader_locator_requested_changes_set() {
        let mut reader_locator =
            RtpsReaderLocatorImpl::new(RtpsReaderLocator::new(LOCATOR_INVALID, false));

        let req_seq_num_set = vec![1, 2, 3];
        reader_locator.requested_changes_set(&req_seq_num_set, &3);

        let expected_requested_changes = vec![1, 2, 3];
        assert_eq!(
            reader_locator.requested_changes(),
            expected_requested_changes
        )
    }

    #[test]
    fn reader_locator_requested_changes_set_above_last_change_sequence_number() {
        let mut reader_locator =
            RtpsReaderLocatorImpl::new(RtpsReaderLocator::new(LOCATOR_INVALID, false));

        let req_seq_num_set = vec![1, 2, 3];
        reader_locator.requested_changes_set(&req_seq_num_set, &1);

        let expected_requested_changes = vec![1];
        assert_eq!(
            reader_locator.requested_changes(),
            expected_requested_changes
        )
    }

    #[test]
    fn reader_locator_unsent_changes() {
        let reader_locator =
            RtpsReaderLocatorImpl::new(RtpsReaderLocator::new(LOCATOR_INVALID, false));

        let unsent_changes = reader_locator.unsent_changes(&3);
        let expected_unsent_changes = vec![1, 2, 3];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn reader_locator_unsent_changes_after_next_unsent_change() {
        let mut reader_locator =
            RtpsReaderLocatorImpl::new(RtpsReaderLocator::new(LOCATOR_INVALID, false));

        let last_change_sequence_number = 3;
        reader_locator.next_unsent_change(&last_change_sequence_number);
        let unsent_changes = reader_locator.unsent_changes(&last_change_sequence_number);

        let expected_unsent_changes = vec![2, 3];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }
}
