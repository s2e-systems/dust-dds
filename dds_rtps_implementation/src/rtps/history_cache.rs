use std::{
    ops::Deref,
    sync::{Mutex, MutexGuard},
};

use rust_rtps::{
    structure::{RTPSCacheChange, RTPSHistoryCache},
    types::SequenceNumber,
};

pub struct HistoryCache<T: RTPSCacheChange> {
    changes: Mutex<Vec<T>>,
}

pub struct BorrowedCacheChange<'a, T: RTPSCacheChange> {
    guard: MutexGuard<'a, Vec<T>>,
    index: usize,
}

impl<'a, T: RTPSCacheChange> Deref for BorrowedCacheChange<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard[self.index]
    }
}

impl<'a, T: RTPSCacheChange + 'a> RTPSHistoryCache<'a> for HistoryCache<T> {
    type CacheChangeType = T;
    type CacheChangeReadType = BorrowedCacheChange<'a, T>;

    fn new() -> Self {
        Self {
            changes: Mutex::new(Vec::new()),
        }
    }

    fn add_change(&self, change: Self::CacheChangeType) {
        self.changes.lock().unwrap().push(change)
    }

    fn remove_change(&self, seq_num: SequenceNumber) {
        self.changes
            .lock()
            .unwrap()
            .retain(|cc| cc.sequence_number() != seq_num)
    }

    fn get_change(&'a self, seq_num: SequenceNumber) -> Option<Self::CacheChangeReadType> {
        let guard = self.changes.lock().unwrap();
        let index = guard
            .iter()
            .position(|cc| cc.sequence_number() == seq_num)?;
        Some(BorrowedCacheChange { guard, index })
    }

    fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.changes
            .lock()
            .unwrap()
            .iter()
            .map(|cc| cc.sequence_number())
            .min()
    }

    fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.changes
            .lock()
            .unwrap()
            .iter()
            .map(|cc| cc.sequence_number())
            .max()
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
        let history_cache = HistoryCache::new();
        let cc1 = MockCacheChange { sequence_number: 1 };
        let cc2 = MockCacheChange { sequence_number: 2 };
        history_cache.add_change(cc1.clone());
        history_cache.add_change(cc2.clone());

        assert_eq!(*history_cache.get_change(1).unwrap(), cc1);
        assert_eq!(*history_cache.get_change(2).unwrap(), cc2);
        assert!(history_cache.get_change(3).is_none());
    }

    #[test]
    fn remove_change() {
        let history_cache = HistoryCache::new();
        let cc1 = MockCacheChange { sequence_number: 1 };
        let cc2 = MockCacheChange { sequence_number: 2 };
        history_cache.add_change(cc1.clone());
        history_cache.add_change(cc2.clone());
        history_cache.remove_change(1);

        assert!(history_cache.get_change(1).is_none());
        assert_eq!(*history_cache.get_change(2).unwrap(), cc2);
    }

    #[test]
    fn get_seq_num_min_and_max() {
        let history_cache = HistoryCache::new();
        let min_seq_num = 4;
        let max_seq_num = 6;
        let cc_min = MockCacheChange {
            sequence_number: min_seq_num,
        };
        let cc_max = MockCacheChange {
            sequence_number: max_seq_num,
        };
        history_cache.add_change(cc_min);
        history_cache.add_change(cc_max);

        assert_eq!(history_cache.get_seq_num_max(), Some(max_seq_num));
        assert_eq!(history_cache.get_seq_num_min(), Some(min_seq_num));
    }
}
