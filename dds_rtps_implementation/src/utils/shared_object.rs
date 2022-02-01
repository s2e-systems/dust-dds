use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};

use rust_dds_api::return_type::{DDSError, DDSResult};

pub struct RtpsShared<T: ?Sized>(pub Arc<RwLock<T>>);

impl<T: ?Sized> Clone for RtpsShared<T> {
    fn clone(&self) -> Self {
        RtpsShared(self.0.clone())
    }
}

pub fn rtps_shared_new<T>(t: T) -> RtpsShared<T> {
    RtpsShared(Arc::new(RwLock::new(t)))
}

pub fn rtps_shared_downgrade<T: ?Sized>(this: &RtpsShared<T>) -> RtpsWeak<T> {
    RtpsWeak(Arc::downgrade(&this.0))
}

pub fn rtps_shared_read_lock<T: ?Sized>(this: &RtpsShared<T>) -> RwLockReadGuard<'_, T> {
    this.0.read().unwrap()
}

pub fn rtps_shared_write_lock<T: ?Sized>(this: &RtpsShared<T>) -> RwLockWriteGuard<'_, T> {
    this.0.write().unwrap()
}

pub struct RtpsWeak<T: ?Sized>(pub Weak<RwLock<T>>);

impl<T> RtpsWeak<T> {
    pub fn new() -> Self {
        RtpsWeak(Weak::new())
    }
}

impl<T: ?Sized> Clone for RtpsWeak<T> {
    fn clone(&self) -> Self {
        RtpsWeak(self.0.clone())
    }
}

pub fn rtps_weak_upgrade<T: ?Sized>(this: &RtpsWeak<T>) -> DDSResult<RtpsShared<T>> {
    this.0.upgrade()
        .map(|x| RtpsShared(x))
        .ok_or(DDSError::AlreadyDeleted)
}
