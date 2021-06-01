use crate::{behavior::{types::DurationType, RTPSWriter}, messages::types::ParameterIdType, structure::{RTPSHistoryCache, types::{
        DataType, EntityIdType, GUIDType, GuidPrefixType, InstanceHandleType, LocatorType,
        ParameterListType, SequenceNumberType,
    }}};

pub trait RTPSReaderProxy<
    PSM: GuidPrefixType + EntityIdType + LocatorType + EntityIdType + GUIDType<PSM> + SequenceNumberType,
>
{
    type SequenceNumberVector; //: IntoIterator<Item = PSM::SequenceNumber>;

    fn remote_reader_guid(&self) -> &PSM::GUID;
    fn remote_group_entity_id(&self) -> &PSM::EntityId;
    fn unicast_locator_list(&self) -> &[PSM::Locator];
    fn multicast_locator_list(&self) -> &[PSM::Locator];
    fn expects_inline_qos(&self) -> bool;
    fn is_active(&self) -> bool;

    fn acked_changes_set(&mut self, committed_seq_num: PSM::SequenceNumber);
    fn next_requested_change(&mut self) -> Option<PSM::SequenceNumber>;
    fn next_unsent_change(&mut self) -> Option<PSM::SequenceNumber>;
    fn unsent_changes(&self) -> Self::SequenceNumberVector;
    fn requested_changes(&self) -> Self::SequenceNumberVector;
    fn requested_changes_set(&mut self, req_seq_num_set: Self::SequenceNumberVector);
    fn unacked_changes(&self) -> Self::SequenceNumberVector;
}

pub trait RTPSStatefulWriter<
    PSM: GuidPrefixType
        + EntityIdType
        + LocatorType
        + EntityIdType
        + DurationType
        + SequenceNumberType
        + DataType
        + ParameterListType<PSM>
        + GUIDType<PSM>
        + InstanceHandleType
        + ParameterIdType,
>: RTPSWriter<PSM>
{
    type ReaderProxyType: RTPSReaderProxy<PSM>;

    fn matched_readers(&self) -> &[Self::ReaderProxyType];

    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType);

    fn matched_reader_remove(&mut self, reader_proxy_guid: &PSM::GUID);

    fn matched_reader_lookup(&self, a_reader_guid: &PSM::GUID) -> Option<&Self::ReaderProxyType>;

    fn is_acked_by_all(&self) -> bool;
}

pub enum DataOrGap {
    Data(),
    Gap(),
}

pub fn can_send<
    PSM: GuidPrefixType
        + EntityIdType
        + LocatorType
        + EntityIdType
        + DurationType
        + SequenceNumberType
        + DataType
        + ParameterListType<PSM>
        + GUIDType<PSM>
        + InstanceHandleType
        + ParameterIdType,
>(
    reader_proxy: &mut impl RTPSReaderProxy<PSM>,
    writer_cache: &impl RTPSHistoryCache<PSM>,
) -> Option<DataOrGap> {
    if let Some(seq_num) = reader_proxy.next_unsent_change() {
        if let Some(change) = writer_cache.get_change(&seq_num) {
            todo!()
        } else {
            todo!()
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        messages::submessage_elements::{Parameter, ParameterList},
        structure::types::{LocatorSubTypes, GUID},
    };

    use super::*;

    struct MockPSM;

    impl SequenceNumberType for MockPSM {
        type SequenceNumber = i64;
        const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber = -1;
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

    impl GUIDType<Self> for MockPSM {
        type GUID = MockGUID;
        const GUID_UNKNOWN: Self::GUID = MockGUID;
    }

    impl LocatorType for MockPSM {
        type Locator = MockLocator;
    }

    impl DurationType for MockPSM {
        type Duration = i64;
    }

    impl DataType for MockPSM {
        type Data = ();
    }

    impl ParameterIdType for MockPSM {
        type ParameterId = ();
    }

    impl ParameterListType<Self> for MockPSM {
        type ParameterList = MockParameterList;
    }

    impl InstanceHandleType for MockPSM {
        type InstanceHandle = ();
    }

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

    struct MockParameterList;

    impl ParameterList<MockPSM> for MockParameterList {
        type Parameter = MockParameter;

        fn parameter(&self) -> &[Self::Parameter] {
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

    struct MockReaderProxy;

    impl RTPSReaderProxy<MockPSM> for MockReaderProxy {
        type SequenceNumberVector = [i64; 2];

        fn remote_reader_guid(&self) -> &MockGUID {
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
