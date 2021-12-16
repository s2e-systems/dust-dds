use crate::{
    behavior::types::Duration,
    structure::{
        cache_change::RtpsCacheChange,
        types::{ChangeKind, InstanceHandle, SequenceNumber},
    },
};

pub trait RtpsWriterAttributes {
    type WriterHistoryCacheType;

    fn push_mode(&self) -> &bool;
    fn heartbeat_period(&self) -> &Duration;
    fn nack_response_delay(&self) -> &Duration;
    fn nack_suppression_duration(&self) -> &Duration;
    fn last_change_sequence_number(&self) -> &SequenceNumber;
    fn data_max_size_serialized(&self) -> &Option<i32>;
    fn writer_cache(&self) -> &Self::WriterHistoryCacheType;
}

pub trait RtpsWriterOperations {
    fn new_change<'a, P, D>(
        &mut self,
        kind: ChangeKind,
        data: D,
        inline_qos: P,
        handle: InstanceHandle,
    ) -> RtpsCacheChange<P, D>;
}
