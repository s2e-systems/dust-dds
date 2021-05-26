use crate::{
    messages::submessage_elements::ParameterListType,
    structure::{
        types::{
            DataType, EntityIdType, GuidPrefixType, InstanceHandleType, LocatorType,
            SequenceNumberType,
        },
        RTPSEndpoint, RTPSHistoryCache,
    },
};

use super::types::DurationType;

pub trait RTPSReader<
    PSM: InstanceHandleType
        + GuidPrefixType
        + DataType
        + ParameterListType
        + EntityIdType
        + SequenceNumberType
        + LocatorType
        + DurationType,
    HistoryCache: RTPSHistoryCache<PSM>,
>: RTPSEndpoint<PSM>
{
    fn expects_inline_qos(&self) -> bool;
    fn heartbeat_response_delay(&self) -> PSM::Duration;
    fn heartbeat_supression_duration(&self) -> PSM::Duration;
    fn reader_cache(&self) -> &HistoryCache;
    fn reader_cache_mut(&mut self) -> &mut HistoryCache;
}
