use crate::structure::RtpsEndpoint;
use rust_dds_interface::history_cache::HistoryCache;

pub struct RtpsReader {
    pub endpoint: RtpsEndpoint,
    pub reader_cache: HistoryCache,
    pub expects_inline_qos: bool,
}

impl RtpsReader {
    pub fn new(endpoint: RtpsEndpoint, reader_cache: HistoryCache, expects_inline_qos: bool) -> Self {
        Self {
            endpoint,
            reader_cache,
            expects_inline_qos,
        }
    }
}