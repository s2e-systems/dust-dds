use rust_rtps_pim::{
    messages::types::ParameterIdType,
    structure::types::{
        ChangeKind, DataType, EntityIdType, GuidPrefixType, InstanceHandleType, ParameterListType,
        SequenceNumberType, GUID,
    },
};

pub trait RTPSCacheChangeImplTrait:
    InstanceHandleType
    + SequenceNumberType
    + DataType
    + ParameterIdType
    + EntityIdType
    + GuidPrefixType
    + ParameterListType<Self>
    + Sized
{
}
impl<
        T: InstanceHandleType
            + SequenceNumberType
            + DataType
            + ParameterIdType
            + EntityIdType
            + GuidPrefixType
            + ParameterListType<Self>
            + Sized,
    > RTPSCacheChangeImplTrait for T
{
}

pub struct RTPSCacheChangeImpl<PSM: RTPSCacheChangeImplTrait> {
    kind: ChangeKind,
    writer_guid: GUID<PSM>,
    instance_handle: PSM::InstanceHandle,
    sequence_number: PSM::SequenceNumber,
    data: PSM::Data,
    inline_qos: PSM::ParameterList,
}

impl<PSM: RTPSCacheChangeImplTrait> rust_rtps_pim::structure::RTPSCacheChange<PSM>
    for RTPSCacheChangeImpl<PSM>
{
    fn kind(&self) -> &ChangeKind {
        &self.kind
    }

    fn writer_guid(&self) -> &GUID<PSM> {
        &self.writer_guid
    }

    fn instance_handle(&self) -> &PSM::InstanceHandle {
        &self.instance_handle
    }

    fn sequence_number(&self) -> &PSM::SequenceNumber {
        &self.sequence_number
    }

    fn data_value(&self) -> &PSM::Data {
        &self.data
    }

    fn inline_qos(&self) -> &PSM::ParameterList {
        &self.inline_qos
    }
}
