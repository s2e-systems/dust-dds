use std::sync::Arc;

use rust_rtps::{
    behavior::{
        stateful_writer::reader_proxy::RTPSChangeForReader, types::ChangeForReaderStatusKind,
        RTPSReaderProxy, RTPSWriter,
    },
    types::{EntityId, Locator, SequenceNumber, GUID},
};

pub struct ChangeForReader {
    change: SequenceNumber,
    status: ChangeForReaderStatusKind,
    is_relevant: bool,
}

impl RTPSChangeForReader for ChangeForReader {
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

pub struct ReaderProxy<W: RTPSWriter> {
    remote_reader_guid: GUID,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    expects_inline_qos: bool,
    is_active: bool,

    writer: Arc<W>,
    next_unsent_change: SequenceNumber,
    highest_acked_change: SequenceNumber,
    requested_changes: Vec<SequenceNumber>,
}

impl<W: RTPSWriter> RTPSReaderProxy for ReaderProxy<W> {
    type ChangeForReaderType = ChangeForReader;
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
        let mut changes_for_reader: Vec<Self::ChangeForReaderType> = (1..=self
            .highest_acked_change)
            .map(|sn| {
                Self::ChangeForReaderType::new(sn, ChangeForReaderStatusKind::Acknowledged, true)
            })
            .collect();
        changes_for_reader.append(&mut self.unsent_changes());
        changes_for_reader.append(&mut self.unacked_changes());
        changes_for_reader.append(&mut self.requested_changes());

        changes_for_reader
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    // fn writer(&self) -> &Self::Writer {
    //     self.writer
    // }

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
        writer: Arc<Self::Writer>,
    ) -> Self {
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
            expects_inline_qos,
            is_active,
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
        Some(Self::ChangeForReaderType::new(
            next_requested_change,
            ChangeForReaderStatusKind::Requested,
            true,
        ))
    }

    fn next_unsent_change(&mut self) -> Option<Self::ChangeForReaderType> {
        self.next_unsent_change = self.unsent_changes().iter().map(|x| x.change()).min()?;
        Some(Self::ChangeForReaderType::new(
            self.next_unsent_change,
            ChangeForReaderStatusKind::Unsent,
            true,
        ))
    }

    fn unsent_changes(&self) -> Self::ChangeForReaderTypeList {
        if self.writer.push_mode() == true {
            let max_history_cache_seq_num = self.writer.last_change_sequence_number();
            (self.next_unsent_change + 1..=max_history_cache_seq_num)
                .map(|sn| {
                    Self::ChangeForReaderType::new(sn, ChangeForReaderStatusKind::Unsent, true)
                })
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
            .map(|sn| {
                Self::ChangeForReaderType::new(*sn, ChangeForReaderStatusKind::Requested, true)
            })
            .collect()
    }

    fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        for value in req_seq_num_set {
            if value <= &self.writer.last_change_sequence_number() {
                if !self.requested_changes.contains(value) {
                    self.requested_changes.push(*value);
                }
            }
        }
    }

    type WriterReferenceType = Arc<W>;

