use super::{
    behavior_types::InstanceHandle,
    messages::{
        self,
        submessage_elements::{Data, ParameterList},
        submessages::data::DataSubmessage,
    },
    types::{ChangeKind, EntityId, Guid, SequenceNumber},
};

pub struct RtpsWriterCacheChange {
    kind: ChangeKind,
    writer_guid: Guid,
    sequence_number: SequenceNumber,
    instance_handle: InstanceHandle,
    timestamp: messages::types::Time,
    data_value: Data,
    inline_qos: ParameterList,
}

impl RtpsWriterCacheChange {
    pub fn as_data_submessage(&self, reader_id: EntityId) -> DataSubmessage {
        let (data_flag, key_flag) = match self.kind() {
            ChangeKind::Alive => (true, false),
            ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => (false, true),
            _ => todo!(),
        };

        DataSubmessage::new(
            true,
            data_flag,
            key_flag,
            false,
            reader_id,
            self.writer_guid().entity_id(),
            self.sequence_number(),
            self.inline_qos.clone(),
            self.data_value.clone(),
        )
    }

    pub fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }
}

impl RtpsWriterCacheChange {
    pub fn new(
        kind: ChangeKind,
        writer_guid: Guid,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        timestamp: messages::types::Time,
        data_value: Data,
        inline_qos: ParameterList,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            sequence_number,
            instance_handle,
            timestamp,
            data_value,
            inline_qos,
        }
    }
}

impl RtpsWriterCacheChange {
    pub fn kind(&self) -> ChangeKind {
        self.kind
    }

    pub fn writer_guid(&self) -> Guid {
        self.writer_guid
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }

    pub fn timestamp(&self) -> messages::types::Time {
        self.timestamp
    }

    pub fn data_value(&self) -> &Data {
        &self.data_value
    }

    pub fn inline_qos(&self) -> &ParameterList {
        &self.inline_qos
    }
}
