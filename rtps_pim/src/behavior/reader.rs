use crate::{
    behavior,
    structure::{self, RTPSEndpoint, RTPSHistoryCache},
};

pub trait RTPSReader<PSM: structure::Types + behavior::Types, HistoryCache: RTPSHistoryCache<PSM>>:
    RTPSEndpoint<PSM>
{
    fn expects_inline_qos(&self) -> bool;
    fn heartbeat_response_delay(&self) -> PSM::Duration;
    fn heartbeat_supression_duration(&self) -> PSM::Duration;
    fn reader_cache(&self) -> &HistoryCache;
    fn reader_cache_mut(&mut self) -> &mut HistoryCache;
}