use crate::{
    messages::submessage_elements::ParameterListSubmessageElementPIM,
    structure::{RTPSEndpoint, RTPSHistoryCache},
};

use super::types::DurationPIM;

pub trait RTPSReader<PSM: DurationPIM + ParameterListSubmessageElementPIM>: RTPSEndpoint {
    type HistoryCacheType: RTPSHistoryCache;

    fn expects_inline_qos(&self) -> bool;
    fn heartbeat_response_delay(&self) -> &PSM::DurationType;
    fn heartbeat_supression_duration(&self) -> &PSM::DurationType;
    fn reader_cache(&self) -> &Self::HistoryCacheType;
    fn reader_cache_mut(&mut self) -> &mut Self::HistoryCacheType;
}
