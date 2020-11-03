use crate::structure::CacheChange;
pub trait StatefulReaderListener : 'static {
    fn on_add_change(&self, cc: &CacheChange) -> (){}
}

pub struct NoOpStatefulReaderListener;
impl StatefulReaderListener for NoOpStatefulReaderListener {}