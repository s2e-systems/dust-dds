use std::marker::PhantomData;

use rust_rtps::{
    behavior::{
        stateful_writer::reader_proxy::RTPSChangeForReader, types::ChangeForReaderStatusKind,
        RTPSReaderProxy, RTPSWriter,
    },
    structure::RTPSHistoryCache,
    types::{EntityId, Locator, SequenceNumber, GUID},
};

pub struct ReaderProxy<
    'a,
    T: RTPSChangeForReader<CacheChangeRepresentation = SequenceNumber>,
    W: RTPSWriter<'a>,
> {
    remote_reader_guid: GUID,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    expects_inline_qos: bool,
    is_active: bool,

    phantom: PhantomData<T>,
    writer: &'a W,
    next_unsent_change: SequenceNumber,
    highest_acked_change: SequenceNumber,
    requested_changes: Vec<SequenceNumber>,
}

impl<'a, T: RTPSChangeForReader<CacheChangeRepresentation = SequenceNumber>, W: RTPSWriter<'a>>
    RTPSReaderProxy<'a> for ReaderProxy<'a, T, W>
{
    type ChangeForReaderType = T;
    type ChangeForReaderTypeList = Vec<Self::ChangeForReaderType>;
    type Writer = W;
    fn remote_reader_guid(&self) -> GUID {
        self.remote_reader_guid
    }

    fn remote_group_entity_id(&self) -> EntityId {
        self.remote_group_entity_id
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.multicast_locator_list
    }

    fn changes_for_reader(&self) -> Self::ChangeForReaderTypeList {
        todo!()
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn writer(&self) -> &Self::Writer {
        todo!()
    }

    fn new(
        remote_reader_guid: GUID,
        remote_group_entity_id: EntityId,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        expects_inline_qos: bool,
        is_active: bool,
        writer: &'a Self::Writer,
    ) -> Self {
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
            expects_inline_qos,
            is_active,
            phantom: PhantomData,
            writer,
            next_unsent_change: 0,
            highest_acked_change: 0,
            requested_changes: Vec::new(),
        }
    }

    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        self.highest_acked_change = committed_seq_num;
    }

    fn next_requested_change(&mut self) -> Option<Self::ChangeForReaderType> {
        let next_requested_change = *self.requested_changes.iter().min()?;
        self.requested_changes
            .retain(|x| x != &next_requested_change);
        Some(T::new(
            next_requested_change,
            ChangeForReaderStatusKind::Requested,
            true,
        ))
    }

    fn next_unsent_change(&mut self) -> Option<Self::ChangeForReaderType> {
        self.next_unsent_change = self.unsent_changes().iter().map(|x| x.change()).min()?;
        Some(T::new(
            self.next_unsent_change,
            ChangeForReaderStatusKind::Unsent,
            true,
        ))
    }

    fn unsent_changes(&self) -> Self::ChangeForReaderTypeList {
        if self.writer.push_mode() == true {
            let max_history_cache_seq_num =
                self.writer.writer_cache().get_seq_num_max().unwrap_or(0);
            (self.next_unsent_change + 1..=max_history_cache_seq_num)
                .map(|sn| T::new(sn, ChangeForReaderStatusKind::Unsent, true))
                .collect()
        } else {
            // If writer push_mode is false no change is unsent since they have to be
            // explicitly requested by the receiver using acknack
            Vec::new()
        }
    }

    fn requested_changes(&self) -> Self::ChangeForReaderTypeList {
        self.requested_changes
            .iter()
            .map(|sn| T::new(*sn, ChangeForReaderStatusKind::Requested, true))
            .collect()
    }

    fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        for value in req_seq_num_set {
            if value
                <= &self
                    .writer
                    .writer_cache()
                    .get_seq_num_max()
                    .unwrap_or_default()
            {
                self.requested_changes.push(*value);
            }
        }
    }

    fn unacked_changes(&self) -> Self::ChangeForReaderTypeList {
        if self.writer.push_mode() == true {
            // According to the diagram in page 8.4.9.3 this is every change that has been sent
            // longer ago than writer.nackSuppressionDuration() and not yet acknowledged
            // TODO: nackSuppressionDuration is for now hard-coded 0
            (self.highest_acked_change + 1..=self.next_unsent_change)
                .map(|sn| T::new(sn, ChangeForReaderStatusKind::Unacknowledged, true))
                .collect()
        } else {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps::{
        messages::submessages::submessage_elements::ParameterList,
        structure::{RTPSCacheChange, RTPSEndpoint, RTPSEntity},
    };

    use super::*;

    struct MockCacheChange;

    impl RTPSCacheChange for MockCacheChange {
        type Data = ();

        fn new(
            _kind: rust_rtps::types::ChangeKind,
            _writer_guid: GUID,
            _instance_handle: rust_rtps::types::InstanceHandle,
            _sequence_number: SequenceNumber,
            _data_value: Self::Data,
            _inline_qos: rust_rtps::messages::submessages::submessage_elements::ParameterList,
        ) -> Self {
            todo!()
        }

        fn kind(&self) -> rust_rtps::types::ChangeKind {
            todo!()
        }

        fn writer_guid(&self) -> GUID {
            todo!()
        }

        fn instance_handle(&self) -> &rust_rtps::types::InstanceHandle {
            todo!()
        }

        fn sequence_number(&self) -> SequenceNumber {
            todo!()
        }

        fn data_value(&self) -> &Self::Data {
            todo!()
        }

        fn inline_qos(
            &self,
        ) -> &rust_rtps::messages::submessages::submessage_elements::ParameterList {
            todo!()
        }
    }
    struct MockHistoryCache {
        seq_num_max: Option<SequenceNumber>,
    }

    impl<'a> RTPSHistoryCache<'a> for MockHistoryCache {
        type CacheChangeType = MockCacheChange;
        type CacheChangeReadType = Box<MockCacheChange>;

        fn new() -> Self {
            todo!()
        }

        fn add_change(&self, _change: Self::CacheChangeType) {
            todo!()
        }

        fn remove_change(&self, _seq_num: SequenceNumber) {
            todo!()
        }

        fn get_change(&self, _seq_num: SequenceNumber) -> Option<Self::CacheChangeReadType> {
            todo!()
        }

        fn get_seq_num_min(&self) -> Option<SequenceNumber> {
            todo!()
        }

        fn get_seq_num_max(&self) -> Option<SequenceNumber> {
            self.seq_num_max
        }
    }
    struct MockWriter {
        push_mode: bool,
        writer_cache: MockHistoryCache,
    }

    impl RTPSEntity for MockWriter {
        fn guid(&self) -> GUID {
            todo!()
        }
    }
    impl RTPSEndpoint for MockWriter {
        fn unicast_locator_list(&self) -> &[Locator] {
            todo!()
        }

        fn multicast_locator_list(&self) -> &[Locator] {
            todo!()
        }

        fn topic_kind(&self) -> rust_rtps::types::TopicKind {
            todo!()
        }

        fn reliability_level(&self) -> rust_rtps::types::ReliabilityKind {
            todo!()
        }
    }
    impl<'a> RTPSWriter<'a> for MockWriter {
        type HistoryCacheType = MockHistoryCache;

        fn new(
            _guid: GUID,
            _topic_kind: rust_rtps::types::TopicKind,
            _reliablility_level: rust_rtps::types::ReliabilityKind,
            _unicast_locator_list: &[Locator],
            _multicast_locator_list: &[Locator],
            _push_mode: bool,
            _heartbeat_period: rust_rtps::behavior::types::Duration,
            _nack_response_delay: rust_rtps::behavior::types::Duration,
            _nack_suppression_duration: rust_rtps::behavior::types::Duration,
            _data_max_sized_serialized: i32,
            _writer_cache: Self::HistoryCacheType,
        ) -> Self {
            todo!()
        }

        fn push_mode(&self) -> bool {
            self.push_mode
        }

        fn heartbeat_period(&self) -> rust_rtps::behavior::types::Duration {
            todo!()
        }

        fn nack_response_delay(&self) -> rust_rtps::behavior::types::Duration {
            todo!()
        }

        fn nack_suppression_duration(&self) -> rust_rtps::behavior::types::Duration {
            todo!()
        }

        fn last_change_sequence_number(&self) -> SequenceNumber {
            todo!()
        }

        fn data_max_sized_serialized(&self) -> i32 {
            todo!()
        }

        fn writer_cache(&self) -> &Self::HistoryCacheType {
            &self.writer_cache
        }

        fn new_change(
            &self,
            _kind: rust_rtps::types::ChangeKind,
            _data: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType as RTPSCacheChange>::Data,
            _inline_qos: rust_rtps::messages::submessages::submessage_elements::ParameterList,
            _handle: rust_rtps::types::InstanceHandle,
        ) -> <Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType {
            todo!()
        }
    }

    #[derive(Debug, PartialEq)]
    struct MockChangeForReader {
        change: SequenceNumber,
        status: ChangeForReaderStatusKind,
        is_relevant: bool,
    }

    impl RTPSChangeForReader for MockChangeForReader {
        type CacheChangeRepresentation = SequenceNumber;

        fn new(
            change: Self::CacheChangeRepresentation,
            status: ChangeForReaderStatusKind,
            is_relevant: bool,
        ) -> Self {
            Self {
                change,
                status,
                is_relevant,
            }
        }

        fn change(&self) -> Self::CacheChangeRepresentation {
            self.change
        }

        fn status(&self) -> ChangeForReaderStatusKind {
            self.status
        }

        fn is_relevant(&self) -> bool {
            self.is_relevant
        }
    }

    #[test]
    fn new_and_getters() {
        let remote_reader_guid = GUID::new([5; 12], EntityId::new([5, 6, 7], 1));
        let remote_group_entity_id = EntityId::new([1, 2, 3], 10);
        let unicast_locator_list = [Locator::new(20, 200, [1; 16])];
        let multicast_locator_list = [Locator::new(10, 100, [2; 16])];
        let expects_inline_qos = false;
        let is_active = true;
        let writer = MockWriter {
            push_mode: true,
            writer_cache: MockHistoryCache { seq_num_max: None },
        };
        let reader_proxy: ReaderProxy<MockChangeForReader, _> = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            &writer,
        );

        assert_eq!(reader_proxy.remote_reader_guid(), remote_reader_guid);
        assert_eq!(
            reader_proxy.remote_group_entity_id(),
            remote_group_entity_id
        );
        assert_eq!(reader_proxy.unicast_locator_list(), unicast_locator_list);
        assert_eq!(
            reader_proxy.multicast_locator_list(),
            multicast_locator_list
        );
        assert_eq!(reader_proxy.expects_inline_qos(), expects_inline_qos);
        assert_eq!(reader_proxy.is_active(), is_active);
    }

    #[test]
    fn unsent_changes_push_mode_true() {
        let remote_reader_guid = GUID::new([5; 12], EntityId::new([5, 6, 7], 1));
        let remote_group_entity_id = EntityId::new([1, 2, 3], 10);
        let unicast_locator_list = [Locator::new(20, 200, [1; 16])];
        let multicast_locator_list = [Locator::new(10, 100, [2; 16])];
        let expects_inline_qos = false;
        let is_active = true;
        let writer = MockWriter {
            push_mode: true,
            writer_cache: MockHistoryCache {
                seq_num_max: Some(3),
            },
        };
        let reader_proxy: ReaderProxy<MockChangeForReader, _> = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            &writer,
        );

        let unsent_changes = reader_proxy.unsent_changes();
        let expected_unsent_changes = vec![
            MockChangeForReader {
                change: 1,
                is_relevant: true,
                status: ChangeForReaderStatusKind::Unsent,
            },
            MockChangeForReader {
                change: 2,
                is_relevant: true,
                status: ChangeForReaderStatusKind::Unsent,
            },
            MockChangeForReader {
                change: 3,
                is_relevant: true,
                status: ChangeForReaderStatusKind::Unsent,
            },
        ];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn unsent_changes_push_mode_false() {
        let remote_reader_guid = GUID::new([5; 12], EntityId::new([5, 6, 7], 1));
        let remote_group_entity_id = EntityId::new([1, 2, 3], 10);
        let unicast_locator_list = [Locator::new(20, 200, [1; 16])];
        let multicast_locator_list = [Locator::new(10, 100, [2; 16])];
        let expects_inline_qos = false;
        let is_active = true;
        let writer = MockWriter {
            push_mode: false,
            writer_cache: MockHistoryCache {
                seq_num_max: Some(3),
            },
        };
        let reader_proxy: ReaderProxy<MockChangeForReader, _> = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            &writer,
        );

        let unsent_changes = reader_proxy.unsent_changes();
        assert!(unsent_changes.is_empty());
    }

    #[test]
    fn next_unsent_change_push_mode_true() {
        let remote_reader_guid = GUID::new([5; 12], EntityId::new([5, 6, 7], 1));
        let remote_group_entity_id = EntityId::new([1, 2, 3], 10);
        let unicast_locator_list = [Locator::new(20, 200, [1; 16])];
        let multicast_locator_list = [Locator::new(10, 100, [2; 16])];
        let expects_inline_qos = false;
        let is_active = true;
        let writer = MockWriter {
            push_mode: true,
            writer_cache: MockHistoryCache {
                seq_num_max: Some(3),
            },
        };
        let mut reader_proxy: ReaderProxy<MockChangeForReader, _> = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            &writer,
        );

        let next_unsent_change1 = reader_proxy.next_unsent_change();
        let expected_unsent_change1 = Some(MockChangeForReader {
            change: 1,
            is_relevant: true,
            status: ChangeForReaderStatusKind::Unsent,
        });
        let next_unsent_change2 = reader_proxy.next_unsent_change();
        let expected_unsent_change2 = Some(MockChangeForReader {
            change: 2,
            is_relevant: true,
            status: ChangeForReaderStatusKind::Unsent,
        });
        let next_unsent_change3 = reader_proxy.next_unsent_change();
        let expected_unsent_change3 = Some(MockChangeForReader {
            change: 3,
            is_relevant: true,
            status: ChangeForReaderStatusKind::Unsent,
        });

        let next_unsent_change4 = reader_proxy.next_unsent_change();
        let expected_unsent_change4 = None;

        assert_eq!(next_unsent_change1, expected_unsent_change1);
        assert_eq!(next_unsent_change2, expected_unsent_change2);
        assert_eq!(next_unsent_change3, expected_unsent_change3);
        assert_eq!(next_unsent_change4, expected_unsent_change4);
    }

    #[test]
    fn next_unsent_change_push_mode_false() {
        let remote_reader_guid = GUID::new([5; 12], EntityId::new([5, 6, 7], 1));
        let remote_group_entity_id = EntityId::new([1, 2, 3], 10);
        let unicast_locator_list = [Locator::new(20, 200, [1; 16])];
        let multicast_locator_list = [Locator::new(10, 100, [2; 16])];
        let expects_inline_qos = false;
        let is_active = true;
        let writer = MockWriter {
            push_mode: false,
            writer_cache: MockHistoryCache {
                seq_num_max: Some(3),
            },
        };
        let mut reader_proxy: ReaderProxy<MockChangeForReader, _> = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            &writer,
        );

        let next_unsent_change = reader_proxy.next_unsent_change();
        let expected_unsent_change = None;

        assert_eq!(next_unsent_change, expected_unsent_change);
    }

    #[test]
    fn unacked_changes_push_mode_true() {
        let remote_reader_guid = GUID::new([5; 12], EntityId::new([5, 6, 7], 1));
        let remote_group_entity_id = EntityId::new([1, 2, 3], 10);
        let unicast_locator_list = [Locator::new(20, 200, [1; 16])];
        let multicast_locator_list = [Locator::new(10, 100, [2; 16])];
        let expects_inline_qos = false;
        let is_active = true;
        let writer = MockWriter {
            push_mode: true,
            writer_cache: MockHistoryCache {
                seq_num_max: Some(5),
            },
        };
        let mut reader_proxy: ReaderProxy<MockChangeForReader, _> = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            &writer,
        );

        // Changes up to 5 are available
        // Changes 1 to 4 are sent
        // Changes up to 2 are acknowledged
        // Expected unacked changes are 3 and 4
        reader_proxy.next_unsent_change();
        reader_proxy.next_unsent_change();
        reader_proxy.next_unsent_change();
        reader_proxy.next_unsent_change();
        reader_proxy.acked_changes_set(2);

        let unacked_changes = reader_proxy.unacked_changes();
        let expected_unacked_changes = vec![
            MockChangeForReader {
                change: 3,
                is_relevant: true,
                status: ChangeForReaderStatusKind::Unacknowledged,
            },
            MockChangeForReader {
                change: 4,
                is_relevant: true,
                status: ChangeForReaderStatusKind::Unacknowledged,
            },
        ];

        assert_eq!(unacked_changes, expected_unacked_changes);
    }

    #[test]
    fn unacked_changes_push_mode_false() {
        let remote_reader_guid = GUID::new([5; 12], EntityId::new([5, 6, 7], 1));
        let remote_group_entity_id = EntityId::new([1, 2, 3], 10);
        let unicast_locator_list = [Locator::new(20, 200, [1; 16])];
        let multicast_locator_list = [Locator::new(10, 100, [2; 16])];
        let expects_inline_qos = false;
        let is_active = true;
        let writer = MockWriter {
            push_mode: false,
            writer_cache: MockHistoryCache {
                seq_num_max: Some(5),
            },
        };
        let mut reader_proxy: ReaderProxy<MockChangeForReader, _> = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            &writer,
        );

        let cc = writer.new_change(
            rust_rtps::types::ChangeKind::Alive,
            (),
            ParameterList::new(),
            [1; 16],
        );

        writer.writer_cache().add_change(cc);

        // Changes up to 5 are available
        // Changes up to 2 are acknowledged
        // Change 4 is requested
        // Expected unacked changes are 3 and 5
        reader_proxy.acked_changes_set(2);
        reader_proxy.requested_changes_set(&[4]);

        let unacked_changes = reader_proxy.unacked_changes();
        let expected_unacked_changes = vec![
            MockChangeForReader {
                change: 3,
                is_relevant: true,
                status: ChangeForReaderStatusKind::Unacknowledged,
            },
            MockChangeForReader {
                change: 5,
                is_relevant: true,
                status: ChangeForReaderStatusKind::Unacknowledged,
            },
        ];

        assert_eq!(unacked_changes, expected_unacked_changes);
    }

    #[test]
    fn requested_changes() {
        let remote_reader_guid = GUID::new([5; 12], EntityId::new([5, 6, 7], 1));
        let remote_group_entity_id = EntityId::new([1, 2, 3], 10);
        let unicast_locator_list = [Locator::new(20, 200, [1; 16])];
        let multicast_locator_list = [Locator::new(10, 100, [2; 16])];
        let expects_inline_qos = false;
        let is_active = true;
        let writer = MockWriter {
            push_mode: false,
            writer_cache: MockHistoryCache {
                seq_num_max: Some(5),
            },
        };
        let mut reader_proxy: ReaderProxy<MockChangeForReader, _> = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            &writer,
        );

        reader_proxy.requested_changes_set(&[2, 3]);
        reader_proxy.requested_changes_set(&[4]);

        let expected_requested_changes = vec![
            MockChangeForReader::new(2, ChangeForReaderStatusKind::Requested, true),
            MockChangeForReader::new(3, ChangeForReaderStatusKind::Requested, true),
            MockChangeForReader::new(4, ChangeForReaderStatusKind::Requested, true),
        ];
        assert_eq!(reader_proxy.requested_changes(), expected_requested_changes);
    }

    #[test]
    fn requested_inexistent_changes() {
        let remote_reader_guid = GUID::new([5; 12], EntityId::new([5, 6, 7], 1));
        let remote_group_entity_id = EntityId::new([1, 2, 3], 10);
        let unicast_locator_list = [Locator::new(20, 200, [1; 16])];
        let multicast_locator_list = [Locator::new(10, 100, [2; 16])];
        let expects_inline_qos = false;
        let is_active = true;
        let writer = MockWriter {
            push_mode: false,
            writer_cache: MockHistoryCache {
                seq_num_max: Some(5),
            },
        };
        let mut reader_proxy: ReaderProxy<MockChangeForReader, _> = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            &writer,
        );

        reader_proxy.requested_changes_set(&[6, 7, 8]);

        assert!(reader_proxy.requested_changes().is_empty());
    }

    #[test]
    fn next_requested_change() {
        let remote_reader_guid = GUID::new([5; 12], EntityId::new([5, 6, 7], 1));
        let remote_group_entity_id = EntityId::new([1, 2, 3], 10);
        let unicast_locator_list = [Locator::new(20, 200, [1; 16])];
        let multicast_locator_list = [Locator::new(10, 100, [2; 16])];
        let expects_inline_qos = false;
        let is_active = true;
        let writer = MockWriter {
            push_mode: false,
            writer_cache: MockHistoryCache {
                seq_num_max: Some(5),
            },
        };
        let mut reader_proxy: ReaderProxy<MockChangeForReader, _> = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            &writer,
        );

        reader_proxy.requested_changes_set(&[2, 3]);
        reader_proxy.requested_changes_set(&[3, 4]);

        let next_requested_change1 = Some(MockChangeForReader::new(
            2,
            ChangeForReaderStatusKind::Requested,
            true,
        ));
        let next_requested_change2 = Some(MockChangeForReader::new(
            3,
            ChangeForReaderStatusKind::Requested,
            true,
        ));
        let next_requested_change3 = Some(MockChangeForReader::new(
            4,
            ChangeForReaderStatusKind::Requested,
            true,
        ));
        let next_requested_change4 = None;

        assert_eq!(reader_proxy.next_requested_change(), next_requested_change1);
        assert_eq!(reader_proxy.next_requested_change(), next_requested_change2);
        assert_eq!(reader_proxy.next_requested_change(), next_requested_change3);
        assert_eq!(reader_proxy.next_requested_change(), next_requested_change4);
    }
}
