use core::iter::FromIterator;

use crate::{
    behavior::{self, RTPSWriter},
    messages::{self, Submessage},
    structure::{self, RTPSHistoryCache},
};

#[derive(Debug)]
pub struct RTPSReaderLocator<PSM: structure::Types> {
    locator: PSM::Locator,
    expects_inline_qos: bool,
    last_sent_sequence_number: PSM::SequenceNumber,
    requested_changes: PSM::SequenceNumberVector,
}

impl<PSM> Clone for RTPSReaderLocator<PSM>
where
    PSM: structure::Types,
    PSM::Locator: Clone,
{
    fn clone(&self) -> Self {
        Self {
            locator: self.locator.clone(),
            expects_inline_qos: self.expects_inline_qos,
            last_sent_sequence_number: self.last_sent_sequence_number,
            requested_changes: self.requested_changes.clone(),
        }
    }
}

impl<PSM: structure::Types> core::cmp::PartialEq for RTPSReaderLocator<PSM>
where
    PSM::Locator: core::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.locator == other.locator
    }
}

impl<PSM> RTPSReaderLocator<PSM>
where
    PSM: structure::Types + behavior::Types,
{
    pub fn new(locator: PSM::Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
            last_sent_sequence_number: 0.into(),
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
        let next_requested_change = self.requested_changes().clone().into_iter().min()?;
        self.requested_changes = self
            .requested_changes()
            .clone()
            .into_iter()
            .filter(|x| x != &next_requested_change)
            .collect();
        Some(next_requested_change)
    }

    pub fn next_unsent_change<HistoryCache: RTPSHistoryCache<PSM = PSM>>(
        &mut self,
        writer: &RTPSWriter<PSM, HistoryCache>,
    ) -> Option<PSM::SequenceNumber> {
        let unsent_changes = self.unsent_changes(writer);
        let next_unsent_change = unsent_changes.into_iter().min()?;
        self.last_sent_sequence_number = next_unsent_change;
        Some(next_unsent_change)
    }

    pub fn requested_changes(&self) -> &PSM::SequenceNumberVector {
        &self.requested_changes
    }

    pub fn requested_changes_set<'a, HistoryCache: RTPSHistoryCache<PSM = PSM>>(
        &'a mut self,
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

    pub fn unsent_changes<HistoryCache: RTPSHistoryCache<PSM = PSM>>(
        &self,
        writer: &RTPSWriter<PSM, HistoryCache>,
    ) -> PSM::SequenceNumberVector {
        let history_cache_max_seq_num: i64 = writer
            .writer_cache
            .get_seq_num_max()
            .unwrap_or(0.into())
            .into();

        (self.last_sent_sequence_number.into() + 1..=history_cache_max_seq_num)
            .map(|x| PSM::SequenceNumber::from(x))
            .filter(|seq_num| writer.writer_cache.get_change(seq_num).is_some())
            .collect()
    }
}

pub struct RTPSStatelessWriter<
    PSM: structure::Types + behavior::Types,
    HistoryCache: RTPSHistoryCache<PSM = PSM>,
    ReaderLocatorList,
> {
    writer: RTPSWriter<PSM, HistoryCache>,
    reader_locators: ReaderLocatorList,
}

