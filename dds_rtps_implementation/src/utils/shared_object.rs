use std::sync::{Arc, Mutex, Weak};

pub struct RtpsShared<T>(Arc<Mutex<T>>);

impl<T> RtpsShared<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }

    pub fn downgrade(&self) -> RtpsWeak<T> {
        RtpsWeak(Arc::downgrade(&self.0))
    }
}

impl<T> Clone for RtpsShared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub struct RtpsWeak<T>(Weak<Mutex<T>>);
