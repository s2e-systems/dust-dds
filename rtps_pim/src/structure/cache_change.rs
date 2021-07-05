use super::types::{ChangeKind, SequenceNumber, GUID};

pub trait RTPSCacheChange {
    type DataType: AsRef<[u8]>;
    type InstanceHandleType;
    type InlineQosType;

    fn kind(&self) -> ChangeKind;
    fn writer_guid(&self) -> &GUID;
    fn instance_handle(&self) -> &Self::InstanceHandleType;
    fn sequence_number(&self) -> &SequenceNumber;
    fn data_value(&self) -> &Self::DataType;
    fn inline_qos(&self) -> &Self::InlineQosType;
}
