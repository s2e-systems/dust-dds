use crate::{
    behavior::RTPSWriter,
    messages::submessages::{Data, Gap},
    structure::{types::Locator, RTPSHistoryCache},
    PIM,
};

pub struct RTPSReaderLocator<PSM: PIM> {
    locator: Locator<PSM>,
    expects_inline_qos: bool,
    last_sent_sequence_number: PSM::SequenceNumber,
    requested_changes: PSM::SequenceNumberVector,
}

impl<PSM: PIM> Clone for RTPSReaderLocator<PSM> {
    fn clone(&self) -> Self {
        Self {
            locator: self.locator.clone(),
            expects_inline_qos: self.expects_inline_qos,
            last_sent_sequence_number: self.last_sent_sequence_number,
            requested_changes: self.requested_changes.clone(),
        }
    }
}

impl<PSM: PIM> core::cmp::PartialEq for RTPSReaderLocator<PSM> {
    fn eq(&self, other: &Self) -> bool {
        self.locator == other.locator
    }
}

impl<PSM: PIM> RTPSReaderLocator<PSM> {
    pub fn new(locator: Locator<PSM>, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
            last_sent_sequence_number: 0.into(),
            requested_changes: core::iter::empty().collect(),
        }
    }

    pub fn locator(&self) -> &Locator<PSM> {
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

    pub fn next_unsent_change(
        &mut self,
        writer_cache: &dyn RTPSHistoryCache<PSM>,
    ) -> Option<PSM::SequenceNumber> {
        let unsent_changes = self.unsent_changes(writer_cache);
        let next_unsent_change = unsent_changes.into_iter().min()?;
        self.last_sent_sequence_number = next_unsent_change;
        Some(next_unsent_change)
    }

    pub fn requested_changes(&self) -> &PSM::SequenceNumberVector {
        &self.requested_changes
    }

    pub fn requested_changes_set<'a>(
        &'a mut self,
        req_seq_num_set: PSM::SequenceNumberVector,
        writer_cache: &dyn RTPSHistoryCache<PSM>,
    ) {
        self.requested_changes = self
            .requested_changes
            .clone()
            .into_iter()
            .chain(
                req_seq_num_set
                    .into_iter()
                    .filter(|seq_num| writer_cache.get_change(seq_num).is_some()),
            )
            .collect();
    }

    pub fn unsent_changes(
        &self,
        writer_cache: &dyn RTPSHistoryCache<PSM>,
    ) -> PSM::SequenceNumberVector {
        let history_cache_max_seq_num: i64 =
            writer_cache.get_seq_num_max().unwrap_or(0.into()).into();

        (self.last_sent_sequence_number.into() + 1..=history_cache_max_seq_num)
            .map(|x| PSM::SequenceNumber::from(x))
            .filter(|seq_num| writer_cache.get_change(seq_num).is_some())
            .collect()
    }
}

pub trait RTPSStatelessWriter<PSM: PIM>: RTPSWriter<PSM> {
    fn reader_locator_add(&mut self, a_locator: Locator<PSM>);

    fn reader_locator_remove(&mut self, a_locator: &Locator<PSM>);

    fn reader_locators(&mut self) -> &mut [RTPSReaderLocator<PSM>];

    fn unsent_changes_reset(&mut self) {
        for reader_locator in self.reader_locators() {
            reader_locator.last_sent_sequence_number = 0.into();
        }
    }
}

