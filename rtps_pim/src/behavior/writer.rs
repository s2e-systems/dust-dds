use crate::{
    messages::types::ParameterIdType,
    structure::{
        types::{
            ChangeKind, DataType, EntityIdPIM, GUIDType, GuidPrefixPIM, InstanceHandleType,
            LocatorType, ParameterListType, SequenceNumberType,
        },
        RTPSEndpoint, RTPSHistoryCache,
    },
};

use super::types::DurationType;

pub trait RTPSWriter<
    PSM: GuidPrefixPIM
        + EntityIdPIM
        + LocatorType
        + DurationType
        + SequenceNumberType
        + DataType
        + ParameterIdType
        + GUIDType<PSM>
        + ParameterListType<PSM>
        + InstanceHandleType,
>: RTPSEndpoint<PSM>
{
    type HistoryCacheType: RTPSHistoryCache<PSM>;

    fn push_mode(&self) -> bool;
    fn heartbeat_period(&self) -> &PSM::Duration;
    fn nack_response_delay(&self) -> &PSM::Duration;
    fn nack_suppression_duration(&self) -> &PSM::Duration;
    fn last_change_sequence_number(&self) -> &PSM::SequenceNumber;
    fn data_max_size_serialized(&self) -> i32;
    fn writer_cache(&self) -> &Self::HistoryCacheType;
    fn writer_cache_mut(&mut self) -> &mut Self::HistoryCacheType;

    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: PSM::Data,
        inline_qos: PSM::ParameterList,
        handle: PSM::InstanceHandle,
    ) -> <Self::HistoryCacheType as RTPSHistoryCache<PSM>>::CacheChange;
}
