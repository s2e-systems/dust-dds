use std::cmp::Ordering;
use std::sync::Mutex;

use crate::types::{GUID,SequenceNumber,ParameterList,InstanceHandle};

#[derive(Eq)]
#[allow(dead_code)]
pub struct CacheChange {
    // kind: ChangeKind,
    writer_guid: GUID,
    instance_handle: InstanceHandle,
    sequence_number: SequenceNumber,
    data: Option<Vec<u8>>,
    inline_qos: ParameterList,
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
    changes: Mutex<Vec<CacheChange>>,
}

impl HistoryCache {
    pub fn new() -> HistoryCache {
        HistoryCache {
            changes: Mutex::new(Vec::new())
        }
    }

    fn add_change(&self, change: CacheChange) {
        self.changes.lock().unwrap().push(change);
    }
    
    fn remove_change(&self) {

    }
    
    fn get_change(&self) {
        
    }

    fn get_seq_num_min(&self) -> Option<SequenceNumber>{
        Some(self.changes.lock().unwrap().iter().max()?.sequence_number.clone())
    }
    fn get_seq_num_max(&self) -> Option<SequenceNumber>{
        Some(self.changes.lock().unwrap().iter().min()?.sequence_number.clone())
    }
}