use std::{
    ops::{Deref, DerefMut},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use super::maybe_valid::MaybeValid;

pub struct SharedMaybeValid<T>(RwLock<MaybeValid<T>>);

impl<T> Default for SharedMaybeValid<T> {
    fn default() -> Self {
        Self(RwLock::new(Default::default()))
    }
}

impl<T> SharedMaybeValid<T> {
    pub fn new(value: T) -> Self {
        Self(RwLock::new(MaybeValid::new(value)))
    }

    pub fn try_read(&self) -> Option<MaybeValidReadRef<T>> {
        if let Some(reader_guard) = self.0.try_read().ok() {
            if reader_guard.is_valid() {
                Some(MaybeValidReadRef(reader_guard))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn try_write(&self) -> Option<MaybeValidWriteRef<T>> {
        if let Some(writer_guard) = self.0.try_write().ok() {
            if !writer_guard.is_valid() {
                Some(MaybeValidWriteRef(writer_guard))
            } else {
                None
            }
        } else {
            None
        }
        
    }
}

pub struct MaybeValidReadRef<'a, T>(RwLockReadGuard<'a, MaybeValid<T>>);

impl<'a, T> std::ops::Deref for MaybeValidReadRef<'a, T> {
    type Target = MaybeValid<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct MaybeValidWriteRef<'a, T>(RwLockWriteGuard<'a, MaybeValid<T>>);

impl<'a, T> Deref for MaybeValidWriteRef<'a, T> {
    type Target = MaybeValid<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for MaybeValidWriteRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_read() {
        let shared_maybe_valid = SharedMaybeValid::new(10);
        let shared_maybe_valid_read_ref = shared_maybe_valid
            .try_read()
            .expect("Failed to get read ref");
        assert_eq!(
            shared_maybe_valid_read_ref.get().expect("Error getting valid value"),
            &10
        );
    }

    #[test]
    fn try_write() {
        let shared_maybe_valid = SharedMaybeValid::default();
        let mut shared_maybe_valid_write_ref = shared_maybe_valid
            .try_write()
            .expect("Failed to get write ref");
            shared_maybe_valid_write_ref.set(5);
        assert_eq!(
            shared_maybe_valid_write_ref.get().expect("Error getting valid value"),
            &5
        );
    }

    #[test]
    fn fail_try_write_after_try_read() {
        let shared_maybe_valid = SharedMaybeValid::new(10);
        let _shared_maybe_valid_read_ref = shared_maybe_valid
            .try_read()
            .expect("Failed to get read ref");
        assert!(shared_maybe_valid.try_write().is_none());
    }

    #[test]
    fn fail_try_write_after_dropped_try_read() {
        let shared_maybe_valid = SharedMaybeValid::new(10);
        shared_maybe_valid
            .try_read()
            .expect("Failed to get read ref");
        assert!(shared_maybe_valid.try_write().is_none());
    }
    
    #[test]
    fn succesful_try_write_after_invalidated_and_dropped_try_read() {
        let shared_maybe_valid = SharedMaybeValid::new(10);
        shared_maybe_valid
            .try_read()
            .expect("Failed to get read ref").invalidate();
        assert!(shared_maybe_valid.try_write().is_some());
    }

    #[test]
    fn fail_try_write_valid() {
        let shared_maybe_valid = SharedMaybeValid::new(10);
        assert!(shared_maybe_valid.try_write().is_none());
    }
}
