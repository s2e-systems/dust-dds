use rust_rtps::{
    messages::submessages::submessage_elements::ParameterList,
    structure::RTPSCacheChange,
    types::{ChangeKind, InstanceHandle, SequenceNumber, GUID},
};

pub struct CacheChange {
    kind: ChangeKind,
    writer_guid: GUID,
    instance_handle: InstanceHandle,
    sequence_number: SequenceNumber,
    data_value: <Self as RTPSCacheChange>::Data,
    inline_qos: ParameterList,
}

impl RTPSCacheChange for CacheChange {
    type Data = Vec<u8>;

    fn new(
        kind: ChangeKind,
        writer_guid: GUID,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        data_value: Self::Data,
        inline_qos: ParameterList,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data_value,
            inline_qos,
        }
    }

    fn kind(&self) -> ChangeKind {
        self.kind
    }

    fn writer_guid(&self) -> GUID {
        self.writer_guid
    }

    fn instance_handle(&self) -> &InstanceHandle {
        &self.instance_handle
    }

    fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }

    fn data_value(&self) -> &Self::Data {
        &self.data_value
    }

    fn inline_qos(&self) -> &ParameterList {
        &self.inline_qos
    }
}
