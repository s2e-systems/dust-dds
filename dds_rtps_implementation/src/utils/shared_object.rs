use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard, Weak},
};

use rust_dds_api::return_type::{DDSError, DDSResult};

pub struct RtpsShared<T>(Arc<Mutex<T>>);

impl<T> RtpsShared<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }

    pub fn downgrade(&self) -> RtpsWeak<T> {
        RtpsWeak(Arc::downgrade(&self.0))
    }

    pub fn lock(&self) -> RtpsLock<'_, T> {
        RtpsLock(self.0.lock().unwrap())
    }

    pub fn try_lock(&self) -> Option<RtpsLock<'_, T>> {
        Some(RtpsLock(self.0.try_lock().ok()?))
    }
}

impl<T> Clone for RtpsShared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
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

impl<'a,T> AsRef<T> for RtpsLock<'a, T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<'a,T> AsMut<T> for RtpsLock<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

pub struct RtpsWeak<T>(Weak<Mutex<T>>);

impl<T> RtpsWeak<T> {
    pub fn upgrade(&self) -> DDSResult<RtpsShared<T>> {
        Ok(RtpsShared(
            self.0.upgrade().ok_or(DDSError::AlreadyDeleted)?,
        ))
    }
}
