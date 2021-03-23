use crate::{
    messages::submessages::submessage_elements::ParameterList,
    types::{ChangeKind, EntityId, GuidPrefix, InstanceHandle, SequenceNumber, GUID},
};

pub trait RTPSCacheChange {
    type InstanceHandle: InstanceHandle;
    type SequenceNumber: SequenceNumber;
    type GuidPrefix: GuidPrefix;
    type EntityId: EntityId;
    type GUID: GUID<GuidPrefix = Self::GuidPrefix, EntityId = Self::EntityId>;
    type Data;
    type ParameterList: ParameterList;

    fn new(
        kind: ChangeKind,
        writer_guid: Self::GUID,
        instance_handle: Self::InstanceHandle,
        sequence_number: Self::SequenceNumber,
        data_value: Self::Data,
        inline_qos: Self::ParameterList,
    ) -> Self;
    fn kind(&self) -> ChangeKind;
    fn writer_guid(&self) -> Self::GUID;
    fn instance_handle(&self) -> Self::InstanceHandle;
    fn sequence_number(&self) -> Self::SequenceNumber;
    fn data_value(&self) -> &Self::Data;
    fn inline_qos(&self) -> &Self::ParameterList;
}
