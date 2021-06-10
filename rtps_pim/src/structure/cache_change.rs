use crate::messages::types::ParameterIdPIM;

use super::types::{
    ChangeKind, DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, ParameterListPIM,
    SequenceNumberPIM, GUIDPIM,
};

pub trait RTPSCacheChange<
    PSM: GuidPrefixPIM
        + EntityIdPIM
        + InstanceHandlePIM
        + SequenceNumberPIM
        + DataPIM
        + ParameterIdPIM
        + GUIDPIM
        + ParameterListPIM<PSM>,
>
{
    fn kind(&self) -> ChangeKind;
    fn writer_guid(&self) -> &PSM::GUIDType;
    fn instance_handle(&self) -> &PSM::InstanceHandleType;
    fn sequence_number(&self) -> &PSM::SequenceNumberType;
    fn data_value(&self) -> &PSM::DataType;
    fn inline_qos(&self) -> &PSM::ParameterListType;
}
