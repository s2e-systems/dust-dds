use alloc::sync::Arc;

use super::types::{ChangeKind, Guid, Time};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheChange {
    pub kind: ChangeKind,
    pub writer_guid: Guid,
    pub sequence_number: i64,
    pub source_timestamp: Option<Time>,
    pub instance_handle: Option<[u8; 16]>,
    pub data_value: Arc<[u8]>,
}

impl CacheChange {
    pub fn kind(&self) -> ChangeKind {
        self.kind
    }

    pub fn sequence_number(&self) -> i64 {
        self.sequence_number
    }

    pub fn source_timestamp(&self) -> Option<Time> {
        self.source_timestamp
    }

    pub fn data_value(&self) -> &Arc<[u8]> {
        &self.data_value
    }
}

pub trait HistoryCache: Send {
    fn add_change(&mut self, cache_change: CacheChange);

    fn remove_change(&mut self, sequence_number: i64);
}
