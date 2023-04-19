use std::{collections::HashMap, sync::{RwLockReadGuard, RwLockWriteGuard}};

pub struct DdsListIntoIterator<'a, T> {
    lock: RwLockReadGuard<'a, Vec<T>>,
}

impl<'a, T> IntoIterator for &'a DdsListIntoIterator<'_, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.lock.iter()
    }
}

impl<'a, T> DdsListIntoIterator<'a, T> {
    pub fn new(lock: RwLockReadGuard<'a, Vec<T>>) -> Self {
        Self { lock }
    }
}

pub struct DdsDrainIntoIterator<'a, T> {
    lock: RwLockWriteGuard<'a, Vec<T>>,
}

impl<'a, T> DdsDrainIntoIterator<'a, T> {
    pub fn new(lock: RwLockWriteGuard<'a, Vec<T>>) -> Self {
        Self { lock }
    }
}

impl<'a, T> IntoIterator for &'a mut DdsDrainIntoIterator<'_, T> {
    type Item = T;
    type IntoIter = std::vec::Drain<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.lock.drain(..)
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
