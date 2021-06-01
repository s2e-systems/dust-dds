use crate::{
    messages::types::ParameterIdPIM,
    structure::{
        types::{
            ChangeKind, DataPIM, EntityIdPIM, GUIDPIM, GuidPrefixPIM, InstanceHandlePIM,
            LocatorPIM, ParameterListPIM, SequenceNumberPIM,
        },
        RTPSEndpoint, RTPSHistoryCache,
    },
};

use super::types::DurationType;

pub trait RTPSWriter<
    PSM: GuidPrefixPIM
        + EntityIdPIM
        + LocatorPIM
        + DurationType
        + SequenceNumberPIM
        + DataPIM
        + ParameterIdPIM
        + GUIDPIM<PSM>
        + ParameterListPIM<PSM>
        + InstanceHandlePIM,
>: RTPSEndpoint<PSM>
{
    type HistoryCacheType: RTPSHistoryCache<PSM>;

    fn push_mode(&self) -> bool;
    fn heartbeat_period(&self) -> &PSM::Duration;
    fn nack_response_delay(&self) -> &PSM::Duration;
    fn nack_suppression_duration(&self) -> &PSM::Duration;
    fn last_change_sequence_number(&self) -> &PSM::SequenceNumberType;
    fn data_max_size_serialized(&self) -> i32;
    fn writer_cache(&self) -> &Self::HistoryCacheType;
    fn writer_cache_mut(&mut self) -> &mut Self::HistoryCacheType;

    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: PSM::DataType,
        inline_qos: PSM::ParameterListType,
        handle: PSM::InstanceHandleType,
    ) -> <Self::HistoryCacheType as RTPSHistoryCache<PSM>>::CacheChange;
}
