use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak},
};

use rust_dds_api::return_type::{DDSError, DDSResult};

pub struct RtpsShared<T>(Arc<RwLock<T>>);

impl<T> RtpsShared<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }

    pub fn downgrade(&self) -> RtpsWeak<T> {
        RtpsWeak(Arc::downgrade(&self.0))
    }

    pub fn write_lock(&self) -> RtpsWriteLock<T> {
        RtpsWriteLock(self.0.write().unwrap())
    }

    pub fn read_lock(&self) -> RtpsReadLock<T> {
        RtpsReadLock(self.0.read().unwrap())
    }
}

impl<T> Clone for RtpsShared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> PartialEq for RtpsShared<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

pub struct RtpsReadLock<'a, T>(RwLockReadGuard<'a, T>);

impl<'a, T> Deref for RtpsReadLock<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> AsRef<T> for RtpsReadLock<'a, T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

pub struct RtpsWriteLock<'a, T>(RwLockWriteGuard<'a, T>);

impl<'a, T> Deref for RtpsWriteLock<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for RtpsWriteLock<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T> AsRef<T> for RtpsWriteLock<'a, T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<'a, T> AsMut<T> for RtpsWriteLock<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

pub struct RtpsWeak<T>(Weak<RwLock<T>>);

impl<T> RtpsWeak<T> {
    pub fn upgrade(&self) -> DDSResult<RtpsShared<T>> {
        Ok(RtpsShared(
            self.0.upgrade().ok_or(DDSError::AlreadyDeleted)?,
        ))
    }
}
