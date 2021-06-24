use rust_rtps_pim::{
    messages::submessage_elements::ParameterListSubmessageElementPIM,
    structure::types::{ChangeKind, DataPIM, InstanceHandlePIM, SequenceNumberPIM, GUIDPIM},
};
pub struct RTPSCacheChangeImpl<PSM>
where
    PSM: GUIDPIM<PSM>
        + InstanceHandlePIM
        + SequenceNumberPIM
        + DataPIM
        + ParameterListSubmessageElementPIM<PSM>,
{
    kind: ChangeKind,
    writer_guid: PSM::GUIDType,
    instance_handle: PSM::InstanceHandleType,
    sequence_number: PSM::SequenceNumberType,
    data: PSM::DataType,
    inline_qos: PSM::ParameterListSubmessageElementType,
}

impl<PSM> RTPSCacheChangeImpl<PSM>
where
    PSM: GUIDPIM<PSM>
        + InstanceHandlePIM
        + SequenceNumberPIM
        + DataPIM
        + ParameterListSubmessageElementPIM<PSM>,
{
    pub fn new(
        kind: ChangeKind,
        writer_guid: PSM::GUIDType,
        instance_handle: PSM::InstanceHandleType,
        sequence_number: PSM::SequenceNumberType,
        data: PSM::DataType,
        inline_qos: PSM::ParameterListSubmessageElementType,
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

impl<PSM> rust_rtps_pim::structure::RTPSCacheChange<PSM> for RTPSCacheChangeImpl<PSM>
where
    PSM: GUIDPIM<PSM>
        + InstanceHandlePIM
        + SequenceNumberPIM
        + DataPIM
        + ParameterListSubmessageElementPIM<PSM>,
{
    fn kind(&self) -> ChangeKind {
        self.kind
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

    fn inline_qos(&self) -> &PSM::ParameterListSubmessageElementType {
        &self.inline_qos
    }
}
