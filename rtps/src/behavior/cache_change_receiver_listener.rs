use rust_dds_interface::cache_change::CacheChange;

pub trait CacheChangeReceiverListener{
    fn on_add_change(&self, _cc: &CacheChange) -> (){}
}

pub struct NoOpCacheChangeReceiverListener;
impl CacheChangeReceiverListener for NoOpCacheChangeReceiverListener {}