impl<'a, PSM: PIM> RTPSReaderLocator<PSM> {
    pub fn produce_messages<SendDataTo, SendGapTo>(
        &'a mut self,
        writer_cache: &'a dyn RTPSHistoryCache<PSM>,
        send_data_to: &mut SendDataTo,
        send_gap_to: &mut SendGapTo,
    ) where
        SendDataTo: FnMut(&Locator<PSM>, Data<PSM, &'a PSM::Data>),
        SendGapTo: FnMut(&Locator<PSM>, Gap<PSM>),
    {
        while self
            .unsent_changes(writer_cache)
            .into_iter()
            .next()
            .is_some()
        {
            // Pushing state
            if let Some(seq_num) = self.next_unsent_change(writer_cache) {
                // Transition T4
                if let Some(change) = writer_cache.get_change(&seq_num) {
                    // Send Data submessage
                    let endianness_flag = true.into();
                    let inline_qos_flag = false.into();
                    let data_flag = true.into();
                    let key_flag = false.into();
                    let non_standard_payload_flag = false.into();
                    let reader_id = PSM::ENTITYID_UNKNOWN;
                    let writer_id = PSM::ENTITYID_UNKNOWN;
                    let writer_sn = change.sequence_number;
                    // let inline_qos = change.inline_qos.clone();
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
                        // inline_qos,
                        serialized_payload,
                    );
                    send_data_to(self.locator(), data)
                } else {
                    // Send Gap submessage
                    let endianness_flag = true.into();
                    let reader_id = PSM::ENTITYID_UNKNOWN;
                    let writer_id = PSM::ENTITYID_UNKNOWN;
                    let gap_start = seq_num;
                    let gap_list = core::iter::empty().collect();
                    let gap = Gap::new(endianness_flag, reader_id, writer_id, gap_start, gap_list);
                    send_gap_to(self.locator(), gap)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        behavior,
        messages::{self, submessage_elements::Parameter},
        structure::{
            self,
            types::{ChangeKind, ReliabilityKind, TopicKind, GUID},
            RTPSCacheChange, RTPSEndpoint, RTPSEntity, RTPSHistoryCache,
        },
    };
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

    #[derive(Debug)]
    pub struct MockPsm;

    impl structure::Types for MockPsm {
        type GuidPrefix = [u8; 12];
        const GUIDPREFIX_UNKNOWN: Self::GuidPrefix = [0; 12];

        type EntityId = [u8; 4];
        const ENTITYID_UNKNOWN: Self::EntityId = [0; 4];
        const ENTITYID_PARTICIPANT: Self::EntityId = [0, 0, 0, 1];

        type SequenceNumber = i64;
        const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber = i64::MIN;

        type LocatorKind = i32;
        type LocatorPort = u32;
        type LocatorAddress = [u8; 16];

        const LOCATOR_KIND_INVALID: Self::LocatorKind = -1;
        const LOCATOR_KIND_RESERVED: Self::LocatorKind = 0;
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv4: Self::LocatorKind = 1;
        #[allow(non_upper_case_globals)]
        const LOCATOR_KIND_UDPv6: Self::LocatorKind = 2;
        const LOCATOR_ADDRESS_INVALID: Self::LocatorAddress = [0; 16];
        const LOCATOR_PORT_INVALID: Self::LocatorPort = 0;

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
        type Locator = crate::structure::types::Locator<MockPsm>;
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

        type ParticipantMessageData = u8;
    }

    impl PIM for MockPsm {}

    struct MockHistoryCache {
        changes: Vec<RTPSCacheChange<MockPsm>>,
    }

    impl RTPSHistoryCache<MockPsm> for MockHistoryCache {
        fn new() -> Self {
            MockHistoryCache {
                changes: Vec::new(),
            }
        }

        fn add_change(&mut self, change: structure::RTPSCacheChange<MockPsm>) {
            self.changes.push(change)
        }

        fn remove_change(&mut self, _seq_num: &<MockPsm as structure::Types>::SequenceNumber) {
            todo!()
        }

        fn get_change(
            &self,
            seq_num: &<MockPsm as structure::Types>::SequenceNumber,
        ) -> Option<&structure::RTPSCacheChange<MockPsm>> {
            self.changes.iter().find(|x| &x.sequence_number == seq_num)
        }

        fn get_seq_num_min(&self) -> Option<<MockPsm as structure::Types>::SequenceNumber> {
            self.changes.iter().map(|x| x.sequence_number).min()
        }

        fn get_seq_num_max(&self) -> Option<<MockPsm as structure::Types>::SequenceNumber> {
            self.changes.iter().map(|x| x.sequence_number).max()
        }
    }

    pub struct MockWriter {
        history_cache: MockHistoryCache,
    }

    impl MockWriter {
        fn new() -> Self {
            Self {
                history_cache: MockHistoryCache::new(),
            }
        }
    }

    impl RTPSEntity<MockPsm> for MockWriter {
        fn guid(&self) -> GUID<MockPsm> {
            todo!()
        }
    }

    impl RTPSEndpoint<MockPsm> for MockWriter {
        fn topic_kind(&self) -> TopicKind {
            todo!()
        }

        fn reliability_level(&self) -> ReliabilityKind {
            todo!()
        }

        fn unicast_locator_list(&self) -> &[Locator<MockPsm>] {
            todo!()
        }

        fn multicast_locator_list(&self) -> &[Locator<MockPsm>] {
            todo!()
        }
    }

    impl RTPSWriter<MockPsm> for MockWriter {
        fn push_mode(&self) -> bool {
            todo!()
        }

        fn heartbeat_period(&self) -> i64 {
            todo!()
        }

        fn nack_response_delay(&self) -> i64 {
            todo!()
        }

        fn nack_suppression_duration(&self) -> i64 {
            todo!()
        }

        fn last_change_sequence_number(&self) -> i64 {
            todo!()
        }

        fn data_max_size_serialized(&self) -> i32 {
            todo!()
        }

        fn writer_cache(&self) -> &dyn RTPSHistoryCache<MockPsm> {
            &self.history_cache
        }

        fn writer_cache_mut(&mut self) -> &mut dyn RTPSHistoryCache<MockPsm> {
            &mut self.history_cache
        }

        fn new_change(
            &mut self,
            _kind: ChangeKind,
            _data: <MockPsm as structure::Types>::Data,
            _inline_qos: &[<MockPsm as structure::Types>::Parameter],
            _handle: <MockPsm as structure::Types>::InstanceHandle,
        ) -> RTPSCacheChange<MockPsm> {
            todo!()
        }
    }

    // fn create_rtps_writer() -> RTPSWriter<MockPsm, MockHistoryCache> {
    //     let guid = GUID::new([1; 12], [1; 4]);
    //     let topic_kind = TopicKind::WithKey;
    //     let reliability_level = ReliabilityKind::BestEffort;
    //     let unicast_locator_list = Vec::new();
    //     let multicast_locator_list = Vec::new();
    //     let push_mode = true;
    //     let heartbeat_period = 0;
    //     let nack_response_delay = 0;
    //     let nack_suppression_duration = 0;
    //     let data_max_size_serialized = 65535;
    //     RTPSWriter::new(
    //         guid,
    //         topic_kind,
    //         reliability_level,
    //         unicast_locator_list,
    //         multicast_locator_list,
    //         push_mode,
    //         heartbeat_period,
    //         nack_response_delay,
    //         nack_suppression_duration,
    //         data_max_size_serialized,
    //     )
    // }

    // #[test]
    // fn stateless_writer_unsent_changes_reset() {
    //     let writer = create_rtps_writer();
    //     let reader_locator_1 = RTPSReaderLocator {
    //         locator: MockLocator { id: 1 },
    //         expects_inline_qos: false,
    //         last_sent_sequence_number: 1.into(),
    //         requested_changes: vec![],
    //     };
    //     let reader_locator_2 = RTPSReaderLocator {
    //         locator: MockLocator { id: 2 },
    //         expects_inline_qos: false,
    //         last_sent_sequence_number: 9.into(),
    //         requested_changes: vec![],
    //     };

    //     let mut stateless_writer: RTPSStatelessWriter<
    //         MockPsm,
    //         MockHistoryCache,
    //         Vec<RTPSReaderLocator<MockPsm>>,
    //     > = RTPSStatelessWriter {
    //         writer,
    //         reader_locators: vec![reader_locator_1, reader_locator_2],
    //     };

    //     stateless_writer.unsent_changes_reset();

    //     for reader_locator in stateless_writer.reader_locators {
    //         assert_eq!(reader_locator.last_sent_sequence_number, 0.into());
    //     }
    // }

    #[test]
    fn reader_locator_requested_changes_set() {
        let mut reader_locator = RTPSReaderLocator::new(Locator::new(0, 0, [0; 16]), false);

        let mut writer = MockWriter::new();

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 1,
            data_value: vec![],
            // inline_qos: vec![],
        });

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 2,
            data_value: vec![],
            // inline_qos: vec![],
        });

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 3,
            data_value: vec![],
            // inline_qos: vec![],
        });

        let req_seq_num_set = vec![1, 2, 3];
        reader_locator.requested_changes_set(req_seq_num_set, writer.writer_cache());

        let expected_requested_changes = vec![1, 2, 3];
        assert_eq!(reader_locator.requested_changes, expected_requested_changes)
    }

    #[test]
    fn reader_locator_requested_changes_set_changes_not_in_history_cache() {
        let mut reader_locator = RTPSReaderLocator::new(Locator::new(0, 0, [0; 16]), false);

        let mut writer = MockWriter::new();

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 1,
            data_value: vec![],
            // inline_qos: vec![],
        });

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 3,
            data_value: vec![],
            // inline_qos: vec![],
        });

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 5,
            data_value: vec![],
            // inline_qos: vec![],
        });

        let req_seq_num_set = vec![1, 2, 3, 4, 5];
        reader_locator.requested_changes_set(req_seq_num_set, writer.writer_cache());

        let expected_requested_changes = vec![1, 3, 5];
        assert_eq!(reader_locator.requested_changes, expected_requested_changes);
    }

    #[test]
    fn reader_locator_next_requested_change() {
        let mut reader_locator = RTPSReaderLocator::new(Locator::new(0, 0, [0; 16]), false);

        let mut writer = MockWriter::new();

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 1,
            data_value: vec![],
            // inline_qos: vec![],
        });

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 2,
            data_value: vec![],
            // inline_qos: vec![],
        });

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 3,
            data_value: vec![],
            // inline_qos: vec![],
        });

        let req_seq_num_set = vec![1, 2, 3];
        reader_locator.requested_changes_set(req_seq_num_set, writer.writer_cache());

        assert_eq!(reader_locator.next_requested_change(), Some(1));
        assert_eq!(reader_locator.next_requested_change(), Some(2));
        assert_eq!(reader_locator.next_requested_change(), Some(3));
        assert_eq!(reader_locator.next_requested_change(), None);
    }

    #[test]
    fn reader_locator_unsent_changes() {
        let reader_locator: RTPSReaderLocator<MockPsm> =
            RTPSReaderLocator::new(Locator::new(0, 0, [0; 16]), false);

        let mut writer = MockWriter::new();

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 1,
            data_value: vec![],
            // inline_qos: vec![],
        });

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 2,
            data_value: vec![],
            // inline_qos: vec![],
        });

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 3,
            data_value: vec![],
            // inline_qos: vec![],
        });

        let unsent_changes = reader_locator.unsent_changes(writer.writer_cache());
        let expected_unsent_changes = vec![1, 2, 3];
        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn reader_locator_unsent_changes_with_non_consecutive_changes() {
        let reader_locator = RTPSReaderLocator::new(Locator::new(0, 0, [0; 16]), false);

        let mut writer = MockWriter::new();

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 1,
            data_value: vec![],
            // inline_qos: vec![],
        });

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 3,
            data_value: vec![],
            // inline_qos: vec![],
        });

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 5,
            data_value: vec![],
            // inline_qos: vec![],
        });

        let unsent_changes = reader_locator.unsent_changes(writer.writer_cache());
        let expected_unsent_changes = vec![1, 3, 5];
        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn reader_locator_next_unsent_change() {
        let mut reader_locator = RTPSReaderLocator::new(Locator::new(0, 0, [0; 16]), false);

        let mut writer = MockWriter::new();

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 1,
            data_value: vec![],
            // inline_qos: vec![],
        });

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 3,
            data_value: vec![],
            // inline_qos: vec![],
        });

        writer.writer_cache_mut().add_change(RTPSCacheChange {
            kind: ChangeKind::Alive,
            writer_guid: GUID::new([1; 12], [1; 4]),
            instance_handle: 1,
            sequence_number: 5,
            data_value: vec![],
            // inline_qos: vec![],
        });

        assert_eq!(
            reader_locator.next_unsent_change(writer.writer_cache()),
            Some(1)
        );
        assert_eq!(
            reader_locator.next_unsent_change(writer.writer_cache()),
            Some(3)
        );
        assert_eq!(
            reader_locator.next_unsent_change(writer.writer_cache()),
            Some(5)
        );
        assert_eq!(
            reader_locator.next_unsent_change(writer.writer_cache()),
            None
        );
    }

    // #[test]
    // fn stateless_writer_produce_messages() {
    //     let mut writer = MockWriter::new();
    //     let mut reader_locator_1 = RTPSReaderLocator::new(Locator::new(1, 0, [0; 16]), false);
    //     let mut reader_locator_2 = RTPSReaderLocator::new(Locator::new(2, 0, [0; 16]), false);

    //     writer.writer_cache_mut().add_change(RTPSCacheChange {
    //         kind: ChangeKind::Alive,
    //         writer_guid: GUID::new([1; 12], [1; 4]),
    //         instance_handle: 1,
    //         sequence_number: 1,
    //         data_value: vec![],
    //         inline_qos: vec![],
    //     });

    //     writer.writer_cache_mut().add_change(RTPSCacheChange {
    //         kind: ChangeKind::Alive,
    //         writer_guid: GUID::new([1; 12], [1; 4]),
    //         instance_handle: 1,
    //         sequence_number: 3,
    //         data_value: vec![],
    //         inline_qos: vec![],
    //     });

    //     {
    //         let mut data: Vec<(Locator<MockPsm>, MockDataSubmessage)> = Vec::new();
    //         let mut gap: Vec<(Locator<MockPsm>, MockGapSubmessage)> = Vec::new();
    //         reader_locator_1
    //             .produce_messages::<MockDataSubmessage, MockGapSubmessage, MockHistoryCache, _, _>(
    //                 &writer,
    //                 &mut |locator, message| data.push((locator.clone(), message)),
    //                 &mut |locator, message| gap.push((locator.clone(), message)),
    //             );
    //         println!("Data: {:?}", data[0].1);
    //     }

    //     {
    //         let mut data = Vec::new();
    //         let mut gap = Vec::new();
    //         reader_locator_2
    //             .produce_messages::<MockDataSubmessage, MockGapSubmessage, MockHistoryCache, _, _>(
    //                 &writer,
    //                 &mut |locator, message| data.push((locator.clone(), message)),
    //                 &mut |locator, message| gap.push((locator.clone(), message)),
    //             );
    //     }

    //     {
    //         let mut data = Vec::new();
    //         let mut gap = Vec::new();
    //         reader_locator_1
    //             .produce_messages::<MockDataSubmessage, MockGapSubmessage, MockHistoryCache, _, _>(
    //                 &writer,
    //                 &mut |locator, message| data.push((locator.clone(), message)),
    //                 &mut |locator, message| gap.push((locator.clone(), message)),
    //             );
    //     }
    // }
}
