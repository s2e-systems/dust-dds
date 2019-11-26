use std::cmp::Ordering;
use std::sync::{RwLock,Mutex};
use std::collections::HashMap;

use crate::types::{GUID,SequenceNumber,ParameterList,InstanceHandle};

#[derive(Eq, Default)]
#[allow(dead_code)]
pub struct CacheChange {
    // kind: ChangeKind,
    writer_guid: GUID,
    instance_handle: InstanceHandle,
    sequence_number: SequenceNumber,
    data: Option<Vec<u8>>,
    inline_qos: ParameterList,
}

impl CacheChange {
    pub fn new(writer_guid: GUID, instance_handle: InstanceHandle, sequence_number: SequenceNumber, data: Option<Vec<u8>>, inline_qos: ParameterList) -> CacheChange {
        CacheChange {
            writer_guid,
            instance_handle,
            sequence_number,
            data,
            inline_qos,
        }
    }
}

impl PartialEq for CacheChange {
    fn eq(&self, other: &Self) -> bool {
        self.writer_guid == other.writer_guid &&
        self.instance_handle == other.instance_handle &&
        self.sequence_number == other.sequence_number
    }
}

impl Ord for CacheChange
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.sequence_number.cmp(&other.sequence_number)
    }
}

impl PartialOrd for CacheChange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.sequence_number.cmp(&other.sequence_number))
    }
}

pub struct HistoryCache {
    changes: RwLock<HashMap<InstanceHandle, Mutex<Vec<CacheChange>>>>,
}

impl HistoryCache {
    pub fn new() -> HistoryCache {
        HistoryCache {
            changes: RwLock::new(HashMap::with_capacity(1))
        }
    }

    pub fn add_change(&self, change: CacheChange) -> Result<(),()> {
        // If key doesn't exist then create a new entry for it
        if !self.has_key(&change.instance_handle) {
            let mut map_write_lock = self.changes.write().unwrap();
            map_write_lock.insert(change.instance_handle, Mutex::new(Vec::new()));
        }

        self.changes.read().unwrap()[&change.instance_handle].lock().unwrap().push(change);

        Ok(())
    }
    
    pub fn remove_change(&self, key: &InstanceHandle, sequence_number: &SequenceNumber) {
        if self.has_key(&key) {
            let map_read_lock = &self.changes.read().unwrap()[key];
            let mut vector_lock = map_read_lock.lock().unwrap();
            vector_lock.retain(|x| x.sequence_number != *sequence_number);
        }
    }
    
    pub fn get_change(&self, key: &InstanceHandle, sequence_number: &SequenceNumber) {
        unimplemented!()
    }

    pub fn remove_instance(&self, key: &InstanceHandle) {
        if self.has_key(&key) {
            let mut map_write_lock = self.changes.write().unwrap();
            map_write_lock.remove(key);
        }
    }

    pub fn get_seq_num_min(&self, key: &InstanceHandle) -> Option<SequenceNumber>{
        Some(self.changes.read().unwrap()[key].lock().unwrap().iter().max()?.sequence_number.clone())
    }

    pub fn get_seq_num_max(&self, key: &InstanceHandle) -> Option<SequenceNumber>{
        Some(self.changes.read().unwrap()[key].lock().unwrap().iter().min()?.sequence_number.clone())
    }

    fn has_key(&self, key: &InstanceHandle) -> bool{
        self.changes.read().unwrap().contains_key(key)
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_create_history_cache() {
        let empty_history_cache = HistoryCache::new();

        assert!(empty_history_cache.changes.read().unwrap().is_empty());
    }

    #[test]
    fn test_add_and_remove_cache_change() {
        let history_cache = HistoryCache::new();
        assert_eq!(history_cache.changes.read().unwrap().len(), 0);

        let mut cache_change_sn1 = CacheChange::default();
        cache_change_sn1.instance_handle = [1;16];
        cache_change_sn1.sequence_number = 1;

        history_cache.add_change(cache_change_sn1).unwrap();

        assert_eq!(history_cache.changes.read().unwrap().len(), 1);
        assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap().len(), 1);
        assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap()[0].sequence_number, 1);

        let mut cache_change_sn2 = CacheChange::default();
        cache_change_sn2.instance_handle = [1;16];
        cache_change_sn2.sequence_number = 2;

        history_cache.add_change(cache_change_sn2).unwrap();
        assert_eq!(history_cache.changes.read().unwrap().len(), 1);
        assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap().len(), 2);
        assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap()[0].sequence_number, 1);
        assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap()[1].sequence_number, 2);

        history_cache.remove_change(&[1;16], &1);
        assert_eq!(history_cache.changes.read().unwrap().len(), 1);
        assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap().len(), 1);
        assert_eq!(history_cache.changes.read().unwrap()[&[1;16]].lock().unwrap()[0].sequence_number, 2);

        history_cache.remove_instance(&[1;16]);

        assert_eq!(history_cache.changes.read().unwrap().len(), 0);
    }
}