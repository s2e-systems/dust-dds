use crate::messages::types::ParameterIdType;

use super::types::{
    ChangeKind, DataType, EntityIdType, GuidPrefixType, InstanceHandleType, ParameterListType,
    SequenceNumberType, GUID,
};

pub trait RTPSCacheChange<
    PSM: GuidPrefixType
        + EntityIdType
        + InstanceHandleType
        + SequenceNumberType
        + DataType
        + ParameterIdType
        + ParameterListType<PSM>,
>
{
    fn kind(&self) -> &ChangeKind;
    fn writer_guid(&self) -> &GUID<PSM>;
    fn instance_handle(&self) -> &PSM::InstanceHandle;
    fn sequence_number(&self) -> &PSM::SequenceNumber;
    fn data_value(&self) -> &PSM::Data;
    fn inline_qos(&self) -> &PSM::ParameterList;
}
