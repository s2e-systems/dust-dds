use rust_dds_interface::cache_change::CacheChange;

pub trait StatelessReaderListener: 'static + Send + Sync{
    fn on_add_change(&self, _cc: &CacheChange) -> (){}
}

pub struct NoOpStatelessReaderListener;
impl StatelessReaderListener for NoOpStatelessReaderListener {}