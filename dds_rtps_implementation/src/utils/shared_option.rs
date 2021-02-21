use std::{
    ops::{Deref, DerefMut},
    sync::{atomic, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

struct SharedOptionValue<T> {
    value: Option<T>,
    read_ref_count: atomic::AtomicUsize,
}

impl<T> Default for SharedOptionValue<T> {
    fn default() -> Self {
        Self {
            value: None,
            read_ref_count: atomic::AtomicUsize::new(0),
        }
    }
}

impl<T> SharedOptionValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Some(value),
            read_ref_count: atomic::AtomicUsize::new(0),
        }
    }
}

pub struct SharedOption<T>(RwLock<SharedOptionValue<T>>);
impl<T> Default for SharedOption<T> {
    fn default() -> Self {
        Self(RwLock::new(SharedOptionValue::default()))
    }
}

impl<T> SharedOption<T> {
    pub fn new(value: T) -> Self {
        Self(RwLock::new(SharedOptionValue::new(value)))
    }

    pub fn try_read(&self) -> Option<SharedOptionReadRef<T>> {
        if let Some(reader_guard) = self.0.try_read().ok() {
            if reader_guard.value.is_some() {
                reader_guard
                    .read_ref_count
                    .fetch_add(1, atomic::Ordering::Acquire);
                Some(SharedOptionReadRef(reader_guard))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn write(&self) -> SharedOptionWriteRef<T> {
        SharedOptionWriteRef(self.0.write().unwrap())
    }

    pub fn try_write(&self) -> Option<SharedOptionWriteRef<T>> {
        if let Some(writer_guard) = self.0.try_write().ok() {
            Some(SharedOptionWriteRef(writer_guard))
        } else {
            None
        }
    }
}

pub struct SharedOptionReadRef<'a, T>(RwLockReadGuard<'a, SharedOptionValue<T>>);

impl<'a, T> SharedOptionReadRef<'a, T> {
    pub fn read_ref_count(this: &SharedOptionReadRef<'a, T>) -> usize {
        this.0.read_ref_count.load(atomic::Ordering::Acquire)
    }
}

impl<'a, T> std::ops::Deref for SharedOptionReadRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.value.as_ref().unwrap() // Unwrap because the fact that the optional is Some is guaranteed by the invariant
    }
}

impl<'a, T> Drop for SharedOptionReadRef<'a, T> {
    fn drop(&mut self) {
        self.0
            .read_ref_count
            .fetch_sub(1, atomic::Ordering::Acquire);
    }
}

pub struct SharedOptionWriteRef<'a, T>(RwLockWriteGuard<'a, SharedOptionValue<T>>);

impl<'a, T> Deref for SharedOptionWriteRef<'a, T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0.value
    }
}

impl<'a, T> DerefMut for SharedOptionWriteRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.value
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
        shared_option.try_read().unwrap();
        assert!(shared_option.try_write().is_some());
    }

    #[test]
    fn read_ref_count_value() {
        let shared_option = SharedOption::new(10);
        let read_ref1 = shared_option.try_read().unwrap();

        assert_eq!(SharedOptionReadRef::read_ref_count(&read_ref1), 1);
        {
            let read_ref2 = shared_option.try_read().unwrap();

            assert_eq!(SharedOptionReadRef::read_ref_count(&read_ref1), 2);
            assert_eq!(SharedOptionReadRef::read_ref_count(&read_ref2), 2);
        }
        assert_eq!(SharedOptionReadRef::read_ref_count(&read_ref1), 1);
    }
}
