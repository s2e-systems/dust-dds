use core::marker::PhantomData;

use crate::{
    behavior::RTPSWriter,
    messages::{
        submessage_elements,
        submessages::{
            DataSubmessage, DataSubmessagePIM, GapSubmessage, GapSubmessagePIM,
            HeartbeatSubmessage, HeartbeatSubmessagePIM,
        },
        types::{CountPIM, ParameterIdPIM, SubmessageFlagPIM, SubmessageKindPIM},
    },
    structure::{
        types::{
            DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM, ParameterListPIM,
            SequenceNumberPIM, GUID, GUIDPIM,
        },
        RTPSCacheChange, RTPSHistoryCache,
    },
};

use super::types::DurationPIM;
pub trait RTPSReaderLocator<PSM: LocatorPIM + SequenceNumberPIM> {
    type SequenceNumberVector; //: IntoIterator<Item = PSM::SequenceNumber>;

    fn locator(&self) -> &PSM::LocatorType;

    fn expects_inline_qos(&self) -> bool;

    fn next_requested_change(&mut self) -> Option<PSM::SequenceNumberType>;

    fn next_unsent_change(
        &mut self,
        last_change_sequence_number: &PSM::SequenceNumberType,
    ) -> Option<PSM::SequenceNumberType>;

    fn requested_changes(&self) -> Self::SequenceNumberVector;

    fn requested_changes_set<T: IntoIterator<Item = PSM::SequenceNumberType>>(
        &mut self,
        req_seq_num_set: T,
        last_change_sequence_number: PSM::SequenceNumberType,
    );

    fn unsent_changes(
        &self,
        last_change_sequence_number: PSM::SequenceNumberType,
    ) -> Self::SequenceNumberVector;
}

pub trait RTPSStatelessWriter<
    PSM: GuidPrefixPIM
        + EntityIdPIM
        + DurationPIM
        + DataPIM
        + InstanceHandlePIM
        + LocatorPIM
        + SequenceNumberPIM
        + GUIDPIM<PSM>
        + ParameterIdPIM
        + ParameterListPIM<PSM>,
>: RTPSWriter<PSM>
{
    type ReaderLocatorPIM: RTPSReaderLocator<PSM>;

    fn reader_locators(&mut self) -> (&mut [Self::ReaderLocatorPIM], &Self::HistoryCacheType);

    fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorPIM);

    fn reader_locator_remove(&mut self, a_locator: &PSM::LocatorType);

    fn unsent_changes_reset(&mut self);
}

pub mod best_effort_stateless_writer {
    use super::RTPSReaderLocator;
    use crate::{
        messages::{
            submessage_elements::SequenceNumberSet,
            submessages::AckNackSubmessage,
            types::{CountPIM, SubmessageFlagPIM, SubmessageKindPIM},
        },
        structure::types::{EntityIdPIM, LocatorPIM, SequenceNumberPIM},
    };

    pub fn send_unsent_data<PSM: LocatorPIM + SequenceNumberPIM>(
        reader_locator: &mut impl RTPSReaderLocator<PSM>,
        last_change_sequence_number: PSM::SequenceNumberType,
        mut send: impl FnMut(PSM::SequenceNumberType),
    ) {
        while let Some(seq_num) = reader_locator.next_unsent_change(&last_change_sequence_number) {
            send(seq_num)
        }
    }

    pub fn receive_acknack<
        PSM: LocatorPIM
            + SequenceNumberPIM
            + SubmessageKindPIM
            + SubmessageFlagPIM
            + EntityIdPIM
            + CountPIM,
    >(
        reader_locator: &mut impl RTPSReaderLocator<PSM>,
        acknack_submessage: &impl AckNackSubmessage<PSM>,
        last_change_sequence_number: PSM::SequenceNumberType,
    ) {
        reader_locator.requested_changes_set(
            acknack_submessage.reader_sn_state().set().into_iter(),
            last_change_sequence_number,
        );
    }
}

pub mod reliable_stateless_writer {
    use crate::structure::types::{LocatorPIM, SequenceNumberPIM};

    use super::RTPSReaderLocator;