    fn unacked_changes(&self) -> Self::ChangeForReaderTypeList {
        let mut unacked_changes: Vec<SequenceNumber> = if self.writer.push_mode() == true {
            // According to the diagram in page 8.4.9.3 this is every change that has been sent
            // longer ago than writer.nackSuppressionDuration() and not yet acknowledged
            // TODO: nackSuppressionDuration is for now hard-coded 0
            (self.highest_acked_change + 1..=self.next_unsent_change).collect()
        } else {
            (self.highest_acked_change + 1..=self.writer.last_change_sequence_number()).collect()
        };
        for requested_changed in self.requested_changes.iter() {
            unacked_changes.retain(|x| x != requested_changed);
        }
        unacked_changes
            .iter()
            .map(|sn| {
                Self::ChangeForReaderType::new(*sn, ChangeForReaderStatusKind::Unacknowledged, true)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps::structure::{
        history_cache::RTPSHistoryCacheRead, RTPSCacheChange, RTPSEndpoint, RTPSEntity,
        RTPSHistoryCache,
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
    impl<'a> RTPSHistoryCacheRead<'a> for MockHistoryCache {
        type CacheChangeType = MockCacheChange;
        type Item = &'a MockCacheChange;
    }

    impl RTPSHistoryCache for MockHistoryCache {
        type CacheChangeType = MockCacheChange;
        type HistoryCacheStorageType = Self;

        fn new() -> Self {
            todo!()
        }

        fn add_change(&self, _change: Self::CacheChangeType) {
            todo!()
        }

        fn remove_change(&self, _seq_num: SequenceNumber) {
            todo!()
        }

        fn get_change<'a>(
            &'a self,
            _seq_num: rust_rtps::types::SequenceNumber,
        ) -> Option<<Self::HistoryCacheStorageType as RTPSHistoryCacheRead<'a>>::Item> {
            todo!()
        }

        fn get_seq_num_min(&self) -> Option<SequenceNumber> {
            todo!()
        }

        fn get_seq_num_max(&self) -> Option<SequenceNumber> {
            todo!()
        }
    }
    struct MockWriter {
        push_mode: bool,
        last_change_sequence_number: SequenceNumber,
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
    impl RTPSWriter for MockWriter {
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
            self.last_change_sequence_number
        }

        fn data_max_sized_serialized(&self) -> i32 {
            todo!()
        }

        fn writer_cache(&self) -> &Self::HistoryCacheType {
            todo!()
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

    impl PartialEq for ChangeForReader {
        fn eq(&self, other: &Self) -> bool {
            self.change == other.change
                && self.status == self.status
                && self.is_relevant == other.is_relevant
        }
    }

    impl std::fmt::Debug for ChangeForReader {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ChangeForReader")
                .field("change", &self.change)
                .field("status", &self.status)
                .field("is_relevant", &self.is_relevant)
                .finish()
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
            last_change_sequence_number: 1,
        };
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            Arc::new(writer),
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
            last_change_sequence_number: 3,
        };
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            Arc::new(writer),
        );

        let unsent_changes = reader_proxy.unsent_changes();
        let expected_unsent_changes = vec![
            ChangeForReader {
                change: 1,
                is_relevant: true,
                status: ChangeForReaderStatusKind::Unsent,
            },
            ChangeForReader {
                change: 2,
                is_relevant: true,
                status: ChangeForReaderStatusKind::Unsent,
            },
            ChangeForReader {
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
            last_change_sequence_number: 3,
        };
        let reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            Arc::new(writer),
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
            last_change_sequence_number: 3,
        };
        let mut reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            Arc::new(writer),
        );

        let next_unsent_change1 = reader_proxy.next_unsent_change();
        let expected_unsent_change1 = Some(ChangeForReader {
            change: 1,
            is_relevant: true,
            status: ChangeForReaderStatusKind::Unsent,
        });
        let next_unsent_change2 = reader_proxy.next_unsent_change();
        let expected_unsent_change2 = Some(ChangeForReader {
            change: 2,
            is_relevant: true,
            status: ChangeForReaderStatusKind::Unsent,
        });
        let next_unsent_change3 = reader_proxy.next_unsent_change();
        let expected_unsent_change3 = Some(ChangeForReader {
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
            last_change_sequence_number: 3,
        };
        let mut reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            Arc::new(writer),
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
            last_change_sequence_number: 5,
        };
        let mut reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            Arc::new(writer),
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
            ChangeForReader {
                change: 3,
                is_relevant: true,
                status: ChangeForReaderStatusKind::Unacknowledged,
            },
            ChangeForReader {
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
            last_change_sequence_number: 5,
        };
        let mut reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            Arc::new(writer),
        );

        // Changes up to 5 are available
        // Changes up to 2 are acknowledged
        // Change 4 is requested
        // Expected unacked changes are 3 and 5
        reader_proxy.acked_changes_set(2);
        reader_proxy.requested_changes_set(&[4]);

