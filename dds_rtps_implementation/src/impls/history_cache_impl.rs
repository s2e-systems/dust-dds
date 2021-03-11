use rust_rtps::structure::RTPSHistoryCache;

use super::cache_change::CacheChange;

pub struct HistoryCache {}

impl RTPSHistoryCache for HistoryCache {
    type CacheChangeType = CacheChange;

    fn add_change(&mut self, change: Self::CacheChangeType) {
        todo!()
    }

    fn remove_change(&mut self, seq_num: rust_rtps::types::SequenceNumber) {
        todo!()
    }

    fn get_change(
        &self,
        seq_num: rust_rtps::types::SequenceNumber,
    ) -> Option<&Self::CacheChangeType> {
        todo!()
    }

    fn get_seq_num_min(&self) -> Option<rust_rtps::types::SequenceNumber> {
        todo!()
    }

    fn get_seq_num_max(&self) -> Option<rust_rtps::types::SequenceNumber> {
        todo!()
    }
}
