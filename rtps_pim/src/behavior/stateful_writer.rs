use crate::structure::types::{EntityId, Locator, SequenceNumber, GUID};

pub trait RTPSReaderProxy {
    type SequenceNumberVector;

    fn remote_reader_guid(&self) -> &GUID;
    fn remote_group_entity_id(&self) -> &EntityId;
    fn unicast_locator_list(&self) -> &[Locator];
    fn multicast_locator_list(&self) -> &[Locator];
    fn expects_inline_qos(&self) -> bool;
    fn is_active(&self) -> bool;

    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber);
    fn next_requested_change(&mut self) -> Option<SequenceNumber>;
    fn next_unsent_change(&mut self) -> Option<SequenceNumber>;
    fn unsent_changes(&self) -> Self::SequenceNumberVector;
    fn requested_changes(&self) -> Self::SequenceNumberVector;
    fn requested_changes_set(&mut self, req_seq_num_set: Self::SequenceNumberVector);
    fn unacked_changes(&self) -> Self::SequenceNumberVector;
}

pub trait RTPSStatefulWriter<PSM> {
    type ReaderProxyType;

    fn matched_readers(&self) -> &[Self::ReaderProxyType];

    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType);

    fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID);

    fn matched_reader_lookup(&self, a_reader_guid: &GUID) -> Option<&Self::ReaderProxyType>;

    fn is_acked_by_all(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use crate::{
        behavior::types::DurationPIM,
        messages::submessage_elements::{
            ParameterListSubmessageElementPIM, ParameterListSubmessageElementType, ParameterType,
        },
    };

    use super::*;

    struct MockPSM;

    impl DurationPIM for MockPSM {
        type DurationType = i64;
    }

    impl ParameterListSubmessageElementPIM for MockPSM {
        type ParameterListSubmessageElementType = MockParameterList;
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

    impl ParameterType for MockParameter {
        fn parameter_id(&self) -> u16 {
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

    impl RTPSReaderProxy for MockReaderProxy {
        type SequenceNumberVector = [i64; 2];

        fn remote_reader_guid(&self) -> &GUID {
            todo!()
        }

        fn remote_group_entity_id(&self) -> &[u8; 4] {
            todo!()
        }

        fn unicast_locator_list(&self) -> &[Locator] {
            todo!()
        }

        fn multicast_locator_list(&self) -> &[Locator] {
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