        let unacked_changes = reader_proxy.unacked_changes();
        let expected_unacked_changes = vec![
            ChangeForReader {
                change: 3,
                is_relevant: true,
                status: ChangeForReaderStatusKind::Unacknowledged,
            },
            ChangeForReader {
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
            last_change_sequence_number: 5,
        };
        let mut reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            Arc::new(writer),
        );

        reader_proxy.requested_changes_set(&[2, 3]);
        reader_proxy.requested_changes_set(&[4]);

        let expected_requested_changes = vec![
            ChangeForReader::new(2, ChangeForReaderStatusKind::Requested, true),
            ChangeForReader::new(3, ChangeForReaderStatusKind::Requested, true),
            ChangeForReader::new(4, ChangeForReaderStatusKind::Requested, true),
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
            last_change_sequence_number: 5,
        };
        let mut reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            Arc::new(writer),
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
            last_change_sequence_number: 5,
        };
        let mut reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            Arc::new(writer),
        );

        reader_proxy.requested_changes_set(&[2, 3]);
        reader_proxy.requested_changes_set(&[3, 4]);

        let next_requested_change1 = Some(ChangeForReader::new(
            2,
            ChangeForReaderStatusKind::Requested,
            true,
        ));
        let next_requested_change2 = Some(ChangeForReader::new(
            3,
            ChangeForReaderStatusKind::Requested,
            true,
        ));
        let next_requested_change3 = Some(ChangeForReader::new(
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

    #[test]
    fn changes_for_reader_push_mode_true() {
        let remote_reader_guid = GUID::new([5; 12], EntityId::new([5, 6, 7], 1));
        let remote_group_entity_id = EntityId::new([1, 2, 3], 10);
        let unicast_locator_list = [Locator::new(20, 200, [1; 16])];
        let multicast_locator_list = [Locator::new(10, 100, [2; 16])];
        let expects_inline_qos = false;
        let is_active = true;
        let writer = MockWriter {
            push_mode: true,
            last_change_sequence_number: 6,
        };
        let mut reader_proxy = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
            Arc::new(writer),
        );

        // Changes up to 6 are available
        // Changes up to including 2 are acknowledged
        // Changes 1 to 4 are sent
        // Change 3 is requested
        reader_proxy.next_unsent_change();
        reader_proxy.next_unsent_change();
        reader_proxy.next_unsent_change();
        reader_proxy.next_unsent_change();
        reader_proxy.acked_changes_set(2);
        reader_proxy.requested_changes_set(&[3]);
        reader_proxy.requested_changes_set(&[3]);

        let expected_changes_for_reader1 = ChangeForReader {
            change: 1,
            status: ChangeForReaderStatusKind::Acknowledged,
            is_relevant: true,
        };
        let expected_changes_for_reader2 = ChangeForReader {
            change: 2,
            status: ChangeForReaderStatusKind::Acknowledged,
            is_relevant: true,
        };
        let expected_changes_for_reader3 = ChangeForReader {
            change: 3,
            status: ChangeForReaderStatusKind::Requested,
            is_relevant: true,
        };
        let expected_changes_for_reader4 = ChangeForReader {
            change: 4,
            status: ChangeForReaderStatusKind::Unacknowledged,
            is_relevant: true,
        };
        let expected_changes_for_reader5 = ChangeForReader {
            change: 5,
            status: ChangeForReaderStatusKind::Unsent,
            is_relevant: true,
        };
        let expected_changes_for_reader6 = ChangeForReader {
            change: 6,
            status: ChangeForReaderStatusKind::Unsent,
            is_relevant: true,
        };

        let changes_for_reader = reader_proxy.changes_for_reader();
        assert_eq!(changes_for_reader.len(), 6);
        assert!(changes_for_reader.contains(&expected_changes_for_reader1));
        assert!(changes_for_reader.contains(&expected_changes_for_reader2));
        assert!(changes_for_reader.contains(&expected_changes_for_reader3));
        assert!(changes_for_reader.contains(&expected_changes_for_reader4));
        assert!(changes_for_reader.contains(&expected_changes_for_reader5));
        assert!(changes_for_reader.contains(&expected_changes_for_reader6));
    }
}
