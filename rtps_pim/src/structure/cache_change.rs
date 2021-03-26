use crate::RtpsPim;

pub struct RTPSCacheChange<PIM: RtpsPim, Data> {
    pub kind: PIM::ChangeKind,
    pub writer_guid: PIM::Guid,
    pub instance_handle: PIM::InstanceHandle,
    pub sequence_number: PIM::SequenceNumber,
    pub data_value: Data,
    pub inline_qos: PIM::ParameterList,
}
