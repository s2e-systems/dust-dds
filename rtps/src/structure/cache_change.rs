use crate::{
    messages::submessages::submessage_elements::ParameterList,
    types::{ChangeKind, InstanceHandle, SequenceNumber, GUID},
};

pub trait RTPSCacheChange : 'static {
    type Data;

    fn new(
        kind: ChangeKind,
        writer_guid: GUID,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        data_value: Self::Data,
        inline_qos: ParameterList,
    ) -> Self;
    fn kind(&self) -> ChangeKind;
    fn writer_guid(&self) -> GUID;
    fn instance_handle(&self) -> &InstanceHandle;
    fn sequence_number(&self) -> SequenceNumber;
    fn data_value(&self) -> &Self::Data;
    fn inline_qos(&self) -> &ParameterList;
}
