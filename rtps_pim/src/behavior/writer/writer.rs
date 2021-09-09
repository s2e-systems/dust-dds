use crate::{
    behavior::types::Duration,
    messages::submessage_elements::Parameter,
    structure::{
        types::{
            ChangeKind, Guid, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind,
        },
        RtpsCacheChange, RtpsHistoryCache,
    },
};

pub trait RtpsWriter {
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

pub trait RtpsWriterOperations {
    fn new(
        guid: Guid,
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
        data: <Self::HistoryCacheType as RtpsHistoryCache<'a>>::CacheChangeDataType,
        inline_qos: &'a [Parameter<'a>],
        handle: InstanceHandle,
    ) -> RtpsCacheChange<'a, <Self::HistoryCacheType as RtpsHistoryCache<'a>>::CacheChangeDataType>
    where
        Self: RtpsWriter,
        Self::HistoryCacheType: RtpsHistoryCache<'a>;
}
