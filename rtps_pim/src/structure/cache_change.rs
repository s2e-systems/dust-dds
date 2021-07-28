use super::types::{ChangeKind, SequenceNumber, GUID};

pub trait RTPSCacheChange {
    type InstanceHandleType;
    type InlineQosType;

    fn kind(&self) -> ChangeKind;
    fn writer_guid(&self) -> &GUID;
    fn instance_handle(&self) -> &Self::InstanceHandleType;
    fn sequence_number(&self) -> &SequenceNumber;
    fn data_value(&self) -> &[u8];
    fn inline_qos(&self) -> &Self::InlineQosType;
}

pub trait RTPSCacheChangeOperations {
    type InstanceHandleType;
    type InlineQosType;

    fn new(
        kind: ChangeKind,
        writer_guid: GUID,
        instance_handle: Self::InstanceHandleType,
        sequence_number: SequenceNumber,
        data: &[u8],
        inline_qos: Self::InlineQosType,
    ) -> Self;
}
