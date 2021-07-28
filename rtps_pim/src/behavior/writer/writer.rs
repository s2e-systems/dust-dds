use crate::{
    behavior::types::Duration,
    structure::{
        types::{
            ChangeKind, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID,
        },
        RTPSHistoryCache, RtpsCacheChange,
    },
};

pub trait RTPSWriter {
    type HistoryCacheType;

    fn push_mode(&self) -> bool;
    fn heartbeat_period(&self) -> &Duration;
    fn nack_response_delay(&self) -> &Duration;
    fn nack_suppression_duration(&self) -> &Duration;
    fn last_change_sequence_number(&self) -> &SequenceNumber;
    fn data_max_size_serialized(&self) -> &Option<i32>;
    fn writer_cache(&self) -> &Self::HistoryCacheType;
    fn writer_cache_mut(&mut self) -> &mut Self::HistoryCacheType;
}

pub trait RTPSWriterOperations {
    fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
    ) -> Self;

    fn new_change<'a>(
        &mut self,
        kind: ChangeKind,
        data: &'a [u8],
        inline_qos: (),
        handle: InstanceHandle,
    ) -> RtpsCacheChange<'a>
    where
        Self: RTPSWriter,
        Self::HistoryCacheType: RTPSHistoryCache;
}
