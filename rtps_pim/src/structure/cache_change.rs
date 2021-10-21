use super::types::{ChangeKind, InstanceHandle, SequenceNumber, Guid};

pub struct RtpsCacheChange<P, D> {
    pub kind: ChangeKind,
    pub writer_guid: Guid,
    pub instance_handle: InstanceHandle,
    pub sequence_number: SequenceNumber,
    pub data_value: D,
    pub inline_qos: P,
}