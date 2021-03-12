use rust_rtps::{
    behavior::stateless_writer::RTPSReaderLocator,
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

    fn requested_changes(&self) -> Self::CacheChangeRepresentationList {
        self.requested_changes.clone()
    }

    fn unsent_changes(
        &self,
        writer_cache: &impl RTPSHistoryCache,
    ) -> Self::CacheChangeRepresentationList {
        let max_history_cache_seq_num = writer_cache.get_seq_num_max().unwrap_or(0);
        (self.next_unsent_change + 1..=max_history_cache_seq_num).collect()
    }

    fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
            next_unsent_change: 0,
            requested_changes: Vec::new(),
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

    fn next_unsent_change(
        &mut self,
        writer_cache: &impl RTPSHistoryCache,
    ) -> Option<Self::CacheChangeRepresentation> {
        self.next_unsent_change = *self.unsent_changes(writer_cache).iter().min()?;
        Some(self.next_unsent_change)
    }

    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[SequenceNumber],
        writer_cache: &impl RTPSHistoryCache,
    ) {
        for value in req_seq_num_set {
            if value <= &writer_cache.get_seq_num_max().unwrap_or_default() {
                self.requested_changes.push(*value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps::structure::RTPSCacheChange;

    use super::*;

    struct MockCacheChangeType;

    impl RTPSCacheChange for MockCacheChangeType {
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
        type CacheChangeType = MockCacheChangeType;

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
    #[test]
    fn empty_unsent_changes() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let history_cache = MockHistoryCache {
            seq_num_max: Some(0),
        };
        let reader_locator = ReaderLocator::new(locator, false);
        let unsent_changes = reader_locator.unsent_changes(&history_cache);

        assert!(unsent_changes.is_empty());
    }

    #[test]
    fn some_unsent_changes() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let history_cache = MockHistoryCache {
            seq_num_max: Some(5),
        };
        let reader_locator = ReaderLocator::new(locator, false);
        let unsent_changes = reader_locator.unsent_changes(&history_cache);
        let expected_unsent_changes = vec![1, 2, 3, 4, 5];

        assert_eq!(unsent_changes, expected_unsent_changes);
    }

    #[test]
    fn next_unsent_change() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let history_cache = MockHistoryCache {
            seq_num_max: Some(2),
        };

        let mut reader_locator = ReaderLocator::new(locator, false);
        assert_eq!(reader_locator.next_unsent_change(&history_cache), Some(1));
        assert_eq!(reader_locator.next_unsent_change(&history_cache), Some(2));
        assert_eq!(reader_locator.next_unsent_change(&history_cache), None);
    }

    #[test]
    fn requested_changes_set_and_get() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let history_cache = MockHistoryCache {
            seq_num_max: Some(5),
        };

        let mut reader_locator = ReaderLocator::new(locator, false);

        let requested_changes = [1, 3, 5];
        reader_locator.requested_changes_set(&requested_changes, &history_cache);
        assert_eq!(reader_locator.requested_changes(), requested_changes)
    }

    #[test]
    fn requested_changes_out_of_history_cache_range_set_and_get() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let history_cache = MockHistoryCache {
            seq_num_max: Some(2),
        };

        let mut reader_locator = ReaderLocator::new(locator, false);

        let requested_changes = [1, 3, 5];
        let expected_requested_changes = [1];
        reader_locator.requested_changes_set(&requested_changes, &history_cache);
        assert_eq!(
            reader_locator.requested_changes(),
            expected_requested_changes
        )
    }

    #[test]
    fn next_requested_change() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let history_cache = MockHistoryCache {
            seq_num_max: Some(5),
        };

        let mut reader_locator = ReaderLocator::new(locator, false);

        reader_locator.requested_changes_set(&[1, 3, 5], &history_cache);
        assert_eq!(reader_locator.next_requested_change(), Some(1));
        assert_eq!(reader_locator.next_requested_change(), Some(3));
        assert_eq!(reader_locator.next_requested_change(), Some(5));
        assert_eq!(reader_locator.next_requested_change(), None);
    }

    #[test]
    fn next_requested_change_with_multiple_and_repeated_requested_changes() {
        let locator = Locator::new_udpv4(7400, [127, 0, 0, 1]);
        let history_cache = MockHistoryCache {
            seq_num_max: Some(5),
        };

        let mut reader_locator = ReaderLocator::new(locator, false);

        reader_locator.requested_changes_set(&[1, 3, 5], &history_cache);
        reader_locator.next_requested_change();
        reader_locator.requested_changes_set(&[2, 3, 4], &history_cache);

        assert_eq!(reader_locator.next_requested_change(), Some(2));
        assert_eq!(reader_locator.next_requested_change(), Some(3));
        assert_eq!(reader_locator.next_requested_change(), Some(4));
        assert_eq!(reader_locator.next_requested_change(), Some(5));
        assert_eq!(reader_locator.next_requested_change(), None);
    }
}
