use crate::structure::{RTPSEndpoint, RTPSHistoryCache};

pub trait RTPSReader: RTPSEndpoint {
    type HistoryCacheType: RTPSHistoryCache;
    type DurationType;

    fn expects_inline_qos(&self) -> bool;
    fn heartbeat_response_delay(&self) -> &Self::DurationType;
    fn heartbeat_supression_duration(&self) -> &Self::DurationType;
    fn reader_cache(&self) -> &Self::HistoryCacheType;
    fn reader_cache_mut(&mut self) -> &mut Self::HistoryCacheType;
}
