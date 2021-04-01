use crate::{
    behavior::{self, RTPSWriter},
    structure::{self, RTPSHistoryCache},
};

pub struct RTPSReaderLocator<PSM: structure::Types> {
    locator: PSM::Locator,
    expects_inline_qos: bool,
    requested_changes: PSM::SequenceNumberVector,
}

impl<PSM: structure::Types + behavior::Types> RTPSReaderLocator<PSM> {
    pub fn new(locator: PSM::Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
            requested_changes: core::iter::empty().collect(),
        }
    }

    pub fn locator(&self) -> &PSM::Locator {
        &self.locator
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    pub fn next_requested_change(&mut self) -> Option<PSM::SequenceNumber> {
        todo!()
    }

    pub fn next_unsent_change(&mut self) -> Option<PSM::SequenceNumber> {
        todo!()
    }

    pub fn requested_changes(&self) -> PSM::SequenceNumberVector {
        self.requested_changes.clone()
    }

    pub fn requested_changes_set<HistoryCache: RTPSHistoryCache<PSM = PSM>>(
        &mut self,
        req_seq_num_set: PSM::SequenceNumberVector,
        writer: &RTPSWriter<PSM, HistoryCache>,
    ) {
        self.requested_changes = self
            .requested_changes
            .clone()
            .into_iter()
            .chain(
                req_seq_num_set
                    .into_iter()
                    .filter(|seq_num| writer.writer_cache.get_change(seq_num).is_some()),
            )
            .collect();
    }

    pub fn unsent_changes(&self) -> PSM::SequenceNumberVector {
        todo!()
    }
}

// pub trait RTPSReaderLocator {
//     type PSM: RtpsPsm;
//     type HistoryCache: RTPSHistoryCache;

//     fn requested_changes(&self) -> <Self::PSM as RtpsPsm>::SequenceNumberSet;
//     fn unsent_changes(&self) -> <Self::PSM as RtpsPsm>::SequenceNumberSet;
//     fn next_requested_change(&mut self) -> Option<<Self::PSM as structure::Types>::SequenceNumber>;
//     fn next_unsent_change(&mut self) -> Option<<Self::PSM as structure::Types>::SequenceNumber>;
//     fn requested_changes_set(
//         &mut self,
//         req_seq_num_set: &[<Self::PSM as structure::Types>::SequenceNumber],
//         writer: &RTPSWriter<Self::PSM, Self::HistoryCache>,
//     );
// }

// impl dyn RTPSReaderLocator {
// fn pushing_state<
//     'a,
//     DataSubmessage: submessages::data_submessage::Data<
//         EntityId = EntityIdType,
//         SequenceNumber = SequenceNumberType,
//         ParameterId = ParameterIdType,
//         ParameterValue = ParameterValueType,
//         ParameterList = ParameterListType,
//         SerializedData = &'a [u8],
//     >,
//     GapSubmessage: submessages::gap_submessage::Gap<
//         EntityId = EntityIdType,
//         SequenceNumber = SequenceNumberType,
//         SequenceNumberList = SequenceNumberListType,
//     >,
// >(
//     &mut self,
//     the_writer: &'a RTPSWriter<
//         GuidPrefixType,
//         EntityIdType,
//         LocatorType,
//         LocatorListType,
//         DurationType,
//         SequenceNumberType,
//         InstanceHandleType,
//         DataType,
//         ParameterIdType,
//         ParameterValueType,
//         ParameterListType,
//         HistoryCacheType,
//     >,
// ) -> Option<DataSubmessage> {
//     //     // RL::can_send() is always true when this function is called
// //     // so we don't bother making an if here
// //     Self::transition_t4(reader_locator, writer)
//     self.transition_t4(the_writer)
// }

