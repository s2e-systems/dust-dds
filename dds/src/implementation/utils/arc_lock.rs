use std::sync::Arc;

use super::shared_object::DdsRwLock;

#[allow(dead_code)]
#[derive(Clone)]
pub struct DdsArcLock<T> {
    lock: Arc<DdsRwLock<T>>,
}

#[allow(dead_code)]
impl<T> DdsArcLock<T> {
    pub fn new(t: T) -> Self {
        Self {
            lock: Arc::new(DdsRwLock::new(t)),
        }
    }

    pub fn get<F, O>(&self, mut f: F) -> O
    where
        F: FnMut(&T) -> O,
    {
        f(&*self.lock.read_lock())
    }

    pub fn get_mut<F, O>(&self, mut f: F) -> O
    where
        F: FnMut(&mut T) -> O,
    {
        f(&mut *self.lock.write_lock())
    }
}
