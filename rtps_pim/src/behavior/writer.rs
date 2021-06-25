use crate::{
    messages::submessage_elements::ParameterListSubmessageElementPIM,
    structure::{
        types::{ChangeKind, SequenceNumber},
        RTPSCacheChange, RTPSHistoryCache,
    },
};

use super::types::DurationPIM;

pub trait RTPSWriter<PSM> {
    type HistoryCacheType: RTPSHistoryCache;

    fn push_mode(&self) -> bool;
    fn heartbeat_period(&self) -> &PSM::DurationType
    where
        PSM: DurationPIM;
    fn nack_response_delay(&self) -> &PSM::DurationType
    where
        PSM: DurationPIM;
    fn nack_suppression_duration(&self) -> &PSM::DurationType
    where
        PSM: DurationPIM;
    fn last_change_sequence_number(&self) -> &SequenceNumber;
    fn data_max_size_serialized(&self) -> i32;
    fn writer_cache(&self) -> &Self::HistoryCacheType;
    fn writer_cache_mut(&mut self) -> &mut Self::HistoryCacheType;

    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChange as RTPSCacheChange<PSM>>::DataType,
        inline_qos: PSM::ParameterListSubmessageElementType,
        handle: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChange as RTPSCacheChange<PSM>>::InstanceHandleType,
    ) -> <Self::HistoryCacheType as RTPSHistoryCache>::CacheChange
    where
        <Self::HistoryCacheType as RTPSHistoryCache>::CacheChange: RTPSCacheChange<PSM>,
        PSM: ParameterListSubmessageElementPIM;
}
