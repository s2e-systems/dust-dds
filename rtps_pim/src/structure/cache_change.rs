use crate::RtpsPim;

pub struct RTPSCacheChange<PSM: RtpsPim, Data> {
    pub kind: PSM::ChangeKind,
    pub writer_guid: PSM::Guid,
    pub instance_handle: PSM::InstanceHandle,
    pub sequence_number: PSM::SequenceNumber,
    pub data_value: Data,
    pub inline_qos: PSM::ParameterList,
}
