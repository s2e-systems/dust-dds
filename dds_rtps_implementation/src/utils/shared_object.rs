use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};

use rust_dds_api::return_type::{DDSError, DDSResult};

pub type RtpsShared<T> = Arc<RwLock<T>>;

pub fn rtps_shared_new<T>(t: T) -> RtpsShared<T> {
    Arc::new(RwLock::new(t))
}

pub fn rtps_shared_downgrade<T: ?Sized>(this: &RtpsShared<T>) -> RtpsWeak<T> {
    Arc::downgrade(this)
}

pub fn rtps_shared_read_lock<T: ?Sized>(this: &RtpsShared<T>) -> RwLockReadGuard<'_, T> {
    this.read().unwrap()
}

pub fn rtps_shared_write_lock<T: ?Sized>(this: &RtpsShared<T>) -> RwLockWriteGuard<'_, T> {
    this.write().unwrap()
}

pub type RtpsWeak<T: ?Sized> = Weak<RwLock<T>>;

pub fn rtps_weak_upgrade<T: ?Sized>(this: &RtpsWeak<T>) -> DDSResult<RtpsShared<T>> {
    this.upgrade().ok_or(DDSError::AlreadyDeleted)
}
