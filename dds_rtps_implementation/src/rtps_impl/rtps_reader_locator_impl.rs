use rust_rtps_pim::{
    behavior::stateless_writer::RTPSReaderLocator,
    structure::types::{LocatorType, SequenceNumberType},
};

pub trait RTPSReaderLocatorImplTrait: LocatorType + SequenceNumberType {}

impl<T: LocatorType + SequenceNumberType> RTPSReaderLocatorImplTrait for T {}

pub struct RTPSReaderLocatorImpl<PSM: RTPSReaderLocatorImplTrait> {
    locator: PSM::Locator,
    expects_inline_qos: bool,
}

impl<PSM: RTPSReaderLocatorImplTrait> RTPSReaderLocatorImpl<PSM> {
    fn new(locator: PSM::Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
        }
    }
}

impl<PSM: RTPSReaderLocatorImplTrait> RTPSReaderLocator<PSM> for RTPSReaderLocatorImpl<PSM> {
    type SequenceNumberVector = Vec<PSM::SequenceNumber>;

    fn locator(&self) -> &PSM::Locator {
        &self.locator
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    fn next_requested_change(&mut self) -> Option<PSM::SequenceNumber> {
        todo!()
    }

    fn next_unsent_change(
        &mut self,
        last_change_sequence_number: PSM::SequenceNumber,
    ) -> Option<PSM::SequenceNumber> {
        todo!()
    }

    fn requested_changes(&self) -> Self::SequenceNumberVector {
        todo!()
    }

    fn requested_changes_set(
        &mut self,
        req_seq_num_set: Self::SequenceNumberVector,
        last_change_sequence_number: PSM::SequenceNumber,
    ) {
        todo!()
    }

    fn unsent_changes(
        &self,
        last_change_sequence_number: PSM::SequenceNumber,
    ) -> Self::SequenceNumberVector {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPSM;

    #[derive(Clone, Copy, PartialEq)]
    pub struct MockLocator(u8);

    impl rust_rtps_pim::structure::types::LocatorSubTypes for MockLocator {
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
        const LOCATOR_INVALID: Self = MockLocator(0);

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

    impl rust_rtps_pim::structure::types::LocatorType for MockPSM {
        type Locator = MockLocator;
    }

    impl rust_rtps_pim::structure::types::SequenceNumberType for MockPSM {
        type SequenceNumber = i64;

        const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber = 0;
    }

    #[test]
    fn reader_locator_requested_changes_set() {
        let mut reader_locator: RTPSReaderLocatorImpl<MockPSM> =
            RTPSReaderLocatorImpl::new(MockLocator(0), false);

        let req_seq_num_set = vec![1, 2, 3];
        reader_locator.requested_changes_set(req_seq_num_set, 3);

        let expected_requested_changes = vec![1, 2, 3];
        assert_eq!(reader_locator.requested_changes(), expected_requested_changes)
    }

    //     #[test]
    //     fn reader_locator_requested_changes_set_changes_not_in_history_cache() {
    //         let mut reader_locator = RTPSReaderLocator::new(Locator::new(0, 0, [0; 16]), false);

    //         let mut writer = MockWriter::new();

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 1,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 3,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 5,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         let req_seq_num_set = vec![1, 2, 3, 4, 5];
    //         reader_locator.requested_changes_set(req_seq_num_set, writer.writer_cache());

    //         let expected_requested_changes = vec![1, 3, 5];
    //         assert_eq!(reader_locator.requested_changes, expected_requested_changes);
    //     }

    //     #[test]
    //     fn reader_locator_next_requested_change() {
    //         let mut reader_locator = RTPSReaderLocator::new(Locator::new(0, 0, [0; 16]), false);

    //         let mut writer = MockWriter::new();

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 1,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 2,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 3,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         let req_seq_num_set = vec![1, 2, 3];
    //         reader_locator.requested_changes_set(req_seq_num_set, writer.writer_cache());

    //         assert_eq!(reader_locator.next_requested_change(), Some(1));
    //         assert_eq!(reader_locator.next_requested_change(), Some(2));
    //         assert_eq!(reader_locator.next_requested_change(), Some(3));
    //         assert_eq!(reader_locator.next_requested_change(), None);
    //     }

    //     #[test]
    //     fn reader_locator_unsent_changes() {
    //         let reader_locator: RTPSReaderLocator<MockPsm> =
    //             RTPSReaderLocator::new(Locator::new(0, 0, [0; 16]), false);

    //         let mut writer = MockWriter::new();

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 1,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 2,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 3,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         let unsent_changes = reader_locator.unsent_changes(writer.writer_cache());
    //         let expected_unsent_changes = vec![1, 2, 3];
    //         assert_eq!(unsent_changes, expected_unsent_changes);
    //     }

    //     #[test]
    //     fn reader_locator_unsent_changes_with_non_consecutive_changes() {
    //         let reader_locator = RTPSReaderLocator::new(Locator::new(0, 0, [0; 16]), false);

    //         let mut writer = MockWriter::new();

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 1,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 3,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 5,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         let unsent_changes = reader_locator.unsent_changes(writer.writer_cache());
    //         let expected_unsent_changes = vec![1, 3, 5];
    //         assert_eq!(unsent_changes, expected_unsent_changes);
    //     }

    //     #[test]
    //     fn reader_locator_next_unsent_change() {
    //         let mut reader_locator = RTPSReaderLocator::new(Locator::new(0, 0, [0; 16]), false);

    //         let mut writer = MockWriter::new();

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 1,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 3,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         writer.writer_cache_mut().add_change(RTPSCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: GUID::new([1; 12], [1; 4]),
    //             instance_handle: 1,
    //             sequence_number: 5,
    //             data_value: vec![],
    //             // inline_qos: vec![],
    //         });

    //         assert_eq!(
    //             reader_locator.next_unsent_change(writer.writer_cache()),
    //             Some(1)
    //         );
    //         assert_eq!(
    //             reader_locator.next_unsent_change(writer.writer_cache()),
    //             Some(3)
    //         );
    //         assert_eq!(
    //             reader_locator.next_unsent_change(writer.writer_cache()),
    //             Some(5)
    //         );
    //         assert_eq!(
    //             reader_locator.next_unsent_change(writer.writer_cache()),
    //             None
    //         );
    //     }
}