// fn transition_t4<
//     'a,
//     DataSubmessage: submessages::data_submessage::Data<
//         EntityId = EntityIdType,
//         SequenceNumber = SequenceNumberType,
//         ParameterId = ParameterIdType,
//         ParameterValue = ParameterValueType,
//         ParameterList = ParameterListType,
//         SerializedData = &'a [u8],
//     >,
//     GapSubmessage: submessages::gap_submessage::Gap<
//         EntityId = EntityIdType,
//         SequenceNumber = SequenceNumberType,
//         SequenceNumberList = SequenceNumberListType,
//     >,
// >(
//     &mut self,
//     the_writer: &'a RTPSWriter<
//         GuidPrefixType,
//         EntityIdType,
//         LocatorType,
//         LocatorListType,
//         DurationType,
//         SequenceNumberType,
//         InstanceHandleType,
//         DataType,
//         ParameterIdType,
//         ParameterValueType,
//         ParameterListType,
//         HistoryCacheType,
//     >,
// ) -> Option<DataSubmessage> {
//     if let Some(next_unsent_sequence_number) = self.next_unsent_change() {
//         if let Some(next_unsent_cache_change) = the_writer
//             .writer_cache
//             .get_change(&next_unsent_sequence_number)
//         {
//             Some(data_submessage_from_cache_change(
//                 next_unsent_cache_change,
//                 <EntityIdType as EntityId>::ENTITYID_UNKNOWN,
//             ))
//         } else {
//             let gap = GapSubmessage::new(
//                 true.into(),
//                 submessage_elements::EntityId {
//                     value: <EntityIdType as EntityId>::ENTITYID_UNKNOWN,
//                 },
//                 submessage_elements::EntityId {
//                     value: the_writer.endpoint.guid.entity_id,
//                 },
//                 submessage_elements::SequenceNumber {
//                     value: next_unsent_sequence_number,
//                 },
//                 submessage_elements::SequenceNumberSet {
//                     base: next_unsent_sequence_number,
//                     set: self.requested_changes(),
//                 },
//             );
//             todo!()
//         }
//     } else {
//         None
//     }
// }
// }

#[cfg(test)]
mod tests {
    use structure::Types;

    use crate::{messages::{self, submessage_elements::Parameter}, structure::RTPSCacheChange};

    use super::*;
    use std::vec::Vec;
    pub struct MockPsm;

    impl structure::Types for MockPsm {
        type Guid = [u8; 16];
        const GUID_UNKNOWN: Self::Guid = [0; 16];

        type GuidPrefix = [u8; 12];
        const GUIDPREFIX_UNKNOWN: Self::GuidPrefix = [0; 12];

        type EntityId = [u8; 4];
        const ENTITYID_UNKNOWN: Self::EntityId = [0; 4];

        type SequenceNumber = i64;
        const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber = i64::MIN;

        type Locator = MockLocator;

        const LOCATOR_INVALID: Self::Locator = MockLocator;

        type TopicKind = u8;
        const NO_KEY: Self::TopicKind = 0;
        const WITH_KEY: Self::TopicKind = 1;

        type ChangeKind = u8;

        const ALIVE: Self::ChangeKind = 0;
        const ALIVE_FILTERED: Self::ChangeKind = 1;
        const NOT_ALIVE_DISPOSED: Self::ChangeKind = 2;
        const NOT_ALIVE_UNREGISTERED: Self::ChangeKind = 3;

        type ReliabilityKind = u8;
        const BEST_EFFORT: Self::ReliabilityKind = 0;
        const RELIABLE: Self::ReliabilityKind = 1;

        type InstanceHandle = u32;

        type ProtocolVersion = [u8; 2];

        const PROTOCOLVERSION: Self::ProtocolVersion = [2, 4];
        const PROTOCOLVERSION_1_0: Self::ProtocolVersion = [1, 0];
        const PROTOCOLVERSION_1_1: Self::ProtocolVersion = [1, 1];
        const PROTOCOLVERSION_2_0: Self::ProtocolVersion = [2, 0];
        const PROTOCOLVERSION_2_1: Self::ProtocolVersion = [2, 1];
        const PROTOCOLVERSION_2_2: Self::ProtocolVersion = [2, 2];
        const PROTOCOLVERSION_2_3: Self::ProtocolVersion = [2, 3];
        const PROTOCOLVERSION_2_4: Self::ProtocolVersion = [2, 4];

        type VendorId = i8;
        const VENDOR_ID_UNKNOWN: Self::VendorId = -1;

        type Data = Vec<u8>;
        type LocatorVector = Vec<<Self as structure::Types>::Locator>;

        type SequenceNumberVector = Vec<<Self as structure::Types>::SequenceNumber>;

        type Parameter = MockParameter;
        type ParameterVector = Vec<Self::Parameter>;
    }

    impl messages::Types for MockPsm {
        type ProtocolId = [u8; 4];

        const PROTOCOL_RTPS: Self::ProtocolId = [b'R', b'T', b'P', b'S'];

        type SubmessageFlag = bool;

        type SubmessageKind = u8;
        const DATA: Self::SubmessageKind = 0;
        const GAP: Self::SubmessageKind = 1;
        const HEARTBEAT: Self::SubmessageKind = 2;
        const ACKNACK: Self::SubmessageKind = 3;
        const PAD: Self::SubmessageKind = 4;
        const INFO_TS: Self::SubmessageKind = 5;
        const INFO_REPLY: Self::SubmessageKind = 6;
        const INFO_DST: Self::SubmessageKind = 7;
        const INFO_SRC: Self::SubmessageKind = 8;
        const DATA_FRAG: Self::SubmessageKind = 9;
        const NACK_FRAG: Self::SubmessageKind = 10;
        const HEARTBEAT_FRAG: Self::SubmessageKind = 11;