impl<PSM, HistoryCache, ReaderLocatorList> RTPSStatelessWriter<PSM, HistoryCache, ReaderLocatorList>
where
    PSM: structure::Types + behavior::Types,
    HistoryCache: RTPSHistoryCache<PSM = PSM>,
    PSM::Locator: PartialEq + Clone,
    ReaderLocatorList: IntoIterator<Item = RTPSReaderLocator<PSM>>
        + Extend<RTPSReaderLocator<PSM>>
        + FromIterator<RTPSReaderLocator<PSM>>,
    for<'a> &'a ReaderLocatorList: IntoIterator<Item = &'a RTPSReaderLocator<PSM>>,
    for<'a> &'a mut ReaderLocatorList: IntoIterator<Item = &'a mut RTPSReaderLocator<PSM>>,
{
    pub fn new(writer: RTPSWriter<PSM, HistoryCache>) -> Self {
        Self {
            writer,
            reader_locators: core::iter::empty().collect(),
        }
    }

    pub fn reader_locator_add(&mut self, a_locator: <PSM as structure::Types>::Locator) {
        self.reader_locators
            .extend(Some(RTPSReaderLocator::new(a_locator, false)));
    }

    pub fn reader_locator_remove(&mut self, a_locator: &<PSM as structure::Types>::Locator) {
        self.reader_locators = (&self.reader_locators)
            .into_iter()
            .filter(|&x| x.locator() != a_locator)
            .map(|x| x.clone())
            .collect();
    }

    pub fn unsent_changes_reset(&mut self) {
        for reader_locator in &mut self.reader_locators {
            reader_locator.last_sent_sequence_number = 0.into();
        }
    }

    pub fn produce_messages<'a, Data, Gap, SendDataTo, SendGapTo>(
        &'a mut self,
        mut send_data_to: SendDataTo,
        mut send_gap_to: SendGapTo,
    ) where
        PSM: messages::Types,
        PSM::ParameterVector: Clone,
        Data: messages::submessages::Data<SerializedData = &'a <PSM as structure::types::Types>::Data>
            + Submessage<PSM = PSM>,
        Gap: messages::submessages::Gap + Submessage<PSM = PSM>,
        SendDataTo: FnMut(&PSM::Locator, Data),
        SendGapTo: FnMut(&PSM::Locator, Gap),
    {
        for reader_locator in &mut self.reader_locators {
            while reader_locator
                .unsent_changes(&self.writer)
                .into_iter()
                .next()
                .is_some()
            {
                // Pushing state
                if let Some(seq_num) = reader_locator.next_unsent_change(&self.writer) {
                    // Transition T4
                    if let Some(change) = self.writer.writer_cache.get_change(&seq_num) {
                        // Send Data submessage
                        let endianness_flag = true.into();
                        let inline_qos_flag = false.into();
                        let data_flag = true.into();
                        let key_flag = false.into();
                        let non_standard_payload_flag = false.into();
                        let reader_id = PSM::ENTITYID_UNKNOWN;
                        let writer_id = PSM::ENTITYID_UNKNOWN;
                        let writer_sn = change.sequence_number;
                        let inline_qos = change.inline_qos.clone();
                        let serialized_payload = &change.data_value;
                        let data = Data::new(
                            endianness_flag,
                            inline_qos_flag,
                            data_flag,
                            key_flag,
                            non_standard_payload_flag,
                            reader_id,
                            writer_id,
                            writer_sn,
                            inline_qos,
                            serialized_payload,
                        );
                        send_data_to(reader_locator.locator(), data)
                    } else {
                        // Send Gap submessage
                        let endianness_flag = true.into();
                        let reader_id = PSM::ENTITYID_UNKNOWN;
                        let writer_id = PSM::ENTITYID_UNKNOWN;
                        let gap_start = seq_num;
                        let gap_list = core::iter::empty().collect();
                        let gap =
                            Gap::new(endianness_flag, reader_id, writer_id, gap_start, gap_list);
                        send_gap_to(reader_locator.locator(), gap)
                    }
                }
            }
        }
        //if let Some(_change) = self.reader_locators[0].next_unsent_change(&self.writer) {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        messages::{
            self,
            submessage_elements::Parameter,
            submessages::{Data, Gap},
        },
        structure::RTPSCacheChange,
    };
    use structure::Types;

    use std::vec::Vec;
    #[derive(Clone)]
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
    #[derive(PartialEq, Clone, Debug)]
    pub struct MockLocator {
        id: i32,
    }

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
    #[derive(Debug)]
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

        const LOCATOR_INVALID: Self::Locator = MockLocator { id: -1 };

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

    struct MockHistoryCache {
        changes: Vec<RTPSCacheChange<<Self as RTPSHistoryCache>::PSM>>,
    }

    impl RTPSHistoryCache for MockHistoryCache {
        type PSM = MockPsm;

        fn new() -> Self {
            MockHistoryCache {
                changes: Vec::new(),
            }
        }

        fn add_change(&mut self, change: structure::RTPSCacheChange<Self::PSM>) {
            self.changes.push(change)
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

        fn get_seq_num_min(&self) -> Option<<Self::PSM as structure::Types>::SequenceNumber> {
            self.changes.iter().map(|x| x.sequence_number).min()
        }

        fn get_seq_num_max(&self) -> Option<<Self::PSM as structure::Types>::SequenceNumber> {
            self.changes.iter().map(|x| x.sequence_number).max()
        }
    }

    fn create_rtps_writer() -> RTPSWriter<MockPsm, MockHistoryCache> {
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
        RTPSWriter::new(
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
        )
    }

    #[derive(Debug)]
    struct MockDataSubmessage<'a> {
        serialized_payload: &'a Vec<u8>,
    }
    impl<'a> Submessage for MockDataSubmessage<'a> {
        type PSM = MockPsm;

        fn submessage_header(&self) -> messages::SubmessageHeader<Self::PSM> {
            todo!()
        }
    }

    impl<'a> Data for MockDataSubmessage<'a> {
        type SerializedData = &'a Vec<u8>;

        fn new(
            _endianness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
            _inline_qos_flag: <Self::PSM as messages::Types>::SubmessageFlag,
            _data_flag: <Self::PSM as messages::Types>::SubmessageFlag,
            _key_flag: <Self::PSM as messages::Types>::SubmessageFlag,
            _non_standard_payload_flag: <Self::PSM as messages::Types>::SubmessageFlag,
            _reader_id: <Self::PSM as structure::Types>::EntityId,
            _writer_id: <Self::PSM as structure::Types>::EntityId,
            _writer_sn: <Self::PSM as structure::Types>::SequenceNumber,
            _inline_qos: <Self::PSM as structure::Types>::ParameterVector,
            serialized_payload: Self::SerializedData,
        ) -> Self {
            Self { serialized_payload }
        }

        fn endianness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag {
            todo!()
        }

        fn inline_qos_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag {
            todo!()
        }

        fn data_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag {
            todo!()
        }

        fn key_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag {
            todo!()
        }

        fn non_standard_payload_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag {
            todo!()
        }

        fn reader_id(&self) -> &messages::submessage_elements::EntityId<Self::PSM> {
            todo!()
        }

        fn writer_id(&self) -> &messages::submessage_elements::EntityId<Self::PSM> {
            todo!()
        }

        fn writer_sn(&self) -> &messages::submessage_elements::SequenceNumber<Self::PSM> {
            todo!()
        }

        fn inline_qos(&self) -> &messages::submessage_elements::ParameterList<Self::PSM> {
            todo!()
        }

        fn serialized_payload(
            &self,
        ) -> &messages::submessage_elements::SerializedData<Self::SerializedData> {
            todo!()
        }
    }

    struct MockGapSubmessage;

    impl Submessage for MockGapSubmessage {
        type PSM = MockPsm;

        fn submessage_header(&self) -> messages::SubmessageHeader<Self::PSM> {
            todo!()
        }
    }

    impl Gap for MockGapSubmessage {
        fn new(
            _endianness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
            _reader_id: <Self::PSM as structure::Types>::EntityId,
            _writer_id: <Self::PSM as structure::Types>::EntityId,
            _gap_start: <Self::PSM as structure::Types>::SequenceNumber,
            _gap_list: <Self::PSM as structure::Types>::SequenceNumberVector,
        ) -> Self {
            todo!()
        }

        fn endianness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag {
            todo!()
        }

        fn reader_id(&self) -> &messages::submessage_elements::EntityId<Self::PSM> {
            todo!()
        }

        fn writer_id(&self) -> &messages::submessage_elements::EntityId<Self::PSM> {
            todo!()
        }

        fn gap_start(&self) -> &messages::submessage_elements::SequenceNumber<Self::PSM> {
            todo!()
        }

        fn gap_list(&self) -> &messages::submessage_elements::SequenceNumberSet<Self::PSM> {
            todo!()
        }
    }

    #[test]
    fn stateless_writer_reader_locator_add() {
        let mut stateless_writer: RTPSStatelessWriter<
            MockPsm,
            MockHistoryCache,
            Vec<RTPSReaderLocator<MockPsm>>,
        > = RTPSStatelessWriter::new(create_rtps_writer());
        let a_locator1 = MockLocator { id: 1 };
        let a_locator2 = MockLocator { id: 2 };

        stateless_writer.reader_locator_add(a_locator1);
        stateless_writer.reader_locator_add(a_locator2);

        assert_eq!(
            stateless_writer.reader_locators[0].locator(),
            &MockLocator { id: 1 }
        );
        assert_eq!(
            stateless_writer.reader_locators[1].locator(),
            &MockLocator { id: 2 }
        );
    }

    #[test]
    fn stateless_writer_reader_locator_remove() {
        let writer = create_rtps_writer();
        let reader_locator_1 = RTPSReaderLocator::new(MockLocator { id: 1 }, false);
        let reader_locator_2 = RTPSReaderLocator::new(MockLocator { id: 2 }, false);

        let mut stateless_writer: RTPSStatelessWriter<
            MockPsm,
            MockHistoryCache,
            Vec<RTPSReaderLocator<MockPsm>>,
        > = RTPSStatelessWriter {
            writer,
            reader_locators: vec![
                reader_locator_1.clone(),
                reader_locator_2.clone(),
                reader_locator_1.clone(),
            ],
        };

        stateless_writer.reader_locator_remove(&MockLocator { id: 1 });

        assert_eq!(
            stateless_writer.reader_locators,
            vec![reader_locator_2.clone()]
        );
    }

    #[test]
    fn stateless_writer_unsent_changes_reset() {
        let writer = create_rtps_writer();
        let reader_locator_1 = RTPSReaderLocator {
            locator: MockLocator { id: 1 },
            expects_inline_qos: false,
            last_sent_sequence_number: 1.into(),
            requested_changes: vec![],
        };
        let reader_locator_2 = RTPSReaderLocator {
            locator: MockLocator { id: 2 },
            expects_inline_qos: false,
            last_sent_sequence_number: 9.into(),
            requested_changes: vec![],
        };

        let mut stateless_writer: RTPSStatelessWriter<
            MockPsm,
            MockHistoryCache,
            Vec<RTPSReaderLocator<MockPsm>>,
        > = RTPSStatelessWriter {
            writer,
            reader_locators: vec![reader_locator_1, reader_locator_2],
        };

        stateless_writer.unsent_changes_reset();

        for reader_locator in stateless_writer.reader_locators {
            assert_eq!(reader_locator.last_sent_sequence_number, 0.into());
        }
    }

    #[test]
    fn reader_locator_requested_changes_set() {
        let mut reader_locator: RTPSReaderLocator<MockPsm> =
            RTPSReaderLocator::new(MockLocator { id: 0 }, false);

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
        let mut writer: RTPSWriter<MockPsm, MockHistoryCache> = RTPSWriter::new(
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

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 1,
            data_value: vec![],
            inline_qos: vec![],
        });

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 2,
            data_value: vec![],
            inline_qos: vec![],
        });

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 3,
            data_value: vec![],
            inline_qos: vec![],
        });

        let req_seq_num_set = vec![1, 2, 3];
        reader_locator.requested_changes_set(req_seq_num_set, &writer);

        let expected_requested_changes = vec![1, 2, 3];
        assert_eq!(reader_locator.requested_changes, expected_requested_changes)
    }

    #[test]
    fn reader_locator_requested_changes_set_changes_not_in_history_cache() {
        let mut reader_locator: RTPSReaderLocator<MockPsm> =
            RTPSReaderLocator::new(MockLocator { id: 0 }, false);

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
        let mut writer: RTPSWriter<MockPsm, MockHistoryCache> = RTPSWriter::new(
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

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 1,
            data_value: vec![],
            inline_qos: vec![],
        });

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 3,
            data_value: vec![],
            inline_qos: vec![],
        });

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 5,
            data_value: vec![],
            inline_qos: vec![],
        });

        let req_seq_num_set = vec![1, 2, 3, 4, 5];
        reader_locator.requested_changes_set(req_seq_num_set, &writer);

        let expected_requested_changes = vec![1, 3, 5];
        assert_eq!(reader_locator.requested_changes, expected_requested_changes);
    }

    #[test]
    fn reader_locator_next_requested_change() {
        let mut reader_locator: RTPSReaderLocator<MockPsm> =
            RTPSReaderLocator::new(MockLocator { id: 0 }, false);

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
        let mut writer: RTPSWriter<MockPsm, MockHistoryCache> = RTPSWriter::new(
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

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 1,
            data_value: vec![],
            inline_qos: vec![],
        });

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 2,
            data_value: vec![],
            inline_qos: vec![],
        });

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 3,
            data_value: vec![],
            inline_qos: vec![],
        });

        let req_seq_num_set = vec![1, 2, 3];
        reader_locator.requested_changes_set(req_seq_num_set, &writer);

        assert_eq!(reader_locator.next_requested_change(), Some(1));
        assert_eq!(reader_locator.next_requested_change(), Some(2));
        assert_eq!(reader_locator.next_requested_change(), Some(3));
        assert_eq!(reader_locator.next_requested_change(), None);
    }

    #[test]
    fn reader_locator_unsent_changes() {
        let reader_locator: RTPSReaderLocator<MockPsm> =
            RTPSReaderLocator::new(MockLocator { id: 0 }, false);

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
        let mut writer: RTPSWriter<MockPsm, MockHistoryCache> = RTPSWriter::new(
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

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 1,
            data_value: vec![],
            inline_qos: vec![],
        });

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 2,
            data_value: vec![],
            inline_qos: vec![],
        });

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 3,
            data_value: vec![],
            inline_qos: vec![],
        });

        let unsent_changes = reader_locator.unsent_changes(&writer);
        let expected_unsent_changes = vec![1, 2, 3];
        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn reader_locator_unsent_changes_with_non_consecutive_changes() {
        let reader_locator: RTPSReaderLocator<MockPsm> =
            RTPSReaderLocator::new(MockLocator { id: 0 }, false);

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
        let mut writer: RTPSWriter<MockPsm, MockHistoryCache> = RTPSWriter::new(
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

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 1,
            data_value: vec![],
            inline_qos: vec![],
        });

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 3,
            data_value: vec![],
            inline_qos: vec![],
        });

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 5,
            data_value: vec![],
            inline_qos: vec![],
        });

        let unsent_changes = reader_locator.unsent_changes(&writer);
        let expected_unsent_changes = vec![1, 3, 5];
        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn reader_locator_next_unsent_change() {
        let mut reader_locator: RTPSReaderLocator<MockPsm> =
            RTPSReaderLocator::new(MockLocator { id: 0 }, false);

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
        let mut writer: RTPSWriter<MockPsm, MockHistoryCache> = RTPSWriter::new(
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

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 1,
            data_value: vec![],
            inline_qos: vec![],
        });

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 3,
            data_value: vec![],
            inline_qos: vec![],
        });

        writer.writer_cache.add_change(RTPSCacheChange {
            kind: MockPsm::ALIVE,
            writer_guid: [1; 16],
            instance_handle: 1,
            sequence_number: 5,
            data_value: vec![],
            inline_qos: vec![],
        });

        assert_eq!(reader_locator.next_unsent_change(&writer), Some(1));
        assert_eq!(reader_locator.next_unsent_change(&writer), Some(3));
        assert_eq!(reader_locator.next_unsent_change(&writer), Some(5));
        assert_eq!(reader_locator.next_unsent_change(&writer), None);
    }

    #[test]
    fn stateless_writer_produce_messages() {
        let writer = create_rtps_writer();
        let reader_locator_1 = RTPSReaderLocator::new(MockLocator { id: 1 }, false);
        let reader_locator_2 = RTPSReaderLocator::new(MockLocator { id: 2 }, false);

        let mut stateless_writer: RTPSStatelessWriter<
            MockPsm,
            MockHistoryCache,
            Vec<RTPSReaderLocator<MockPsm>>,
        > = RTPSStatelessWriter {
            writer,
            reader_locators: vec![reader_locator_1, reader_locator_2],
        };

        stateless_writer
            .writer
            .writer_cache
            .add_change(RTPSCacheChange {
                kind: MockPsm::ALIVE,
                writer_guid: [1; 16],
                instance_handle: 1,
                sequence_number: 1,
                data_value: vec![],
                inline_qos: vec![],
            });

        stateless_writer
            .writer
            .writer_cache
            .add_change(RTPSCacheChange {
                kind: MockPsm::ALIVE,
                writer_guid: [1; 16],
                instance_handle: 1,
                sequence_number: 3,
                data_value: vec![],
                inline_qos: vec![],
            });

        {
            let mut data = Vec::new();
            let mut gap = Vec::new();
            stateless_writer.produce_messages::<MockDataSubmessage, MockGapSubmessage, _, _>(
                |locator, message| data.push((locator.clone(), message)),
                |locator, message| gap.push((locator.clone(), message)),
            );
            println!("Data: {:?}", data);
        }

        {
            let mut data = Vec::new();
            let mut gap = Vec::new();
            stateless_writer.produce_messages::<MockDataSubmessage, MockGapSubmessage, _, _>(
                |locator, message| data.push((locator.clone(), message)),
                |locator, message| gap.push((locator.clone(), message)),
            );
        }

        {
            let mut data = Vec::new();
            let mut gap = Vec::new();
            stateless_writer.produce_messages::<MockDataSubmessage, MockGapSubmessage, _, _>(
                |locator, message| data.push((locator.clone(), message)),
                |locator, message| gap.push((locator.clone(), message)),
            );
        }
    }
}
