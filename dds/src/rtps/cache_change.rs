use super::{
    messages::{
        self,
        submessage_elements::{Data, ParameterList},
        submessages::data::DataSubmessage,
    },
    types::{ChangeKind, EntityId, SequenceNumber},
};

#[derive(Debug)]
pub struct RtpsCacheChange {
    pub kind: ChangeKind,
    pub sequence_number: SequenceNumber,
    pub source_timestamp: Option<messages::types::Time>,
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

    pub fn source_timestamp(&self) -> Option<messages::types::Time> {
        self.source_timestamp
    }

    pub fn data_value(&self) -> &Data {
        &self.data_value
    }

    pub fn inline_qos(&self) -> &ParameterList {
        &self.inline_qos
    }
}

impl RtpsCacheChange {
    pub fn as_data_submessage(&self, reader_id: EntityId, writer_id: EntityId) -> DataSubmessage {
        let (data_flag, key_flag) = match self.kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => (true, false),
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => (false, true),
        };

        DataSubmessage::new(
            true,
            data_flag,
            key_flag,
            false,
            reader_id,
            writer_id,
            self.sequence_number,
            self.inline_qos.clone(),
            self.data_value.clone(),
        )
    }
}
