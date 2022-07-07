use std::{
    ops::Deref,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak},
};

use dds_api::return_type::{DdsError, DdsResult};

pub struct DdsShared<T: ?Sized>(Arc<T>);

impl<T> DdsShared<T> {
    pub fn new(t: T) -> Self {
        DdsShared(Arc::new(t))
    }

    pub fn downgrade(&self) -> DdsWeak<T> {
        DdsWeak(Arc::downgrade(&self.0))
    }

    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        Arc::ptr_eq(&this.0, &other.0)
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }
}

impl<T: ?Sized> Deref for DdsShared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub struct DdsRwLock<T>(RwLock<T>);

impl<T> DdsRwLock<T> {
    pub fn new(t: T) -> Self {
        Self(RwLock::new(t))
    }

    pub fn read_lock(&self) -> RwLockReadGuard<'_, T> {
        self.0.read().expect("The lock is poisoned (;_;)")
    }

    pub fn write_lock(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write().expect("The lock is poisoned (;_;)")
    }
}

impl<T: ?Sized> Clone for DdsShared<T> {
    fn clone(&self) -> Self {
        DdsShared(self.0.clone())
    }
}

impl<T> PartialEq for DdsShared<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

pub struct DdsWeak<T: ?Sized>(Weak<T>);

impl<T> DdsWeak<T> {
    pub fn new() -> Self {
        DdsWeak(Weak::new())
    }

    pub fn upgrade(&self) -> DdsResult<DdsShared<T>> {
        self.0
            .upgrade()
            .map(|x| DdsShared(x))
            .ok_or(DdsError::AlreadyDeleted)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}

impl<T: ?Sized> Clone for DdsWeak<T> {
    fn clone(&self) -> Self {
        DdsWeak(self.0.clone())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn ptr_eq() {
        let x = DdsShared::new(vec![1, 2, 3, 4]);
        let y = x.downgrade();
        let z = x.clone();
        assert!(DdsShared::ptr_eq(&x, &z));
        assert!(DdsShared::ptr_eq(&x, &y.upgrade().unwrap()));
    }
}
