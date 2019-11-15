use std::cmp::Ordering;
use std::sync::Mutex;

use crate::types::{GUID,SequenceNumber,ParameterList,InstanceHandle};

trait HistoryCache<D> {
    fn add_change(&self, change: CacheChange<D>);
    fn remove_change(&self);
    fn get_change(&self);
    fn get_seq_num_min(&self) -> Option<SequenceNumber>;
    fn get_seq_num_max(&self) -> Option<SequenceNumber>;
}

#[derive(Eq)]
#[allow(dead_code)]
pub struct CacheChange<D> {
    // kind: ChangeKind,
    writer_guid: GUID,
    instance_handle: InstanceHandle,
    sequence_number: SequenceNumber,
    inline_qos: ParameterList,
    data: Option<D>,
}

impl<D> PartialEq for CacheChange<D> {
    fn eq(&self, other: &Self) -> bool {
        self.writer_guid == other.writer_guid &&
        self.instance_handle == other.instance_handle &&
        self.sequence_number == other.sequence_number
    }
}

impl<D> Ord for CacheChange<D>  
    where D: Eq
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.sequence_number.cmp(&other.sequence_number)
    }
}

impl<D> PartialOrd for CacheChange<D> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.sequence_number.cmp(&other.sequence_number))
    }
}

pub struct Cache<D> {
    changes: Mutex<Vec<CacheChange<D>>>,
}

impl<D> Cache<D> {
    pub fn new() -> Cache<D> {
        Cache::<D> {
            changes: Mutex::new(Vec::new())
        }
    }
}

impl<D> HistoryCache<D> for Cache<D>
   where D: Eq 
{
    fn add_change(&self, change: CacheChange<D>) {
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