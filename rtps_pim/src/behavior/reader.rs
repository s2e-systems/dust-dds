use crate::{
    structure::{RTPSEndpoint, RTPSHistoryCache},
    PIM,
};

pub trait RTPSReader<PSM: PIM, HistoryCache: RTPSHistoryCache<PSM>>: RTPSEndpoint<PSM> {
    fn expects_inline_qos(&self) -> bool;
    fn heartbeat_response_delay(&self) -> PSM::Duration;
    fn heartbeat_supression_duration(&self) -> PSM::Duration;
    fn reader_cache(&self) -> &HistoryCache;
    fn reader_cache_mut(&mut self) -> &mut HistoryCache;
}
