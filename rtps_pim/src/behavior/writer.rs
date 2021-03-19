use crate::{
    structure::{RTPSCacheChange, RTPSEndpoint, RTPSEntity, RTPSHistoryCache},
    types::{ChangeKind, ReliabilityKind, TopicKind},
};

use super::types::Duration;

pub trait RTPSWriter: RTPSEndpoint {
    type HistoryCache: RTPSHistoryCache;
    type Duration: Duration;
    fn new(
        guid: <Self as RTPSEntity>::GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[<Self as RTPSEndpoint>::Locator],
        multicast_locator_list: &[<Self as RTPSEndpoint>::Locator],
        push_mode: bool,
        heartbeat_period: Self::Duration,
        nack_response_delay: Self::Duration,
        nack_suppression_duration: Self::Duration,
        data_max_sized_serialized: i32,
    ) -> Self;
    fn push_mode(&self) -> bool;
    fn heartbeat_period(&self) -> &Self::Duration;
    fn nack_response_delay(&self) -> &Self::Duration;
    fn nack_suppression_duration(&self) -> &Self::Duration;
    fn last_change_sequence_number(
        &self,
    ) -> <<Self::HistoryCache as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::SequenceNumber;
    fn data_max_sized_serialized(&self) -> i32;
    fn writer_cache(&self) -> &Self::HistoryCache;
    fn writer_cache_mut(&mut self) -> &mut Self::HistoryCache;

    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: <<Self::HistoryCache as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::Data,
        inline_qos: <<Self::HistoryCache as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::ParameterList,
        handle: <<Self::HistoryCache as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::InstanceHandle,
    ) -> <Self::HistoryCache as RTPSHistoryCache>::CacheChange;
}
