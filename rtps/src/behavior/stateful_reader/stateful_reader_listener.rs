use rust_dds_interface::cache_change::CacheChange;
pub trait StatefulReaderListener : 'static + Send + Sync{
    fn on_add_change(&self, _cc: &CacheChange) -> (){}
}

pub struct NoOpStatefulReaderListener;
impl StatefulReaderListener for NoOpStatefulReaderListener {}