use rust_rtps_pim::{
    messages::types::ParameterIdType,
    structure::types::{
        ChangeKind, DataType, EntityIdType, GUIDType, GuidPrefixType, InstanceHandleType,
        ParameterListType, SequenceNumberType,
    },
};

pub trait RTPSCacheChangeImplTrait:
    InstanceHandleType
    + SequenceNumberType
    + DataType
    + ParameterIdType
    + EntityIdType
    + GuidPrefixType
    + GUIDType<Self>
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
            + GUIDType<Self>
            + ParameterListType<Self>
            + Sized,
    > RTPSCacheChangeImplTrait for T
{
}

pub struct RTPSCacheChangeImpl<PSM: RTPSCacheChangeImplTrait> {
    kind: ChangeKind,
    writer_guid: PSM::GUID,
    instance_handle: PSM::InstanceHandle,
    sequence_number: PSM::SequenceNumber,
    data: PSM::Data,
    inline_qos: PSM::ParameterList,
}

impl<PSM: RTPSCacheChangeImplTrait> RTPSCacheChangeImpl<PSM> {
    pub fn new(
        kind: ChangeKind,
        writer_guid: PSM::GUID,
        instance_handle: PSM::InstanceHandle,
        sequence_number: PSM::SequenceNumber,
        data: PSM::Data,
        inline_qos: PSM::ParameterList,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data,
            inline_qos,
        }
    }
}

impl<PSM: RTPSCacheChangeImplTrait> rust_rtps_pim::structure::RTPSCacheChange<PSM>
    for RTPSCacheChangeImpl<PSM>
{
    fn kind(&self) -> &ChangeKind {
        &self.kind
    }

    fn writer_guid(&self) -> &PSM::GUID {
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
