use crate::{
    messages::submessages::submessage_elements::ParameterList,
    structure::{RTPSCacheChange, RTPSEndpoint, RTPSHistoryCache},
    types::{
        ChangeKind, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID,
    },
};

use super::types::Duration;

pub trait RTPSWriter: RTPSEndpoint {
    type HistoryCacheType: RTPSHistoryCache;
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
        data_max_sized_serialized: i32,
    ) -> Self;
    fn push_mode(&self) -> bool;
    fn heartbeat_period(&self) -> Duration;
    fn nack_response_delay(&self) -> Duration;
    fn nack_suppression_duration(&self) -> Duration;
    fn last_change_sequence_number(&self) -> SequenceNumber;
    fn data_max_sized_serialized(&self) -> i32;
    fn writer_cache(&self) -> &Self::HistoryCacheType;

    fn new_change(
        &self,
        kind: ChangeKind,
        data: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType as RTPSCacheChange>::Data,
        inline_qos: ParameterList,
        handle: InstanceHandle,
    ) -> <Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType;
}
