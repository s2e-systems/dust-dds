use crate::PIM;

use super::types::{ChangeKind, GUID};

pub struct RTPSCacheChange<PSM: PIM> {
    pub kind: ChangeKind,
    pub writer_guid: GUID<PSM>,
    pub instance_handle: PSM::InstanceHandle,
    pub sequence_number: PSM::SequenceNumber,
    pub data_value: PSM::Data,
    // pub inline_qos: PSM::ParameterVector,
}
