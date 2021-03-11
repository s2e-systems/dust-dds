use crate::{
    messages::submessages::submessage_elements::ParameterList,
    structure::{RTPSEndpoint, RTPSCacheChange, RTPSHistoryCache},
    types::{ChangeKind, InstanceHandle, SequenceNumber},
};

use super::types::Duration;

pub trait Writer: RTPSEndpoint {
    type HistoryCacheType: RTPSHistoryCache;

    fn push_mode(&self) -> bool;
    fn heartbeat_period(&self) -> Duration;
    fn nack_response_delay(&self) -> Duration;
    fn nack_suppression_duration(&self) -> Duration;
    fn last_change_sequence_number(&self) -> SequenceNumber;
    fn data_max_sized_serialized(&self) -> i32;
    fn writer_cache(&mut self) -> &mut Self::HistoryCacheType;
    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType as RTPSCacheChange>::Data,
        inline_qos: ParameterList,
        handle: InstanceHandle,
    ) -> <Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType;
}
