use crate::messages::submessage_elements::ParameterListType;

use super::types::{
    ChangeKind, DataType, EntityIdType, GuidPrefixType, InstanceHandleType, SequenceNumberType,
    GUID,
};

pub struct RTPSCacheChange<
    PSM: GuidPrefixType
        + EntityIdType
        + InstanceHandleType
        + SequenceNumberType
        + DataType
        + ParameterListType,
> {
    pub kind: ChangeKind,
    pub writer_guid: GUID<PSM>,
    pub instance_handle: PSM::InstanceHandle,
    pub sequence_number: PSM::SequenceNumber,
    pub data_value: PSM::Data,
    pub inline_qos: PSM::ParameterVector,
}
