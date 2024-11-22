use crate::rtps::messages::submessage_elements::{Data, ParameterList};

use super::types::{ChangeKind, SequenceNumber, Time};

#[derive(Debug)]
pub struct RtpsCacheChange {
    pub kind: ChangeKind,
    pub sequence_number: SequenceNumber,
    pub source_timestamp: Option<Time>,
    pub data_value: Data,
    pub inline_qos: ParameterList,
}

impl RtpsCacheChange {
    pub fn kind(&self) -> ChangeKind {
        self.kind
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }

    pub fn source_timestamp(&self) -> Option<Time> {
        self.source_timestamp
    }

    pub fn data_value(&self) -> &Data {
        &self.data_value
    }

    pub fn inline_qos(&self) -> &ParameterList {
        &self.inline_qos
    }
}

pub trait WriterHistoryCache: Send + Sync {
    fn guid(&self) -> [u8; 16];

    fn add_change(&mut self, cache_change: RtpsCacheChange);

    fn remove_change(&mut self, sequence_number: SequenceNumber);

    fn is_change_acknowledged(&self, sequence_number: SequenceNumber) -> bool;
}
