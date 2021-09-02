use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, MutexGuard, Weak},
};

use rust_dds_api::return_type::{DDSError, DDSResult};

pub struct RtpsShared<T>(Arc<T>);

impl<T> RtpsShared<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(value))
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

impl<T> PartialEq for RtpsShared<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Deref for RtpsShared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RtpsLock<'a, T>(MutexGuard<'a, T>);

impl<'a, T> Deref for RtpsLock<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for RtpsLock<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T> AsRef<T> for RtpsLock<'a, T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<'a, T> AsMut<T> for RtpsLock<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

pub struct RtpsWeak<T>(Weak<T>);

impl<T> RtpsWeak<T> {
    pub fn upgrade(&self) -> DDSResult<RtpsShared<T>> {
        Ok(RtpsShared(
            self.0.upgrade().ok_or(DDSError::AlreadyDeleted)?,
        ))
    }
}
