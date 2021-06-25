use crate::messages::submessage_elements::ParameterListSubmessageElementPIM;

use super::types::{ChangeKind, InstanceHandlePIM, SequenceNumber, GUID};

pub trait RTPSCacheChange<PSM>
where
    PSM: InstanceHandlePIM
        + ParameterListSubmessageElementPIM,
{
    type DataType: AsRef<[u8]>;

    fn kind(&self) -> ChangeKind;
    fn writer_guid(&self) -> &GUID;
    fn instance_handle(&self) -> &PSM::InstanceHandleType;
    fn sequence_number(&self) -> &SequenceNumber;
    fn data_value(&self) -> &Self::DataType;
    fn inline_qos(&self) -> &PSM::ParameterListSubmessageElementType;
}
