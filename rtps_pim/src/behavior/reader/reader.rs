use crate::structure::{RTPSEndpoint, RTPSHistoryCache};

use super::types::Duration;

pub trait RTPSReader: RTPSEndpoint {
    type HistoryCacheType: RTPSHistoryCache;

    fn expects_inline_qos(&self) -> bool;
    fn heartbeat_response_delay(&self) -> &Duration;
    fn heartbeat_supression_duration(&self) -> &Duration;
    fn reader_cache(&self) -> &Self::HistoryCacheType;
    fn reader_cache_mut(&mut self) -> &mut Self::HistoryCacheType;
}
