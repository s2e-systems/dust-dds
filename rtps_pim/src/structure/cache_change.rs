use crate::messages::{
    submessage_elements::ParameterListSubmessageElementPIM, types::ParameterIdPIM,
};

use super::types::{
    ChangeKind, DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, SequenceNumberPIM, GUIDPIM,
};

pub trait RTPSCacheChange<
    PSM: InstanceHandlePIM
        + SequenceNumberPIM
        + DataPIM
        + GUIDPIM<PSM>
        + ParameterListSubmessageElementPIM<PSM>,
>
{
    fn kind(&self) -> ChangeKind;
    fn writer_guid(&self) -> &PSM::GUIDType;
    fn instance_handle(&self) -> &PSM::InstanceHandleType;
    fn sequence_number(&self) -> &PSM::SequenceNumberType;
    fn data_value(&self) -> &PSM::DataType;
    fn inline_qos(&self) -> &PSM::ParameterListSubmessageElementType;
}