        type Time = u64;
        const TIME_ZERO: Self::Time = 0;
        const TIME_INVALID: Self::Time = u64::MIN;
        const TIME_INFINITE: Self::Time = u64::MAX;

        type Count = u32;
        type ParameterId = u8;
        type FragmentNumber = i32;
        type GroupDigest = i32;

        type FragmentNumberVector = Vec<<Self as messages::Types>::FragmentNumber>;
    }

    impl behavior::Types for MockPsm {
        type Duration = i64;

        type ChangeForReaderStatusKind = u8;
        const UNSENT: Self::ChangeForReaderStatusKind = 0;
        const UNACKNOWLEDGED: Self::ChangeForReaderStatusKind = 1;
        const REQUESTED: Self::ChangeForReaderStatusKind = 2;
        const ACKNOWLEDGED: Self::ChangeForReaderStatusKind = 3;
        const UNDERWAY: Self::ChangeForReaderStatusKind = 4;

        type ChangeFromWriterStatusKind = u8;
        const LOST: Self::ChangeFromWriterStatusKind = 0;
        const MISSING: Self::ChangeFromWriterStatusKind = 1;
        const RECEIVED: Self::ChangeFromWriterStatusKind = 2;
        const UNKNOWN: Self::ChangeFromWriterStatusKind = 3;

        type ParticipantMessageData = u8;
    }

    pub struct MockLocator;

    impl structure::types::Locator for MockLocator {
        type Kind = i32;
        type Port = u32;
        type Address = [u8; 16];

        const LOCATOR_KIND_INVALID: Self::Kind = -1;
        const LOCATOR_KIND_RESERVED: Self::Kind = 0;
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv4: Self::Kind = 1;
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv6: Self::Kind = 2;
        const LOCATOR_ADDRESS_INVALID: Self::Address = [0; 16];
        const LOCATOR_PORT_INVALID: Self::Port = 0;
    }

    pub struct MockParameter;

    impl Parameter for MockParameter {
        type PSM = MockPsm;

        fn parameter_id(&self) -> <Self::PSM as messages::Types>::ParameterId {
            todo!()
        }

        fn length(&self) -> i16 {
            todo!()
        }

        fn value(&self) -> &[u8] {
            todo!()
        }
    }

    struct MockHistoryCache{
        changes: Vec<RTPSCacheChange<<Self as RTPSHistoryCache>::PSM>>,
    }

    impl RTPSHistoryCache for MockHistoryCache {
        type PSM = MockPsm;

        fn new() -> Self {
            MockHistoryCache {
                changes: Vec::new(),
            }
        }

        fn add_change(&mut self, _change: structure::RTPSCacheChange<Self::PSM>) {
            todo!()
        }

        fn remove_change(&mut self, _seq_num: &<Self::PSM as structure::Types>::SequenceNumber) {
            todo!()
        }

        fn get_change(
            &self,
            seq_num: &<Self::PSM as structure::Types>::SequenceNumber,
        ) -> Option<&structure::RTPSCacheChange<Self::PSM>> {
            self.changes.iter().find(|x| &x.sequence_number == seq_num)
        }

        fn get_seq_num_min(&self) -> Option<&<Self::PSM as structure::Types>::SequenceNumber> {
            todo!()
        }

        fn get_seq_num_max(&self) -> Option<&<Self::PSM as structure::Types>::SequenceNumber> {
            todo!()
        }
    }

    #[test]
    fn requested_changes_set() {
        let mut reader_locator: RTPSReaderLocator<MockPsm> =
            RTPSReaderLocator::new(MockLocator, false);

        let guid = [1; 16];
        let topic_kind = MockPsm::WITH_KEY;
        let reliability_level = MockPsm::BEST_EFFORT;
        let unicast_locator_list = Vec::new();
        let multicast_locator_list = Vec::new();
        let push_mode = true;
        let heartbeat_period = 0;
        let nack_response_delay = 0;
        let nack_suppression_duration = 0;
        let data_max_size_serialized = 65535;
        let writer: RTPSWriter<MockPsm, MockHistoryCache> = RTPSWriter::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );

        let req_seq_num_set = vec![1, 3, 5];
        reader_locator.requested_changes_set(req_seq_num_set.clone(), &writer);

        assert_eq!(reader_locator.requested_changes, req_seq_num_set)
    }
}
