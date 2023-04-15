use std::{collections::HashMap, sync::RwLockReadGuard};

pub struct DdsListIterator<'a, T> {
    lock: RwLockReadGuard<'a, Vec<T>>,
}

impl<'a, T> IntoIterator for &'a DdsListIterator<'_, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.lock.iter()
    }
}

impl<'a, T> DdsListIterator<'a, T> {
    pub fn new(lock: RwLockReadGuard<'a, Vec<T>>) -> Self {
        Self { lock }
    }
}

pub struct DdsMapIntoIterator<'a, K, T> {
    lock: RwLockReadGuard<'a, HashMap<K, T>>,
}

impl<'a, K, T> DdsMapIntoIterator<'a, K, T> {
    pub fn new(lock: RwLockReadGuard<'a, HashMap<K, T>>) -> Self {
        Self { lock }
    }
}

impl<'a, K, T> IntoIterator for &'a DdsMapIntoIterator<'_, K, T> {
    type Item = (&'a K, &'a T);
    type IntoIter = std::collections::hash_map::Iter<'a, K, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.lock.iter()
    }
}
