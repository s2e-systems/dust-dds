use crate::{
    messages::submessages::submessage_elements::{ParameterList, SerializedData},
    types::{ChangeKind, InstanceHandle, SequenceNumber, GUID},
};

pub trait CacheChange {
    fn new(kind: ChangeKind, writer_guid: GUID, instance_handle: InstanceHandle, sequence_number: SequenceNumber, data_value: SerializedData, inline_qos: ParameterList ) -> Self;
    fn kind(&self) -> ChangeKind;
    fn writer_guid(&self) -> GUID;
    fn instance_handle(&self) -> InstanceHandle;
    fn sequence_number(&self) -> SequenceNumber;
    fn data_value(&self) -> &SerializedData; /*Originally in the standard Data*/
    fn inline_qos(&self) -> &ParameterList;
}
