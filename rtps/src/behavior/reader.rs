use crate::structure::{Endpoint, HistoryCache};

use super::types::Duration;

pub trait Reader: Endpoint {
    fn heartbeat_response_delay(&self) -> Duration;
    fn heartbeat_supression_duration(&self) -> Duration;
    fn reader_cache(&mut self) -> &mut HistoryCache;
    fn expects_inline_qos(&self) -> bool;
}
