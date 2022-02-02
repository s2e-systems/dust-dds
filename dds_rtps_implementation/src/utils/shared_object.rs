use std::{sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak, LockResult}, ops::Deref};

use rust_dds_api::return_type::{DDSError, DDSResult};

pub struct RtpsShared<T: ?Sized>(Arc<RwLock<T>>);

impl<T> RtpsShared<T> {
    pub fn new(t: T) -> Self {
        RtpsShared(Arc::new(RwLock::new(t)))
    }

    pub fn downgrade(&self) -> RtpsWeak<T> {
        RtpsWeak(Arc::downgrade(&self.0))
    }

    pub fn read_lock(&self) -> RwLockReadGuard<'_, T> {
        self.0.read().unwrap()
    }

    pub fn write_lock(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write().unwrap()
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        self.0.write()
    }
}

impl<T: ?Sized> Clone for RtpsShared<T> {
    fn clone(&self) -> Self {
        RtpsShared(self.0.clone())
    }
}

impl<T> PartialEq for RtpsShared<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: ?Sized> Deref for RtpsShared<T> {
    type Target = RwLock<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RtpsWeak<T: ?Sized>(Weak<RwLock<T>>);

impl<T> RtpsWeak<T> {
    pub fn new() -> Self {
        RtpsWeak(Weak::new())
    }

    pub fn upgrade(&self) -> DDSResult<RtpsShared<T>> {
        self.0.upgrade()
            .map(|x| RtpsShared(x))
            .ok_or(DDSError::AlreadyDeleted)
    }
}

impl<T: ?Sized> Clone for RtpsWeak<T> {
    fn clone(&self) -> Self {
        RtpsWeak(self.0.clone())
    }
}
