use crate::{
    messages::types::ParameterIdPIM,
    structure::{
        types::{
            ChangeKind, DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM,
            ParameterListPIM, SequenceNumberPIM, GUIDPIM,
        },
        RTPSEndpoint, RTPSHistoryCache,
    },
};

use super::types::DurationPIM;

pub trait RTPSWriter<
    PSM: GuidPrefixPIM
        + EntityIdPIM
        + LocatorPIM
        + DurationPIM
        + SequenceNumberPIM
        + DataPIM
        + ParameterIdPIM
        + GUIDPIM
        + ParameterListPIM<PSM>
        + InstanceHandlePIM,
>: RTPSEndpoint<PSM>
{
    type HistoryCacheType: RTPSHistoryCache<PSM>;

    fn push_mode(&self) -> bool;
    fn heartbeat_period(&self) -> &PSM::DurationType;
    fn nack_response_delay(&self) -> &PSM::DurationType;
    fn nack_suppression_duration(&self) -> &PSM::DurationType;
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
