use std::{marker::PhantomData, sync::Arc};

use rust_rtps::{
    behavior::{stateless_writer::RTPSReaderLocator, RTPSWriter},
    structure::RTPSHistoryCache,
    types::{Locator, SequenceNumber},
};

pub struct ReaderLocator<'a, T: RTPSWriter<'a>> {
    locator: Locator,
    expects_inline_qos: bool,
    writer: Arc<T>,
    next_unsent_change: SequenceNumber,
    requested_changes: Vec<SequenceNumber>,
    phantom: PhantomData<&'a ()>,
}

impl<'a, T: RTPSWriter<'a>> RTPSReaderLocator<'a> for ReaderLocator<'a, T> {
    type CacheChangeRepresentation = SequenceNumber;
    type CacheChangeRepresentationList = Vec<Self::CacheChangeRepresentation>;
    type Writer = T;
    type WriterReferenceType = Arc<T>;

    fn requested_changes(&self) -> Self::CacheChangeRepresentationList {
        self.requested_changes.clone()
    }

    fn unsent_changes(&self) -> Self::CacheChangeRepresentationList {
        let max_history_cache_seq_num = self.writer.writer_cache().get_seq_num_max().unwrap_or(0);
        (self.next_unsent_change + 1..=max_history_cache_seq_num).collect()
    }

    fn new(locator: Locator, expects_inline_qos: bool, writer: Arc<Self::Writer>) -> Self {
        Self {
            locator,
            expects_inline_qos,
            writer,
            next_unsent_change: 0,
            requested_changes: Vec::new(),
            phantom: PhantomData,
        }
    }

    fn locator(&self) -> Locator {
        self.locator
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    fn next_requested_change(&mut self) -> Option<Self::CacheChangeRepresentation> {
        let next_requested_change = *self.requested_changes.iter().min()?;
        self.requested_changes
            .retain(|x| x != &next_requested_change);
        Some(next_requested_change)
    }

    fn next_unsent_change(&mut self) -> Option<Self::CacheChangeRepresentation> {
        self.next_unsent_change = *self.unsent_changes().iter().min()?;
        Some(self.next_unsent_change)
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
}

#[cfg(test)]
mod tests {
    use rust_rtps::structure::{
        history_cache::RTPSHistoryCacheRead, RTPSCacheChange, RTPSEndpoint, RTPSEntity,
    };

    use super::*;

    struct MockCacheChange;

    impl RTPSCacheChange for MockCacheChange {
        type Data = u8;

        fn new(
            _kind: rust_rtps::types::ChangeKind,
            _writer_guid: rust_rtps::types::GUID,
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

        fn writer_guid(&self) -> rust_rtps::types::GUID {
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

        fn get_change<'a>(
            &'a self,
            seq_num: SequenceNumber,
        ) -> Option<<Self::CacheChangeReadType as RTPSHistoryCacheRead<'a>>::Item>
        where
            Self::CacheChangeReadType: RTPSHistoryCacheRead<'a>,
        {
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
        writer_cache: MockHistoryCache,
    }

    impl RTPSEntity for MockWriter {
        fn guid(&self) -> rust_rtps::types::GUID {
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
            _guid: rust_rtps::types::GUID,
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
            todo!()
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

    #[test]
    fn empty_unsent_changes() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(0),
        };
        let writer = MockWriter { writer_cache };
        let reader_locator = ReaderLocator::new(locator, false, Arc::new(writer));
        let unsent_changes = reader_locator.unsent_changes();

        assert!(unsent_changes.is_empty());
    }

    #[test]
    fn some_unsent_changes() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(5),
        };
        let writer = MockWriter { writer_cache };
        let reader_locator = ReaderLocator::new(locator, false, Arc::new(writer));
        let unsent_changes = reader_locator.unsent_changes();
        let expected_unsent_changes = vec![1, 2, 3, 4, 5];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn next_unsent_change() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(2),
        };
        let writer = MockWriter { writer_cache };
        let mut reader_locator = ReaderLocator::new(locator, false, Arc::new(writer));
        assert_eq!(reader_locator.next_unsent_change(), Some(1));
        assert_eq!(reader_locator.next_unsent_change(), Some(2));
        assert_eq!(reader_locator.next_unsent_change(), None);
    }

    #[test]
    fn requested_changes_set_and_get() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(5),
        };
        let writer = MockWriter { writer_cache };
        let mut reader_locator = ReaderLocator::new(locator, false, Arc::new(writer));
        let requested_changes = [1, 3, 5];
        reader_locator.requested_changes_set(&requested_changes);
        assert_eq!(reader_locator.requested_changes(), requested_changes)
    }

    #[test]
    fn requested_changes_out_of_history_cache_range_set_and_get() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(2),
        };
        let writer = MockWriter { writer_cache };

        let mut reader_locator = ReaderLocator::new(locator, false, Arc::new(writer));

        let requested_changes = [1, 3, 5];
        let expected_requested_changes = [1];
        reader_locator.requested_changes_set(&requested_changes);
        assert_eq!(
            reader_locator.requested_changes(),
            expected_requested_changes
        )
    }

    #[test]
    fn next_requested_change() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(5),
        };
        let writer = MockWriter { writer_cache };
        let mut reader_locator = ReaderLocator::new(locator, false, Arc::new(writer));
        reader_locator.requested_changes_set(&[1, 3, 5]);
        assert_eq!(reader_locator.next_requested_change(), Some(1));
        assert_eq!(reader_locator.next_requested_change(), Some(3));
        assert_eq!(reader_locator.next_requested_change(), Some(5));
        assert_eq!(reader_locator.next_requested_change(), None);
    }

    #[test]
    fn next_requested_change_with_multiple_and_repeated_requested_changes() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(5),
        };
        let writer = MockWriter { writer_cache };
        let mut reader_locator = ReaderLocator::new(locator, false, Arc::new(writer));

        reader_locator.requested_changes_set(&[1, 3, 5]);
        reader_locator.next_requested_change();
        reader_locator.requested_changes_set(&[2, 3, 4]);

        assert_eq!(reader_locator.next_requested_change(), Some(2));
        assert_eq!(reader_locator.next_requested_change(), Some(3));
        assert_eq!(reader_locator.next_requested_change(), Some(4));
        assert_eq!(reader_locator.next_requested_change(), Some(5));
        assert_eq!(reader_locator.next_requested_change(), None);
    }
}
