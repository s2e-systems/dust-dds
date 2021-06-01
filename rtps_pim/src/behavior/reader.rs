use crate::{
    messages::types::ParameterIdPIM,
    structure::{
        types::{
            DataPIM, EntityIdPIM, GUIDPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM,
            ParameterListPIM, SequenceNumberPIM,
        },
        RTPSEndpoint, RTPSHistoryCache,
    },
};

use super::types::DurationType;

pub trait RTPSReader<
    PSM: InstanceHandlePIM
        + GuidPrefixPIM
        + DataPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + LocatorPIM
        + DurationType
        + GUIDPIM<PSM>
        + ParameterIdPIM
        + ParameterListPIM<PSM>,
>: RTPSEndpoint<PSM>
{
    type HistoryCacheType: RTPSHistoryCache<PSM>;

    fn expects_inline_qos(&self) -> bool;
    fn heartbeat_response_delay(&self) -> &PSM::Duration;
    fn heartbeat_supression_duration(&self) -> &PSM::Duration;
    fn reader_cache(&self) -> &Self::HistoryCacheType;
    fn reader_cache_mut(&mut self) -> &mut Self::HistoryCacheType;
}
