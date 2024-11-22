use crate::rtps::messages::submessage_elements::{Data, ParameterList};

use super::types::{ChangeKind, SequenceNumber, Time};

#[derive(Clone)]
pub struct ReaderCacheChange {
    pub kind: ChangeKind,
    pub writer_guid: [u8; 16],
    pub sequence_number: SequenceNumber,
    pub source_timestamp: Option<Time>,
    pub data_value: Data,
    pub inline_qos: ParameterList,
}

pub trait ReaderHistoryCache: Send + Sync {
    fn add_change(&mut self, cache_change: ReaderCacheChange);
}

pub trait TransportReader: Send + Sync {
    fn guid(&self) -> [u8; 16];
    fn is_historical_data_received(&self) -> bool;
}
