use rust_rtps_pim::{
    messages::types::ParameterIdPIM,
    structure::types::{
        ChangeKind, DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, ParameterListPIM,
        SequenceNumberPIM, GUIDPIM,
    },
};

pub trait RTPSCacheChangeImplTrait:
    InstanceHandlePIM
    + SequenceNumberPIM
    + DataPIM
    + ParameterIdPIM
    + EntityIdPIM
    + GuidPrefixPIM
    + GUIDPIM<Self>
    + ParameterListPIM<Self>
    + Sized
{
}
impl<
        T: InstanceHandlePIM
            + SequenceNumberPIM
            + DataPIM
            + ParameterIdPIM
            + EntityIdPIM
            + GuidPrefixPIM
            + GUIDPIM<Self>
            + ParameterListPIM<Self>
            + Sized,
    > RTPSCacheChangeImplTrait for T
{
}

pub struct RTPSCacheChangeImpl<PSM: RTPSCacheChangeImplTrait> {
    kind: ChangeKind,
    writer_guid: PSM::GUIDType,
    instance_handle: PSM::InstanceHandleType,
    sequence_number: PSM::SequenceNumberType,
    data: PSM::DataType,
    inline_qos: PSM::ParameterListType,
}

impl<PSM: RTPSCacheChangeImplTrait> RTPSCacheChangeImpl<PSM> {
    pub fn new(
        kind: ChangeKind,
        writer_guid: PSM::GUIDType,
        instance_handle: PSM::InstanceHandleType,
        sequence_number: PSM::SequenceNumberType,
        data: PSM::DataType,
        inline_qos: PSM::ParameterListType,
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

    fn writer_guid(&self) -> &PSM::GUIDType {
        &self.writer_guid
    }

    fn instance_handle(&self) -> &PSM::InstanceHandleType {
        &self.instance_handle
    }

    fn sequence_number(&self) -> &PSM::SequenceNumberType {
        &self.sequence_number
    }

    fn data_value(&self) -> &PSM::DataType {
        &self.data
    }

    fn inline_qos(&self) -> &PSM::ParameterListType {
        &self.inline_qos
    }
}
