use crate::structure::{
    types::{ChangeKind, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID},
    RTPSCacheChange, RTPSHistoryCache,
};

use super::types::Duration;

pub trait RTPSWriter {
    type HistoryCacheType;

    fn push_mode(&self) -> bool;
    fn heartbeat_period(&self) -> &Duration;
    fn nack_response_delay(&self) -> &Duration;
    fn nack_suppression_duration(&self) -> &Duration;
    fn last_change_sequence_number(&self) -> &SequenceNumber;
    fn data_max_size_serialized(&self) -> i32;
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
        data_max_size_serialized: i32,
    ) -> Self;

    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::DataType,
        inline_qos: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::InlineQosType,
        handle: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::InstanceHandleType,
    ) -> <Self::HistoryCacheType as RTPSHistoryCache>::CacheChange
    where
        Self: RTPSWriter,
        Self::HistoryCacheType: RTPSHistoryCache,
        <Self::HistoryCacheType as RTPSHistoryCache>::CacheChange: RTPSCacheChange;
}
