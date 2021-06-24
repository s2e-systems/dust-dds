use rust_rtps_pim::{
    messages::submessage_elements::ParameterListSubmessageElementPIM,
    structure::types::{ChangeKind, DataPIM, InstanceHandlePIM, SequenceNumber, GUID},
};
pub struct RTPSCacheChangeImpl<PSM>
where
    PSM: InstanceHandlePIM + DataPIM + ParameterListSubmessageElementPIM,
{
    kind: ChangeKind,
    writer_guid: GUID,
    instance_handle: PSM::InstanceHandleType,
    sequence_number: SequenceNumber,
    data: PSM::DataType,
    inline_qos: PSM::ParameterListSubmessageElementType,
}

impl<PSM> RTPSCacheChangeImpl<PSM>
where
    PSM: InstanceHandlePIM + DataPIM + ParameterListSubmessageElementPIM,
{
    pub fn new(
        kind: ChangeKind,
        writer_guid: GUID,
        instance_handle: PSM::InstanceHandleType,
        sequence_number: SequenceNumber,
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
    PSM: InstanceHandlePIM + DataPIM + ParameterListSubmessageElementPIM,
{
    fn kind(&self) -> ChangeKind {
        self.kind
    }

    fn writer_guid(&self) -> &GUID {
        &self.writer_guid
    }

    fn instance_handle(&self) -> &PSM::InstanceHandleType {
        &self.instance_handle
    }

    fn sequence_number(&self) -> &SequenceNumber {
        &self.sequence_number
    }

    fn data_value(&self) -> &PSM::DataType {
        &self.data
    }

    fn inline_qos(&self) -> &PSM::ParameterListSubmessageElementType {
        &self.inline_qos
    }
}
