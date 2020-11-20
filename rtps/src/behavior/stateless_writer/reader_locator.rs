use std::collections::BTreeSet;

use crate::types::Locator;
use rust_dds_interface::types::SequenceNumber;

pub struct ReaderLocator {
    //requested_changes: HashSet<CacheChange>,
    // unsent_changes: SequenceNumber,
    locator: Locator,

    highest_sequence_number_sent: SequenceNumber,
}

impl ReaderLocator {
    pub fn new(locator: Locator) -> Self {
        Self {
            locator,
            highest_sequence_number_sent: 0,
        }
    }

    pub fn unsent_changes_reset(&mut self) {
        self.highest_sequence_number_sent = 0;
    }

    pub fn unsent_changes(
        &self,
        last_change_sequence_number: SequenceNumber,
    ) -> BTreeSet<SequenceNumber> {
        let mut unsent_changes_set = BTreeSet::new();

        for unsent_sequence_number in
            self.highest_sequence_number_sent + 1..=last_change_sequence_number
        {
            unsent_changes_set.insert(unsent_sequence_number);
        }

        unsent_changes_set
    }

    pub fn next_unsent_change(
        &mut self,
        last_change_sequence_number: SequenceNumber,
    ) -> Option<SequenceNumber> {
        let next_unsent_sequence_number = self.highest_sequence_number_sent + 1;
        if next_unsent_sequence_number > last_change_sequence_number {
            None
        } else {
            self.highest_sequence_number_sent = next_unsent_sequence_number;
            Some(next_unsent_sequence_number)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsent_change_operations() {
        let locator = Locator::new_udpv4(7400, [127,0,0,1]);
        let mut reader_locator = ReaderLocator::new(locator);

        let unsent_changes = reader_locator.unsent_changes(0);
        assert!(unsent_changes.is_empty());

        let unsent_changes = reader_locator.unsent_changes(2);
        assert_eq!(unsent_changes.len(), 2);
        assert!(unsent_changes.contains(&1));
        assert!(unsent_changes.contains(&2));

        let next_unsent_change = reader_locator.next_unsent_change(2);
        assert_eq!(next_unsent_change, Some(1));
        let next_unsent_change = reader_locator.next_unsent_change(2);
        assert_eq!(next_unsent_change, Some(2));
        let next_unsent_change = reader_locator.next_unsent_change(2);
        assert_eq!(next_unsent_change, None);

        // Test also that the system is robust if the last_change_sequence_number input does not follow the precondition
        // of being a constantly increasing number
        let next_unsent_change = reader_locator.next_unsent_change(1);
        assert_eq!(next_unsent_change, None);
    }

    #[test]
    fn unsent_changes_reset() {
        let locator = Locator::new_udpv4(7400, [127,0,0,1]);
        let mut reader_locator = ReaderLocator::new(locator);

        let next_unsent_change = reader_locator.next_unsent_change(2);
        assert_eq!(next_unsent_change, Some(1));
        let next_unsent_change = reader_locator.next_unsent_change(2);
        assert_eq!(next_unsent_change, Some(2));
        let next_unsent_change = reader_locator.next_unsent_change(2);
        assert_eq!(next_unsent_change, None);

        reader_locator.unsent_changes_reset();

        let next_unsent_change = reader_locator.next_unsent_change(2);
        assert_eq!(next_unsent_change, Some(1));
        let next_unsent_change = reader_locator.next_unsent_change(2);
        assert_eq!(next_unsent_change, Some(2));
        let next_unsent_change = reader_locator.next_unsent_change(2);
        assert_eq!(next_unsent_change, None);
    }
}
