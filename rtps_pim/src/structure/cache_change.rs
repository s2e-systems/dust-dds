use crate::messages::types::ParameterIdType;

use super::types::{
    ChangeKind, DataType, EntityIdType, GuidPrefixType, InstanceHandleType, ParameterListType,
    SequenceNumberType, GUID,
};

pub struct RTPSCacheChange<
    PSM: GuidPrefixType
        + EntityIdType
        + InstanceHandleType
        + SequenceNumberType
        + DataType
        + ParameterIdType
        + ParameterListType<PSM>,
> {
    pub kind: ChangeKind,
    pub writer_guid: GUID<PSM>,
    pub instance_handle: PSM::InstanceHandle,
    pub sequence_number: PSM::SequenceNumber,
    pub data_value: PSM::Data,
    pub inline_qos: PSM::ParameterList,
}
