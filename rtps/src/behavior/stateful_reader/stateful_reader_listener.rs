use crate::structure::CacheChange;
pub trait StatefulReaderListener : 'static {
    fn on_add_change(&self, _cc: &CacheChange) -> (){}
}

pub struct NoOpStatefulReaderListener;
impl StatefulReaderListener for NoOpStatefulReaderListener {}