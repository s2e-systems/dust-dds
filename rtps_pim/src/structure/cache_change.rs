use crate::structure;

use super::types::GUID;

pub struct RTPSCacheChange<PSM: structure::Types> {
    pub kind: PSM::ChangeKind,
    pub writer_guid: GUID<PSM>,
    pub instance_handle: PSM::InstanceHandle,
    pub sequence_number: PSM::SequenceNumber,
    pub data_value: PSM::Data,
    pub inline_qos: PSM::ParameterVector,
}
