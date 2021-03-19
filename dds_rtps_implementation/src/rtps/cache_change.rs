use rust_dds_api::dcps_psm::InstanceHandle;
use rust_rtps::{
    messages::submessages::submessage_elements::{Parameter, ParameterList, SubmessageElement},
    structure::RTPSCacheChange,
    types::{ChangeKind, SequenceNumber, GUID},
};

pub struct MyParameterList {
    parameter: Vec<Box<dyn Parameter>>,
}

impl MyParameterList {
    pub fn new() -> Self {
        Self {
            parameter: Vec::new(),
        }
    }
}

impl SubmessageElement for MyParameterList {}

impl ParameterList for MyParameterList {
    fn parameter(&self) -> &[Box<dyn Parameter>] {
        &self.parameter
    }
}

pub struct CacheChange {
    kind: ChangeKind,
    writer_guid: GUID,
    instance_handle: InstanceHandle,
    sequence_number: SequenceNumber,
    data_value: <Self as RTPSCacheChange>::Data,
    inline_qos: <Self as RTPSCacheChange>::ParameterList,
}

impl RTPSCacheChange for CacheChange {
    type Data = Vec<u8>;
    type InstanceHandle = InstanceHandle;
    type ParameterList = MyParameterList;

    fn new(
        kind: ChangeKind,
        writer_guid: GUID,
        instance_handle: Self::InstanceHandle,
        sequence_number: SequenceNumber,
        data_value: Self::Data,
        inline_qos: Self::ParameterList,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data_value,
            inline_qos,
        }
    }

    fn kind(&self) -> ChangeKind {
        self.kind
    }

    fn writer_guid(&self) -> GUID {
        self.writer_guid
    }

    fn instance_handle(&self) -> &Self::InstanceHandle {
        &self.instance_handle
    }

    fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }

    fn data_value(&self) -> &Self::Data {
        &self.data_value
    }

    fn inline_qos(&self) -> &Self::ParameterList {
        &self.inline_qos
    }
}
