use rust_rtps_pim::structure::{
    types::{ChangeKind, SequenceNumber, GUID},
    RTPSCacheChange, RTPSCacheChangeOperations,
};
pub struct RTPSCacheChangeImpl {
    kind: ChangeKind,
    writer_guid: GUID,
    instance_handle: <Self as RTPSCacheChange>::InstanceHandleType,
    sequence_number: SequenceNumber,
    data: <Self as RTPSCacheChange>::DataType,
    inline_qos: <Self as RTPSCacheChange>::InlineQosType,
}

impl<'a> RTPSCacheChangeOperations<'a> for RTPSCacheChangeImpl {
    type DataType = &'a [u8];
    type InstanceHandleType = i32;
    type InlineQosType = ();

    fn new(
        kind: ChangeKind,
        writer_guid: GUID,
        instance_handle: Self::InstanceHandleType,
        sequence_number: SequenceNumber,
        data: Self::DataType,
        inline_qos: Self::InlineQosType,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data: data.into_iter().cloned().collect(),
            inline_qos,
        }
    }
}

impl rust_rtps_pim::structure::RTPSCacheChange for RTPSCacheChangeImpl {
    type DataType = Vec<u8>;
    type InstanceHandleType = i32;
    type InlineQosType = ();

    fn kind(&self) -> ChangeKind {
        self.kind
    }

    fn writer_guid(&self) -> &GUID {
        &self.writer_guid
    }

    fn instance_handle(&self) -> &<Self as RTPSCacheChange>::InstanceHandleType {
        &self.instance_handle
    }

    fn sequence_number(&self) -> &SequenceNumber {
        &self.sequence_number
    }

    fn data_value(&self) -> &Self::DataType {
        &self.data
    }

    fn inline_qos(&self) -> &Self::InlineQosType {
        &self.inline_qos
    }
}
