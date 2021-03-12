use crate::structure::{RTPSEndpoint, RTPSHistoryCache};

use super::types::Duration;

pub trait RTPSReader<'a>: RTPSEndpoint {
    type HistoryCacheType: RTPSHistoryCache<'a>;

    fn heartbeat_response_delay(&self) -> Duration;
    fn heartbeat_supression_duration(&self) -> Duration;
    fn reader_cache(&mut self) -> &mut Self::HistoryCacheType;
    fn expects_inline_qos(&self) -> bool;
}
