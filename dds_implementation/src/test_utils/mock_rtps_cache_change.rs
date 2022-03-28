use mockall::mock;
use rtps_pim::structure::{
    cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
    types::{ChangeKind, Guid, InstanceHandle, SequenceNumber},
};

mock! {
    pub RtpsCacheChange{}

    impl RtpsCacheChangeAttributes for RtpsCacheChange {
        type DataType = [u8];
        type ParameterListType = [u8];

        fn kind(&self) -> ChangeKind;
        fn writer_guid(&self) -> Guid;
        fn instance_handle(&self) -> InstanceHandle;
        fn sequence_number(&self) -> SequenceNumber;
        fn data_value(&self) -> &[u8];
        fn inline_qos(&self) -> &[u8];
    }
}

impl RtpsCacheChangeConstructor for MockRtpsCacheChange {
    type DataType = Vec<u8>;
    type ParameterListType = Vec<u8>;

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
