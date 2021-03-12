use std::marker::PhantomData;

use rust_rtps::{
    behavior::{
        stateful_writer::reader_proxy::RTPSChangeForReader, types::ChangeForReaderStatusKind,
        RTPSReaderProxy, RTPSWriter,
    },
    structure::RTPSHistoryCache,
    types::{EntityId, Locator, SequenceNumber, GUID},
};

pub struct ReaderProxy<T: RTPSChangeForReader<CacheChangeRepresentation = SequenceNumber>> {
    remote_reader_guid: GUID,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    expects_inline_qos: bool,
    is_active: bool,

    phantom: PhantomData<T>,
    next_unsent_change: SequenceNumber,
}

impl<T: RTPSChangeForReader<CacheChangeRepresentation = SequenceNumber>> RTPSReaderProxy
    for ReaderProxy<T>
{
    type ChangeForReaderType = T;
    type ChangeForReaderTypeList = Vec<Self::ChangeForReaderType>;

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

    fn new(
        remote_reader_guid: GUID,
        remote_group_entity_id: EntityId,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        expects_inline_qos: bool,
        is_active: bool,
    ) -> Self {
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
            expects_inline_qos,
            is_active,
            phantom: PhantomData,
            next_unsent_change: 0,
        }
    }

    fn acked_changes_set(&mut self, _committed_seq_num: SequenceNumber) {
        todo!()
    }

    fn next_requested_change(&mut self) -> Option<Self::ChangeForReaderType> {
        todo!()
    }

    fn next_unsent_change(
        &mut self,
        _writer: &impl RTPSWriter,
    ) -> Option<Self::ChangeForReaderType> {
        todo!()
        // self.next_unsent_change = self.unsent_changes(writer).iter().min()?;
        // Some(self.next_unsent_change)
    }

    fn unsent_changes(&self, writer: &impl RTPSWriter) -> Self::ChangeForReaderTypeList {
        if writer.push_mode() == true {
            let max_history_cache_seq_num = writer.writer_cache().get_seq_num_max().unwrap_or(0);
            (self.next_unsent_change + 1..=max_history_cache_seq_num)
                .map(|sn| T::new(sn, ChangeForReaderStatusKind::Unsent, true))
                .collect()
        } else {
            // If writer push_mode no change is unsent since they have to be
            // explicitly requested by the receiver using acknack
            Vec::new()
        }
    }

    fn requested_changes(&self) -> Self::ChangeForReaderTypeList {
        todo!()
    }

    fn requested_changes_set(&mut self, _req_seq_num_set: &[SequenceNumber]) {
        todo!()
    }

    fn unacked_changes(&self) -> Self::ChangeForReaderTypeList {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps::structure::{RTPSCacheChange, RTPSEndpoint, RTPSEntity};

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

    impl RTPSHistoryCache for MockHistoryCache {
        type CacheChangeType = MockCacheChange;

        fn new() -> Self {
            todo!()
        }

        fn add_change(&mut self, _change: Self::CacheChangeType) {
            todo!()
        }

        fn remove_change(&mut self, _seq_num: SequenceNumber) {
            todo!()
        }

        fn get_change(&self, _seq_num: SequenceNumber) -> Option<&Self::CacheChangeType> {
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
            &mut self,
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
        let reader_proxy: ReaderProxy<MockChangeForReader> = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
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
        let reader_proxy: ReaderProxy<MockChangeForReader> = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
        );
        let writer = MockWriter {
            push_mode: true,
            writer_cache: MockHistoryCache {
                seq_num_max: Some(3),
            },
        };

        let unsent_changes = reader_proxy.unsent_changes(&writer);
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
        let reader_proxy: ReaderProxy<MockChangeForReader> = ReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            &unicast_locator_list,
            &multicast_locator_list,
            expects_inline_qos,
            is_active,
        );
        let writer = MockWriter {
            push_mode: false,
            writer_cache: MockHistoryCache {
                seq_num_max: Some(3),
            },
        };

        let unsent_changes = reader_proxy.unsent_changes(&writer);
        assert!(unsent_changes.is_empty());
    }
}
