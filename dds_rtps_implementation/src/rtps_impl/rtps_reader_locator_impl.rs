use rust_rtps_pim::{
    behavior::stateless_writer::RTPSReaderLocator,
    structure::types::{LocatorPIM, SequenceNumberPIM},
};

pub trait RTPSReaderLocatorImplTrait: LocatorPIM + SequenceNumberPIM {}

impl<T: LocatorPIM + SequenceNumberPIM> RTPSReaderLocatorImplTrait for T {}

pub struct RTPSReaderLocatorImpl<PSM: RTPSReaderLocatorImplTrait> {
    locator: PSM::LocatorType,
    expects_inline_qos: bool,
    last_sent_sequence_number: PSM::SequenceNumberType,
    requested_changes: Vec<PSM::SequenceNumberType>,
}

impl<PSM: RTPSReaderLocatorImplTrait> RTPSReaderLocatorImpl<PSM> {
    pub fn new(locator: PSM::LocatorType, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
            last_sent_sequence_number: 0.into(),
            requested_changes: Vec::new(),
        }
    }
}

impl<PSM: RTPSReaderLocatorImplTrait> RTPSReaderLocator<PSM> for RTPSReaderLocatorImpl<PSM>
where
    PSM::SequenceNumberType: Clone + Ord + Copy,
{
    type SequenceNumberVector = Vec<PSM::SequenceNumberType>;

    fn locator(&self) -> &PSM::LocatorType {
        &self.locator
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    fn next_requested_change(&mut self) -> Option<PSM::SequenceNumberType> {
        if let Some(requested_change) = self.requested_changes.iter().min().cloned() {
            self.requested_changes.retain(|x| x != &requested_change);
            Some(requested_change.clone())
        } else {
            None
        }
    }

    fn next_unsent_change(
        &mut self,
        last_change_sequence_number: &PSM::SequenceNumberType,
    ) -> Option<PSM::SequenceNumberType> {
        if &self.last_sent_sequence_number < last_change_sequence_number {
            self.last_sent_sequence_number = (self.last_sent_sequence_number.into() + 1).into();
            Some(self.last_sent_sequence_number.clone())
        } else {
            None
        }
    }

    fn requested_changes(&self) -> Self::SequenceNumberVector {
        self.requested_changes.clone()
    }

    fn requested_changes_set<T: IntoIterator<Item = PSM::SequenceNumberType>>(
        &mut self,
        req_seq_num_set: T,
        last_change_sequence_number: PSM::SequenceNumberType,
    ) {
        for requested_change in req_seq_num_set {
            if requested_change <= last_change_sequence_number {
                self.requested_changes.push(requested_change)
            }
        }
    }

    fn unsent_changes(
        &self,
        last_change_sequence_number: PSM::SequenceNumberType,
    ) -> Self::SequenceNumberVector {
        let mut unsent_changes = Vec::new();
        for unsent_change_seq_num in
            self.last_sent_sequence_number.into() + 1..=last_change_sequence_number.into()
        {
            unsent_changes.push(unsent_change_seq_num.into())
        }
        unsent_changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPSM;

    #[derive(Clone, Copy, PartialEq)]
    pub struct MockLocator(u8);

    impl rust_rtps_pim::structure::types::Locator for MockLocator {
        type LocatorKind = [u8; 4];

        const LOCATOR_KIND_INVALID: Self::LocatorKind = [0; 4];
        const LOCATOR_KIND_RESERVED: Self::LocatorKind = [1; 4];
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv4: Self::LocatorKind = [2; 4];
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv6: Self::LocatorKind = [3; 4];

        type LocatorPort = [u8; 4];
        const LOCATOR_PORT_INVALID: Self::LocatorPort = [0; 4];

        type LocatorAddress = [u8; 16];

        const LOCATOR_ADDRESS_INVALID: Self::LocatorAddress = [0; 16];

        fn kind(&self) -> &Self::LocatorKind {
            todo!()
        }

        fn port(&self) -> &Self::LocatorPort {
            todo!()
        }

        fn address(&self) -> &Self::LocatorAddress {
            todo!()
        }
    }

    impl rust_rtps_pim::structure::types::LocatorPIM for MockPSM {
        type LocatorType = MockLocator;

        const LOCATOR_INVALID: Self::LocatorType = MockLocator(0);
    }

    impl rust_rtps_pim::structure::types::SequenceNumberPIM for MockPSM {
        type SequenceNumberType = i64;

        const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumberType = 0;
    }

    #[test]
    fn reader_locator_next_unsent_change() {
        let mut reader_locator: RTPSReaderLocatorImpl<MockPSM> =
            RTPSReaderLocatorImpl::new(MockLocator(0), false);

        assert_eq!(reader_locator.next_unsent_change(&2), Some(1));
        assert_eq!(reader_locator.next_unsent_change(&2), Some(2));
        assert_eq!(reader_locator.next_unsent_change(&2), None);
    }

    #[test]
    fn reader_locator_next_unsent_change_non_compliant_last_change_sequence_number() {
        let mut reader_locator: RTPSReaderLocatorImpl<MockPSM> =
            RTPSReaderLocatorImpl::new(MockLocator(0), false);

        assert_eq!(reader_locator.next_unsent_change(&2), Some(1));
        assert_eq!(reader_locator.next_unsent_change(&2), Some(2));
        assert_eq!(reader_locator.next_unsent_change(&2), None);
        assert_eq!(reader_locator.next_unsent_change(&0), None);
        assert_eq!(reader_locator.next_unsent_change(&-10), None);
        assert_eq!(reader_locator.next_unsent_change(&3), Some(3));
    }

    #[test]
    fn reader_locator_requested_changes_set() {
        let mut reader_locator: RTPSReaderLocatorImpl<MockPSM> =
            RTPSReaderLocatorImpl::new(MockLocator(0), false);

        let req_seq_num_set = vec![1, 2, 3];
        reader_locator.requested_changes_set(req_seq_num_set, 3);

        let expected_requested_changes = vec![1, 2, 3];
        assert_eq!(
            reader_locator.requested_changes(),
            expected_requested_changes
        )
    }

    #[test]
    fn reader_locator_requested_changes_set_above_last_change_sequence_number() {
        let mut reader_locator: RTPSReaderLocatorImpl<MockPSM> =
            RTPSReaderLocatorImpl::new(MockLocator(0), false);

        let req_seq_num_set = vec![1, 2, 3];
        reader_locator.requested_changes_set(req_seq_num_set, 1);

        let expected_requested_changes = vec![1];
        assert_eq!(
            reader_locator.requested_changes(),
            expected_requested_changes
        )
    }

    #[test]
    fn reader_locator_unsent_changes() {
        let reader_locator: RTPSReaderLocatorImpl<MockPSM> =
            RTPSReaderLocatorImpl::new(MockLocator(0), false);

        let unsent_changes = reader_locator.unsent_changes(3);
        let expected_unsent_changes = vec![1, 2, 3];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn reader_locator_unsent_changes_after_next_unsent_change() {
        let mut reader_locator: RTPSReaderLocatorImpl<MockPSM> =
            RTPSReaderLocatorImpl::new(MockLocator(0), false);

        let last_change_sequence_number = 3;
        reader_locator.next_unsent_change(&last_change_sequence_number);
        let unsent_changes = reader_locator.unsent_changes(last_change_sequence_number);

        let expected_unsent_changes = vec![2, 3];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }
}
