use rust_rtps::{
    structure::{RTPSCacheChange, RTPSHistoryCache},
    types::SequenceNumber,
};

pub struct HistoryCache<T: RTPSCacheChange> {
    changes: Vec<T>,
}

impl<T: RTPSCacheChange> RTPSHistoryCache for HistoryCache<T> {
    type CacheChangeType = T;

    fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    fn add_change(&mut self, change: Self::CacheChangeType) {
        self.changes.push(change)
    }

    fn remove_change(&mut self, seq_num: SequenceNumber) {
        self.changes.retain(|cc| cc.sequence_number() != seq_num)
    }

    fn get_change(&self, seq_num: SequenceNumber) -> Option<&Self::CacheChangeType> {
        self.changes
            .iter()
            .find(|cc| cc.sequence_number() == seq_num)
    }

    fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.changes.iter().map(|cc| cc.sequence_number()).min()
    }

    fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.changes.iter().map(|cc| cc.sequence_number()).max()
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps::{
        messages::submessages::submessage_elements::ParameterList,
        types::{ChangeKind, InstanceHandle, GUID},
    };

    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct MockCacheChange {
        sequence_number: SequenceNumber,
    }

    impl RTPSCacheChange for MockCacheChange {
        type Data = ();

        fn new(
            _kind: ChangeKind,
            _writer_guid: GUID,
            _instance_handle: InstanceHandle,
            _sequence_number: SequenceNumber,
            _data_value: Self::Data,
            _inline_qos: ParameterList,
        ) -> Self {
            todo!()
        }

        fn kind(&self) -> ChangeKind {
            todo!()
        }

        fn writer_guid(&self) -> GUID {
            todo!()
        }

        fn instance_handle(&self) -> &InstanceHandle {
            todo!()
        }

        fn sequence_number(&self) -> SequenceNumber {
            self.sequence_number
        }

        fn data_value(&self) -> &Self::Data {
            todo!()
        }

        fn inline_qos(&self) -> &ParameterList {
            todo!()
        }
    }
    #[test]
    fn add_and_get_change() {
        let mut history_cache = HistoryCache::new();
        let cc1 = MockCacheChange { sequence_number: 1 };
        let cc2 = MockCacheChange { sequence_number: 2 };
        history_cache.add_change(cc1.clone());
        history_cache.add_change(cc2.clone());

        assert_eq!(history_cache.get_change(1), Some(&cc1));
        assert_eq!(history_cache.get_change(2), Some(&cc2));
        assert_eq!(history_cache.get_change(3), None);
    }

    #[test]
    fn remove_change() {
        let mut history_cache = HistoryCache::new();
        let cc1 = MockCacheChange { sequence_number: 1 };
        let cc2 = MockCacheChange { sequence_number: 2 };
        history_cache.add_change(cc1.clone());
        history_cache.add_change(cc2.clone());
        history_cache.remove_change(1);

        assert_eq!(history_cache.get_change(1), None);
        assert_eq!(history_cache.get_change(2), Some(&cc2));
    }

    #[test]
    fn get_seq_num_min_and_max() {
        let mut history_cache = HistoryCache::new();
        let min_seq_num = 4;
        let max_seq_num = 6;
        let cc_min = MockCacheChange { sequence_number: min_seq_num };
        let cc_max = MockCacheChange { sequence_number: max_seq_num };
        history_cache.add_change(cc_min);
        history_cache.add_change(cc_max);

        assert_eq!(history_cache.get_seq_num_max(),Some(max_seq_num));
        assert_eq!(history_cache.get_seq_num_min(),Some(min_seq_num));

    }
}
