use std::collections::{HashSet};
use std::sync::{Mutex, MutexGuard};

use crate::types::{SequenceNumber, };

use super::cache_change::CacheChange;

pub struct HistoryCache {
    changes: Mutex<HashSet<CacheChange>>,
}

impl HistoryCache {
    pub fn new() -> Self {
        HistoryCache {
            changes: Mutex::new(HashSet::new()),
        }
    }

    pub fn add_change(&self, change: CacheChange) {
        self.changes().insert(change);
    }

    pub fn remove_change(&self, change: &CacheChange) {
        self.changes().remove(change);
    }

    pub fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        Some(self.changes().iter().min()?.sequence_number())
    }

    pub fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        Some(self.changes().iter().max()?.sequence_number())
    }

    pub fn changes(&self) -> MutexGuard<HashSet<CacheChange>> {
        self.changes.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EntityId, EntityKind, GUID, ChangeKind};

    #[test]
    fn cache_change_list() {
        let history_cache = HistoryCache::new();
        let guid_prefix = [8; 12];
        let entity_id = EntityId::new([1, 2, 3], EntityKind::BuiltInReaderWithKey);
        let guid = GUID::new(guid_prefix, entity_id);
        let instance_handle = [9; 16];
        let sequence_number = 1;
        let data_value = Some(vec![4, 5, 6]);
        let cc = CacheChange::new(
            ChangeKind::Alive,
            guid,
            instance_handle,
            sequence_number,
            data_value,
            None,
        );

        let cc_clone = cc.clone();

        assert_eq!(history_cache.changes().len(), 0);
        history_cache.add_change(cc);
        assert_eq!(history_cache.changes().len(), 1);
        history_cache.remove_change(&cc_clone);
        assert_eq!(history_cache.changes().len(), 0);
    }

    #[test]
    fn cache_change_sequence_number() {
        let history_cache = HistoryCache::new();

        let guid_prefix = [8; 12];
        let entity_id = EntityId::new([1, 2, 3], EntityKind::BuiltInReaderWithKey);
        let guid = GUID::new(guid_prefix, entity_id);
        let instance_handle = [9; 16];
        let data_value = Some(vec![4, 5, 6]);
        let sequence_number_min = 1;
        let sequence_number_max = 2;
        let cc1 = CacheChange::new(
            ChangeKind::Alive,
            guid.clone(),
            instance_handle,
            sequence_number_min,
            data_value.clone(),
            None,
        );
        let cc2 = CacheChange::new(
            ChangeKind::Alive,
            guid.clone(),
            instance_handle,
            sequence_number_max,
            data_value.clone(),
            None,
        );

        assert_eq!(history_cache.get_seq_num_max(), None);
        history_cache.add_change(cc1);
        assert_eq!(
            history_cache.get_seq_num_min(),
            history_cache.get_seq_num_max()
        );
        history_cache.add_change(cc2);
        assert_eq!(history_cache.get_seq_num_min(), Some(sequence_number_min));
        assert_eq!(history_cache.get_seq_num_max(), Some(sequence_number_max));
    }
}
