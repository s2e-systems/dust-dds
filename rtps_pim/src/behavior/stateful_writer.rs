use crate::structure::types::{EntityIdPIM, LocatorPIM, SequenceNumberPIM, GUIDPIM};

pub trait RTPSReaderProxy<PSM> {
    type SequenceNumberVector;

    fn remote_reader_guid(&self) -> &PSM::GUIDType
    where
        PSM: GUIDPIM<PSM>;
    fn remote_group_entity_id(&self) -> &PSM::EntityIdType
    where
        PSM: EntityIdPIM;
    fn unicast_locator_list(&self) -> &[PSM::LocatorType]
    where
        PSM: LocatorPIM;
    fn multicast_locator_list(&self) -> &[PSM::LocatorType]
    where
        PSM: LocatorPIM;
    fn expects_inline_qos(&self) -> bool;
    fn is_active(&self) -> bool;

    fn acked_changes_set(&mut self, committed_seq_num: PSM::SequenceNumberType)
    where
        PSM: SequenceNumberPIM;
    fn next_requested_change(&mut self) -> Option<PSM::SequenceNumberType>
    where
        PSM: SequenceNumberPIM;
    fn next_unsent_change(&mut self) -> Option<PSM::SequenceNumberType>
    where
        PSM: SequenceNumberPIM;
    fn unsent_changes(&self) -> Self::SequenceNumberVector;
    fn requested_changes(&self) -> Self::SequenceNumberVector;
    fn requested_changes_set(&mut self, req_seq_num_set: Self::SequenceNumberVector);
    fn unacked_changes(&self) -> Self::SequenceNumberVector;
}

pub trait RTPSStatefulWriter<PSM> {
    type ReaderProxyType;

    fn matched_readers(&self) -> &[Self::ReaderProxyType];

    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType);

    fn matched_reader_remove(&mut self, reader_proxy_guid: &PSM::GUIDType)
    where
        PSM: GUIDPIM<PSM>;

    fn matched_reader_lookup(
        &self,
        a_reader_guid: &PSM::GUIDType,
    ) -> Option<&Self::ReaderProxyType>
    where
        PSM: GUIDPIM<PSM>;

    fn is_acked_by_all(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use crate::{behavior::types::DurationPIM, messages::{submessage_elements::{ParameterListSubmessageElementPIM, ParameterListSubmessageElementType, ParameterType}, types::ParameterIdPIM}, structure::types::{DataPIM, GUIDType, GuidPrefixPIM, InstanceHandlePIM, LocatorType}};

    use super::*;

    struct MockPSM;

    impl SequenceNumberPIM for MockPSM {
        type SequenceNumberType = i64;
        const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumberType = -1;
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

    impl GUIDPIM<Self> for MockPSM {
        type GUIDType = [u8; 16];
        const GUID_UNKNOWN: Self::GUIDType = [0; 16];
    }

    impl LocatorPIM for MockPSM {
        type LocatorType = MockLocator;

        const LOCATOR_INVALID: Self::LocatorType = MockLocator;
        const LOCATOR_KIND_INVALID: <Self::LocatorType as LocatorType>::LocatorKind = [0; 4];
        const LOCATOR_KIND_RESERVED: <Self::LocatorType as LocatorType>::LocatorKind = [0; 4];
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv4: <Self::LocatorType as LocatorType>::LocatorKind = [0; 4];
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv6: <Self::LocatorType as LocatorType>::LocatorKind = [0; 4];
        const LOCATOR_PORT_INVALID: <Self::LocatorType as LocatorType>::LocatorPort = [0; 4];
        const LOCATOR_ADDRESS_INVALID: <Self::LocatorType as LocatorType>::LocatorAddress = [0; 16];
    }

    impl DurationPIM for MockPSM {
        type DurationType = i64;
    }

    impl DataPIM for MockPSM {
        type DataType = [u8; 0];
    }

    impl ParameterIdPIM for MockPSM {
        type ParameterIdType = ();
    }

    impl ParameterListSubmessageElementPIM<Self> for MockPSM {
        type ParameterListSubmessageElementType = MockParameterList;
    }

    impl InstanceHandlePIM for MockPSM {
        type InstanceHandleType = ();
    }

    #[derive(Clone, Copy, PartialEq)]
    struct MockGUID;

    impl GUIDType<MockPSM> for [u8; 16] {
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

    impl LocatorType for MockLocator {
        type LocatorKind = [u8; 4];

        type LocatorPort = [u8; 4];

        type LocatorAddress = [u8; 16];

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

    struct MockParameterList;

    impl ParameterListSubmessageElementType<MockPSM> for MockParameterList {
        type Parameter = MockParameter;

        fn new(_parameter: &[Self::Parameter]) -> Self {
            todo!()
        }

        fn parameter(&self) -> &[Self::Parameter] {
            todo!()
        }
    }

    struct MockParameter;

    impl ParameterType<MockPSM> for MockParameter {
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

    struct MockReaderProxy;

    impl RTPSReaderProxy<MockPSM> for MockReaderProxy {
        type SequenceNumberVector = [i64; 2];

        fn remote_reader_guid(&self) -> &[u8; 16] {
            todo!()
        }

        fn remote_group_entity_id(&self) -> &[u8; 4] {
            todo!()
        }

        fn unicast_locator_list(&self) -> &[MockLocator] {
            todo!()
        }

        fn multicast_locator_list(&self) -> &[MockLocator] {
            todo!()
        }

        fn expects_inline_qos(&self) -> bool {
            todo!()
        }

        fn is_active(&self) -> bool {
            todo!()
        }

        fn acked_changes_set(&mut self, _committed_seq_num: i64) {
            todo!()
        }

        fn next_requested_change(&mut self) -> Option<i64> {
            todo!()
        }

        fn next_unsent_change(&mut self) -> Option<i64> {
            todo!()
        }

        fn unsent_changes(&self) -> Self::SequenceNumberVector {
            todo!()
        }

        fn requested_changes(&self) -> Self::SequenceNumberVector {
            todo!()
        }

        fn requested_changes_set(&mut self, _req_seq_num_set: Self::SequenceNumberVector) {
            todo!()
        }

        fn unacked_changes(&self) -> Self::SequenceNumberVector {
            todo!()
        }
    }
}
