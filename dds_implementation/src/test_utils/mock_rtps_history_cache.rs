use mockall::mock;
use rtps_pim::structure::history_cache::RtpsHistoryCacheConstructor;

mock! {
    pub RtpsHistoryCache{}
}

impl RtpsHistoryCacheConstructor for MockRtpsHistoryCache {
    fn new() -> Self {
        MockRtpsHistoryCache::new()
    }
}
