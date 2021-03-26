use crate::{structure, RtpsPim};

pub struct RTPSCacheChange<PIM: RtpsPim, Data> {
    pub kind: <PIM as structure::Types>::ChangeKind,
    pub writer_guid: <PIM as structure::Types>::Guid,
    pub instance_handle: <PIM as structure::Types>::InstanceHandle,
    pub sequence_number: <PIM as structure::Types>::SequenceNumber,
    pub data_value: Data,
    // pub inline_qos:  <PIM as messages::Types>:: submessage_elements::ParameterList<ParameterId, ParameterValue, ParameterList>,
}