    pub fn send_unsent_data<PSM: LocatorPIM + SequenceNumberPIM>(
        reader_locator: &mut impl RTPSReaderLocator<PSM>,
        last_change_sequence_number: PSM::SequenceNumberType,
        mut send: impl FnMut(PSM::SequenceNumberType),
    ) {
        while let Some(seq_num) = reader_locator.next_unsent_change(&last_change_sequence_number) {
            send(seq_num)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        messages::submessage_elements::{Parameter, ParameterList},
        structure::types::{Locator, GUID},
    };

    use super::*;

    #[derive(Clone, Copy, PartialEq)]
    struct MockGUID;

    impl GUID<MockPSM> for MockGUID {
        fn new(_prefix: [u8; 12], _entity_id: [u8; 4]) -> Self {
            todo!()
        }

        fn prefix(&self) -> &[u8; 12] {
            todo!()
        }

        fn entity_id(&self) -> &[u8; 4] {
            todo!()
        }
    }
    #[derive(Clone, Copy, PartialEq)]
    struct MockLocator;

    impl Locator for MockLocator {
        type LocatorKind = [u8; 4];
        const LOCATOR_KIND_INVALID: Self::LocatorKind = [0; 4];
        const LOCATOR_KIND_RESERVED: Self::LocatorKind = [0; 4];
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv4: Self::LocatorKind = [0; 4];
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv6: Self::LocatorKind = [0; 4];
        type LocatorPort = [u8; 4];
        const LOCATOR_PORT_INVALID: Self::LocatorPort = [0; 4];
        type LocatorAddress = [u8; 16];
        const LOCATOR_ADDRESS_INVALID: Self::LocatorAddress = [0; 16];
        const LOCATOR_INVALID: Self = MockLocator;

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

    struct MockParameter;

    impl Parameter<MockPSM> for MockParameter {
        fn parameter_id(&self) -> () {
            todo!()
        }

        fn length(&self) -> i16 {
            todo!()
        }

        fn value(&self) -> &[u8] {
            todo!()
        }
    }
    struct MockParameterList;

    impl ParameterList<MockPSM> for MockParameterList {
        type Parameter = MockParameter;
        type ParameterList = MockParameterList;

        fn new(_parameter: Self::ParameterList) -> Self {
            todo!()
        }

        fn parameter(&self) -> &Self::ParameterList {
            todo!()
        }
    }

    struct MockPSM;

    impl ParameterIdPIM for MockPSM {
        type ParameterIdType = ();
    }

    impl ParameterListPIM<MockPSM> for MockPSM {
        type ParameterListType = MockParameterList;
    }

    impl DataPIM for MockPSM {
        type DataType = [u8; 0];
    }

    impl InstanceHandlePIM for MockPSM {
        type InstanceHandleType = ();
    }

    impl EntityIdPIM for MockPSM {
        type EntityIdType = [u8; 4];
        const ENTITYID_UNKNOWN: Self::EntityIdType = [0; 4];
        const ENTITYID_PARTICIPANT: Self::EntityIdType = [0; 4];
    }

    impl GuidPrefixPIM for MockPSM {
        type GuidPrefixType = [u8; 12];
        const GUIDPREFIX_UNKNOWN: Self::GuidPrefixType = [0; 12];
    }

    impl GUIDPIM<MockPSM> for MockPSM {
        type GUIDType = MockGUID;
        const GUID_UNKNOWN: Self::GUIDType = MockGUID;
    }

    impl SequenceNumberPIM for MockPSM {
        type SequenceNumberType = i64;
        const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumberType = -1;
    }

    impl LocatorPIM for MockPSM {
        type LocatorType = MockLocator;
    }

    struct MockReaderLocator {
        last_sent_sequence_number: i64,
    }

    impl<'a> RTPSReaderLocator<MockPSM> for MockReaderLocator {
        type SequenceNumberVector = ();

        fn locator(&self) -> &MockLocator {
            todo!()
        }

        fn expects_inline_qos(&self) -> bool {
            todo!()
        }

        fn next_requested_change(&mut self) -> Option<i64> {
            todo!()
        }

        fn next_unsent_change(&mut self, last_change_sequence_number: &i64) -> Option<i64> {
            if &self.last_sent_sequence_number < last_change_sequence_number {
                self.last_sent_sequence_number += 1;
                Some(self.last_sent_sequence_number)
            } else {
                None
            }
        }

        fn requested_changes(&self) -> Self::SequenceNumberVector {
            todo!()
        }

        fn requested_changes_set<T: IntoIterator<Item = i64>>(
            &mut self,
            _req_seq_num_set: T,
            _last_change_sequence_number: i64,
        ) {
            todo!()
        }

        fn unsent_changes(&self, _last_change_sequence_number: i64) -> Self::SequenceNumberVector {
            todo!()
        }
    }

    struct MockCacheChange {
        sequence_number: i64,
    }

    impl RTPSCacheChange<MockPSM> for MockCacheChange {
        fn kind(&self) -> &crate::structure::types::ChangeKind {
            todo!()
        }

        fn writer_guid(&self) -> &MockGUID {
            todo!()
        }

        fn instance_handle(&self) -> &() {
            todo!()
        }

        fn sequence_number(&self) -> &i64 {
            todo!()
        }

        fn data_value(&self) -> &[u8; 0] {
            todo!()
        }

        fn inline_qos(&self) -> &MockParameterList {
            todo!()
        }
    }
    struct MockHistoryCache<const N: usize> {
        changes: [MockCacheChange; N],
    }

    impl<const N: usize> RTPSHistoryCache<MockPSM> for MockHistoryCache<N> {
        type CacheChange = MockCacheChange;

        fn new() -> Self
        where
            Self: Sized,
        {
            todo!()
        }

        fn add_change(&mut self, _change: Self::CacheChange) {
            todo!()
        }

        fn remove_change(&mut self, _seq_num: &i64) {
            todo!()
        }

        fn get_change(&self, seq_num: &i64) -> Option<&Self::CacheChange> {
            self.changes.iter().find(|&x| &x.sequence_number == seq_num)
        }

        fn get_seq_num_min(&self) -> Option<&i64> {
            todo!()
        }

        fn get_seq_num_max(&self) -> Option<&i64> {
            todo!()
        }
    }

    // #[test]
    // fn stateless_writer_produce_messages_only_data() {
    //     let mut sent_data_seq_num = [0, 0];
    //     let mut total_data = 0;
    //     let mut sent_gap_seq_num = [];
    //     let mut total_gap = 0;

    //     let expected_total_data = 2;
    //     let expected_sent_data_seq_num = [1, 2];
    //     let expected_total_gap = 0;
    //     let expected_sent_gap_seq_num = [];

    //     let mut reader_locator = MockReaderLocator {
    //         last_sent_sequence_number: 0,
    //     };
    //     let writer_cache = MockHistoryCache::<2> {
    //         changes: [
    //             MockCacheChange { sequence_number: 1 },
    //             MockCacheChange { sequence_number: 2 },
    //         ],
    //     };
    //     produce_messages(
    //         &mut reader_locator,
    //         &writer_cache,
    //         &2,
    //         |cc| {
    //             sent_data_seq_num[total_data] = cc.sequence_number;
    //             total_data += 1;
    //         },
    //         |gap_seq_num| {
    //             sent_gap_seq_num[total_gap] = *gap_seq_num;
    //             total_gap += 1;
    //         },
    //     );

    //     assert_eq!(total_data, expected_total_data);
    //     assert_eq!(sent_data_seq_num, expected_sent_data_seq_num);
    //     assert_eq!(total_gap, expected_total_gap);
    //     assert_eq!(sent_gap_seq_num, expected_sent_gap_seq_num);
    // }

    // #[test]
    // fn stateless_writer_produce_messages_only_gap() {
    //     let mut sent_data_seq_num = [];
    //     let mut total_data = 0;
    //     let mut sent_gap_seq_num = [0, 0];
    //     let mut total_gap = 0;

    //     let expected_total_data = 0;
    //     let expected_sent_data_seq_num = [];
    //     let expected_total_gap = 2;
    //     let expected_sent_gap_seq_num = [1, 2];

    //     let mut reader_locator = MockReaderLocator {
    //         last_sent_sequence_number: 0,
    //     };
    //     let writer_cache = MockHistoryCache::<0> { changes: [] };
    //     produce_messages(
    //         &mut reader_locator,
    //         &writer_cache,
    //         &2,
    //         |cc| {
    //             sent_data_seq_num[total_data] = cc.sequence_number;
    //             total_data += 1;
    //         },
    //         |gap_seq_num| {
    //             sent_gap_seq_num[total_gap] = *gap_seq_num;
    //             total_gap += 1;
    //         },
    //     );

    //     assert_eq!(total_data, expected_total_data);
    //     assert_eq!(sent_data_seq_num, expected_sent_data_seq_num);
    //     assert_eq!(total_gap, expected_total_gap);
    //     assert_eq!(sent_gap_seq_num, expected_sent_gap_seq_num);
    // }

    // #[test]
    // fn stateless_writer_produce_messages_data_and_gap() {
    //     let mut sent_data_seq_num = [0];
    //     let mut total_data = 0;
    //     let mut sent_gap_seq_num = [0];
    //     let mut total_gap = 0;

    //     let expected_total_data = 1;
    //     let expected_sent_data_seq_num = [2];
    //     let expected_total_gap = 1;
    //     let expected_sent_gap_seq_num = [1];

    //     let mut reader_locator = MockReaderLocator {
    //         last_sent_sequence_number: 0,
    //     };
    //     let writer_cache = MockHistoryCache::<1> {
    //         changes: [MockCacheChange { sequence_number: 2 }],
    //     };
    //     produce_messages(
    //         &mut reader_locator,
    //         &writer_cache,
    //         &2,
    //         |cc| {
    //             sent_data_seq_num[total_data] = cc.sequence_number;
    //             total_data += 1;
    //         },
    //         |gap_seq_num| {
    //             sent_gap_seq_num[total_gap] = *gap_seq_num;
    //             total_gap += 1;
    //         },
    //     );

    //     assert_eq!(total_data, expected_total_data);
    //     assert_eq!(sent_data_seq_num, expected_sent_data_seq_num);
    //     assert_eq!(total_gap, expected_total_gap);
    //     assert_eq!(sent_gap_seq_num, expected_sent_gap_seq_num);
    // }
}
