use rust_rtps::{
    behavior::{stateless_writer::RTPSReaderLocator, RTPSWriter},
    structure::RTPSHistoryCache,
    types::{Locator, SequenceNumber},
};

pub struct ReaderLocator {
    locator: Locator,
    expects_inline_qos: bool,
    next_unsent_change: SequenceNumber,
    requested_changes: Vec<SequenceNumber>,
}

impl RTPSReaderLocator for ReaderLocator {
    type CacheChangeRepresentation = SequenceNumber;
    type CacheChangeRepresentationList = Vec<Self::CacheChangeRepresentation>;

    fn locator(&self) -> Locator {
        self.locator
    }

    fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
            next_unsent_change: 0.into(),
            requested_changes: Vec::new(),
        }
    }

    fn requested_changes(&self, _writer: &impl RTPSWriter) -> Self::CacheChangeRepresentationList {
        self.requested_changes.clone()
    }

    fn unsent_changes(&self, writer: &impl RTPSWriter) -> Self::CacheChangeRepresentationList {
        let max_history_cache_seq_num = writer
            .writer_cache()
            .get_seq_num_max()
            .unwrap_or(0.into())
            .into();
        let next_unsent_change: i64 = self.next_unsent_change.into();
        (next_unsent_change + 1..=max_history_cache_seq_num)
            .filter(|&x| writer.writer_cache().get_change(x.into()).is_some())
            .map(|x| x.into())
            .collect()
    }

    fn next_requested_change(
        &mut self,
        _writer: &impl RTPSWriter,
    ) -> Option<Self::CacheChangeRepresentation> {
        let next_requested_change = *self.requested_changes.iter().min()?;
        self.requested_changes
            .retain(|x| x != &next_requested_change);
        Some(next_requested_change)
    }

    fn next_unsent_change(
        &mut self,
        writer: &impl RTPSWriter,
    ) -> Option<Self::CacheChangeRepresentation> {
        self.next_unsent_change = *self.unsent_changes(writer).iter().min()?;
        Some(self.next_unsent_change)
    }

    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[SequenceNumber],
        writer: &impl RTPSWriter,
    ) {
        for value in req_seq_num_set {
            if value <= &writer.writer_cache().get_seq_num_max().unwrap_or(0.into()) {
                self.requested_changes.push(*value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps::structure::{RTPSCacheChange, RTPSEndpoint, RTPSEntity};

    use super::*;

    struct MockCacheChange;

    impl RTPSCacheChange for MockCacheChange {
        type Data = u8;
        type InstanceHandle = u8;

        fn new(
            _kind: rust_rtps::types::ChangeKind,
            _writer_guid: rust_rtps::types::GUID,
            _instance_handle: Self::InstanceHandle,
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

        fn instance_handle(&self) -> &Self::InstanceHandle {
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

        fn get_change(
            &self,
            _seq_num: rust_rtps::types::SequenceNumber,
        ) -> Option<&Self::CacheChangeType> {
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
    impl RTPSWriter for MockWriter {
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

        fn writer_cache_mut(&mut self) -> &mut Self::HistoryCacheType {
            todo!()
        }

        fn new_change(
            &mut self,
            _kind: rust_rtps::types::ChangeKind,
            _data: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType as RTPSCacheChange>::Data,
            _inline_qos: rust_rtps::messages::submessages::submessage_elements::ParameterList,
            _handle:  <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType as RTPSCacheChange>::InstanceHandle,
        ) -> <Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType {
            todo!()
        }
    }

    #[test]
    fn empty_unsent_changes() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(0.into()),
        };
        let writer = MockWriter { writer_cache };
        let reader_locator = ReaderLocator::new(locator, false);
        let unsent_changes = reader_locator.unsent_changes(&writer);

        assert!(unsent_changes.is_empty());
    }

    #[test]
    fn some_unsent_changes() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(5.into()),
        };
        let writer = MockWriter { writer_cache };
        let reader_locator = ReaderLocator::new(locator, false);
        let unsent_changes = reader_locator.unsent_changes(&writer);
        let expected_unsent_changes = vec![1, 2, 3, 4, 5];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn next_unsent_change() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(2.into()),
        };
        let writer = MockWriter { writer_cache };
        let mut reader_locator = ReaderLocator::new(locator, false);
        assert_eq!(reader_locator.next_unsent_change(&writer), Some(1.into()));
        assert_eq!(reader_locator.next_unsent_change(&writer), Some(2.into()));
        assert_eq!(reader_locator.next_unsent_change(&writer), None);
    }

    #[test]
    fn requested_changes_set_and_get() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(5.into()),
        };
        let writer = MockWriter { writer_cache };
        let mut reader_locator = ReaderLocator::new(locator, false);
        let requested_changes = [1.into(), 3.into(), 5.into()];
        reader_locator.requested_changes_set(&requested_changes, &writer);
        assert_eq!(reader_locator.requested_changes(&writer), requested_changes)
    }

    #[test]
    fn requested_changes_out_of_history_cache_range_set_and_get() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(2.into()),
        };
        let writer = MockWriter { writer_cache };

        let mut reader_locator = ReaderLocator::new(locator, false);

        let requested_changes = [1.into(), 3.into(), 5.into()];
        let expected_requested_changes = [1];
        reader_locator.requested_changes_set(&requested_changes, &writer);
        assert_eq!(
            reader_locator.requested_changes(&writer),
            expected_requested_changes
        )
    }

    #[test]
    fn next_requested_change() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(5.into()),
        };
        let writer = MockWriter { writer_cache };
        let mut reader_locator = ReaderLocator::new(locator, false);
        reader_locator.requested_changes_set(&[1.into(), 3.into(), 5.into()], &writer);
        assert_eq!(
            reader_locator.next_requested_change(&writer),
            Some(1.into())
        );
        assert_eq!(
            reader_locator.next_requested_change(&writer),
            Some(3.into())
        );
        assert_eq!(
            reader_locator.next_requested_change(&writer),
            Some(5.into())
        );
        assert_eq!(reader_locator.next_requested_change(&writer), None);
    }

    #[test]
    fn next_requested_change_with_multiple_and_repeated_requested_changes() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let writer_cache = MockHistoryCache {
            seq_num_max: Some(5.into()),
        };
        let writer = MockWriter { writer_cache };
        let mut reader_locator = ReaderLocator::new(locator, false);

        reader_locator.requested_changes_set(&[1.into(), 3.into(), 5.into()], &writer);
        reader_locator.next_requested_change(&writer);
        reader_locator.requested_changes_set(&[2.into(), 3.into(), 4.into()], &writer);

        assert_eq!(
            reader_locator.next_requested_change(&writer),
            Some(2.into())
        );
        assert_eq!(
            reader_locator.next_requested_change(&writer),
            Some(3.into())
        );
        assert_eq!(
            reader_locator.next_requested_change(&writer),
            Some(4.into())
        );
        assert_eq!(
            reader_locator.next_requested_change(&writer),
            Some(5.into())
        );
        assert_eq!(reader_locator.next_requested_change(&writer), None);
    }
}
