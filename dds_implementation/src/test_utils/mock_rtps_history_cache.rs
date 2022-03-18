use mockall::mock;
use rtps_pim::structure::{
    history_cache::{
        RtpsHistoryCacheAttributes, RtpsHistoryCacheConstructor, RtpsHistoryCacheOperations,
    },
    types::SequenceNumber,
};

use super::mock_rtps_cache_change::MockRtpsCacheChange;

mock! {
    pub RtpsHistoryCache{
        pub fn add_change_(&mut self, change: MockRtpsCacheChange);
        pub fn get_seq_num_min_(&self) -> Option<SequenceNumber>;
        pub fn get_seq_num_max_(&self) -> Option<SequenceNumber>;
    }

    impl RtpsHistoryCacheAttributes for RtpsHistoryCache {
        type CacheChangeType = MockRtpsCacheChange;

        fn changes(&self) -> &[MockRtpsCacheChange];
    }
}

impl RtpsHistoryCacheOperations for MockRtpsHistoryCache {
    type CacheChangeType = MockRtpsCacheChange;

    fn add_change(&mut self, change: MockRtpsCacheChange) {
        self.add_change_(change)
    }

    fn remove_change<F>(&mut self, _f: F)
    where
        F: FnMut(&MockRtpsCacheChange) -> bool,
    {
        todo!()
    }

    fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.get_seq_num_min_()
    }

    fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.get_seq_num_max_()
    }
}

impl RtpsHistoryCacheConstructor for MockRtpsHistoryCache {
    fn new() -> Self {
        MockRtpsHistoryCache::new()
    }
}
