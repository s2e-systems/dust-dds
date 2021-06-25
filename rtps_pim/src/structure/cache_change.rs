use crate::messages::submessage_elements::ParameterListSubmessageElementPIM;

use super::types::{ChangeKind, SequenceNumber, GUID};

pub trait RTPSCacheChange<PSM>
where
    PSM: ParameterListSubmessageElementPIM,
{
    type DataType: AsRef<[u8]>;
    type InstanceHandleType;

    fn kind(&self) -> ChangeKind;
    fn writer_guid(&self) -> &GUID;
    fn instance_handle(&self) -> &Self::InstanceHandleType;
    fn sequence_number(&self) -> &SequenceNumber;
    fn data_value(&self) -> &Self::DataType;
    fn inline_qos(&self) -> &PSM::ParameterListSubmessageElementType;
}
