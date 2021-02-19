use std::{
    ops::{Deref, DerefMut},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};
pub struct SharedOption<T>(RwLock<Option<T>>);

impl<T> Default for SharedOption<T> {
    fn default() -> Self {
        Self(RwLock::new(Default::default()))
    }
}

impl<T> SharedOption<T> {
    pub fn new(value: T) -> Self {
        Self(RwLock::new(Some(value)))
    }

    pub fn try_read(&self) -> Option<SharedOptionReadRef<T>> {
        if let Some(reader_guard) = self.0.try_read().ok() {
            if reader_guard.is_some() {
                Some(SharedOptionReadRef(reader_guard))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn try_write(&self) -> Option<SharedOptionWriteRef<T>> {
        if let Some(writer_guard) = self.0.try_write().ok() {
            Some(SharedOptionWriteRef(writer_guard))
        } else {
            None
        }
    }
}

pub struct SharedOptionReadRef<'a, T>(RwLockReadGuard<'a, Option<T>>);

impl<'a, T> std::ops::Deref for SharedOptionReadRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.as_ref().unwrap() // Unwrap because the fact that the optional is Some is guaranteed by the invariant
    }
}

pub struct SharedOptionWriteRef<'a, T>(RwLockWriteGuard<'a, Option<T>>);

impl<'a, T> Deref for SharedOptionWriteRef<'a, T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for SharedOptionWriteRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_read() {
        let shared_option = SharedOption::new(10);
        let shared_option_read_ref = shared_option.try_read().expect("Failed to get read ref");
        assert_eq!(*shared_option_read_ref, 10)
    }

    #[test]
    fn try_write() {
        let shared_option = SharedOption::default();
        let mut shared_option_write_ref =
            shared_option.try_write().expect("Failed to get write ref");
        *shared_option_write_ref = Some(5);
        assert_eq!(shared_option_write_ref.unwrap(), 5);
    }

    #[test]
    fn fail_try_write_after_try_read() {
        let shared_option = SharedOption::new(10);
        let _shared_option_read_ref = shared_option.try_read().unwrap();
        assert!(shared_option.try_write().is_none());
    }

    #[test]
    fn succesful_try_write_after_dropped_try_read() {
        let shared_option = SharedOption::new(10);
        shared_option
            .try_read()
            .unwrap();
        assert!(shared_option.try_write().is_some());
    }
}
