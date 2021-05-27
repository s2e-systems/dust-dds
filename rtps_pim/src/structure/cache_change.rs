use crate::messages::types::ParameterIdType;

use super::types::{
    ChangeKind, DataType, EntityIdType, GUIDType, GuidPrefixType, InstanceHandleType,
    ParameterListType, SequenceNumberType,
};

pub trait RTPSCacheChange<
    PSM: GuidPrefixType
        + EntityIdType
        + InstanceHandleType
        + SequenceNumberType
        + DataType
        + ParameterIdType
        + GUIDType<PSM>
        + ParameterListType<PSM>,
>
{
    fn kind(&self) -> &ChangeKind;
    fn writer_guid(&self) -> &PSM::GUID;
    fn instance_handle(&self) -> &PSM::InstanceHandle;
    fn sequence_number(&self) -> &PSM::SequenceNumber;
    fn data_value(&self) -> &PSM::Data;
    fn inline_qos(&self) -> &PSM::ParameterList;
}
