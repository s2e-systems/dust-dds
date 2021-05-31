use crate::{
    behavior::RTPSWriter,
    messages::types::ParameterIdType,
    structure::{
        types::{
            DataType, EntityIdType, GUIDType, GuidPrefixType, InstanceHandleType, LocatorType,
            ParameterListType, SequenceNumberType,
        },
        RTPSCacheChange, RTPSHistoryCache,
    },
};

use super::types::DurationType;
pub trait RTPSReaderLocator<PSM: LocatorType + SequenceNumberType> {
    type SequenceNumberVector; //: IntoIterator<Item = PSM::SequenceNumber>;

    fn locator(&self) -> &PSM::Locator;

    fn expects_inline_qos(&self) -> bool;

    fn next_requested_change(&mut self) -> Option<PSM::SequenceNumber>;

    fn next_unsent_change(
        &mut self,
        last_change_sequence_number: PSM::SequenceNumber,
    ) -> Option<PSM::SequenceNumber>;

    fn requested_changes(&self) -> Self::SequenceNumberVector;

    fn requested_changes_set(
        &mut self,
        req_seq_num_set: Self::SequenceNumberVector,
        last_change_sequence_number: PSM::SequenceNumber,
    );

    fn unsent_changes(
        &self,
        last_change_sequence_number: PSM::SequenceNumber,
    ) -> Self::SequenceNumberVector;
}

pub trait RTPSStatelessWriter<
    PSM: GuidPrefixType
        + EntityIdType
        + DurationType
        + DataType
        + InstanceHandleType
        + LocatorType
        + SequenceNumberType
        + GUIDType<PSM>
        + ParameterIdType
        + ParameterListType<PSM>,
>: RTPSWriter<PSM>
{
    type ReaderLocatorType: RTPSReaderLocator<PSM>;

    fn reader_locators(&self) -> &[Self::ReaderLocatorType];

    fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorType);

    fn reader_locator_remove(&mut self, a_locator: &PSM::Locator);

    fn unsent_changes_reset(&mut self);
}

pub fn produce_messages<
    PSM: EntityIdType
        + DataType
        + GuidPrefixType
        + InstanceHandleType
        + LocatorType
        + SequenceNumberType
        + ParameterIdType
        + GUIDType<PSM>
        + ParameterListType<PSM>,
    CacheChange: RTPSCacheChange<PSM>,
>(
    reader_locator: &mut impl RTPSReaderLocator<PSM>,
    writer_cache: &impl RTPSHistoryCache<PSM, CacheChange = CacheChange>,
    last_change_sequence_number: &PSM::SequenceNumber,
    mut send_data_to: impl FnMut(&CacheChange),
    mut send_gap_to: impl FnMut(&PSM::SequenceNumber),
) {
    // Pushing state
    while let Some(seq_num) = reader_locator.next_unsent_change(*last_change_sequence_number) {
        // Transition T4
        if let Some(change) = writer_cache.get_change(&seq_num) {
            send_data_to(change)
        } else {
            send_gap_to(&seq_num)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        messages::submessage_elements::{Parameter, ParameterList},
        structure::types::{LocatorSubTypes, GUID},
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

    impl LocatorSubTypes for MockLocator {
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
        fn parameter(&self) -> &[Self::Parameter] {
            todo!()
        }
    }

    struct MockPSM;

    impl ParameterIdType for MockPSM {
        type ParameterId = ();
    }

    impl ParameterListType<MockPSM> for MockPSM {
        type ParameterList = MockParameterList;
    }

    impl DataType for MockPSM {
        type Data = ();
    }

    impl InstanceHandleType for MockPSM {
        type InstanceHandle = ();
    }

    impl EntityIdType for MockPSM {
        type EntityId = [u8; 4];
        const ENTITYID_UNKNOWN: Self::EntityId = [0; 4];
        const ENTITYID_PARTICIPANT: Self::EntityId = [0; 4];
    }

    impl GuidPrefixType for MockPSM {
        type GuidPrefix = [u8; 12];
        const GUIDPREFIX_UNKNOWN: Self::GuidPrefix = [0; 12];
    }

    impl GUIDType<MockPSM> for MockPSM {
        type GUID = MockGUID;
        const GUID_UNKNOWN: Self::GUID = MockGUID;
    }

    impl SequenceNumberType for MockPSM {
        type SequenceNumber = i64;
        const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber = -1;
    }

    impl LocatorType for MockPSM {
        type Locator = MockLocator;
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

        fn next_unsent_change(&mut self, last_change_sequence_number: i64) -> Option<i64> {
            if self.last_sent_sequence_number < last_change_sequence_number {
                self.last_sent_sequence_number += 1;
                Some(self.last_sent_sequence_number)
            } else {
                None
            }
        }

        fn requested_changes(&self) -> Self::SequenceNumberVector {
            todo!()
        }

        fn requested_changes_set(
            &mut self,
            _req_seq_num_set: Self::SequenceNumberVector,
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

        fn data_value(&self) -> &() {
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

    #[test]
    fn stateless_writer_produce_messages() {
        let mut sent_data_seq_num = [0, 0];
        let mut total_data = 0;
        let mut sent_gap_seq_num = [];
        let mut total_gap = 0;

        let expected_total_data = 2;
        let expected_sent_data_seq_num = [1, 2];
        let expected_total_gap = 0;
        let expected_sent_gap_seq_num = [];

        let mut reader_locator = MockReaderLocator {
            last_sent_sequence_number: 0,
        };
        let writer_cache = MockHistoryCache::<2> {
            changes: [
                MockCacheChange { sequence_number: 1 },
                MockCacheChange { sequence_number: 2 },
            ],
        };
        produce_messages(
            &mut reader_locator,
            &writer_cache,
            &2,
            |cc| {
                sent_data_seq_num[total_data] = cc.sequence_number;
                total_data += 1;
            },
            |gap_seq_num| {
                sent_gap_seq_num[total_gap] = *gap_seq_num;
                total_gap += 1;
            },
        );

        assert_eq!(total_data, expected_total_data);
        assert_eq!(sent_data_seq_num, expected_sent_data_seq_num);
        assert_eq!(total_gap, expected_total_gap);
        assert_eq!(sent_gap_seq_num, expected_sent_gap_seq_num);
    }
}
