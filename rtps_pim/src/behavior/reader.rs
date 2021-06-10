use crate::{
    messages::types::ParameterIdPIM,
    structure::{
        types::{
            DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM, ParameterListPIM,
            SequenceNumberPIM, GUIDPIM,
        },
        RTPSEndpoint, RTPSHistoryCache,
    },
};

use super::types::DurationPIM;

pub trait RTPSReader<
    PSM: InstanceHandlePIM
        + GuidPrefixPIM
        + DataPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + LocatorPIM
        + DurationPIM
        + GUIDPIM
        + ParameterIdPIM
        + ParameterListPIM<PSM>,
>: RTPSEndpoint<PSM>
{
    type HistoryCacheType: RTPSHistoryCache<PSM>;

    fn expects_inline_qos(&self) -> bool;
    fn heartbeat_response_delay(&self) -> &PSM::DurationType;
    fn heartbeat_supression_duration(&self) -> &PSM::DurationType;
    fn reader_cache(&self) -> &Self::HistoryCacheType;
    fn reader_cache_mut(&mut self) -> &mut Self::HistoryCacheType;
}
