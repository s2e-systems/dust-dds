use crate::rtps::messages::submessage_elements::{Data, ParameterList};

use super::types::{ChangeKind, SequenceNumber, Time};

#[derive(Debug)]
pub struct CacheChange {
    pub kind: ChangeKind,
    pub writer_guid: [u8; 16],
    pub sequence_number: SequenceNumber,
    pub source_timestamp: Option<Time>,
    pub data_value: Data,
    pub inline_qos: ParameterList,
}

impl CacheChange {
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
