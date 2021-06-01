use crate::{
    messages::types::ParameterIdType,
    structure::{
        types::{
            DataType, EntityIdPIM, GUIDType, GuidPrefixPIM, InstanceHandleType, LocatorType,
            ParameterListType, SequenceNumberType,
        },
        RTPSEndpoint, RTPSHistoryCache,
    },
};

use super::types::DurationType;

pub trait RTPSReader<
    PSM: InstanceHandleType
        + GuidPrefixPIM
        + DataType
        + EntityIdPIM
        + SequenceNumberType
        + LocatorType
        + DurationType
        + GUIDType<PSM>
        + ParameterIdType
        + ParameterListType<PSM>,
>: RTPSEndpoint<PSM>
{
    type HistoryCacheType: RTPSHistoryCache<PSM>;

    fn expects_inline_qos(&self) -> bool;
    fn heartbeat_response_delay(&self) -> &PSM::Duration;
    fn heartbeat_supression_duration(&self) -> &PSM::Duration;
    fn reader_cache(&self) -> &Self::HistoryCacheType;
    fn reader_cache_mut(&mut self) -> &mut Self::HistoryCacheType;
}
