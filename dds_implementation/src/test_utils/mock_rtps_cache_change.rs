use mockall::mock;
use rtps_pim::structure::{
    cache_change::RtpsCacheChangeConstructor,
    types::{ChangeKind, Guid, InstanceHandle, SequenceNumber},
};

mock! {
    pub RtpsCacheChange{}
}

impl RtpsCacheChangeConstructor for MockRtpsCacheChange {
    type DataType = Vec<u8>;
    type ParameterListType = ();

    fn new(
        _kind: ChangeKind,
        _writer_guid: Guid,
        _instance_handle: InstanceHandle,
        _sequence_number: SequenceNumber,
        _data_value: Self::DataType,
        _inline_qos: Self::ParameterListType,
    ) -> Self {
        MockRtpsCacheChange::new()
    }
}
