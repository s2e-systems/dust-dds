use std::{collections::HashMap, sync::RwLockReadGuard};

use super::shared_object::DdsShared;

pub struct DdsListIterator<'a, T> {
    list: RwLockReadGuard<'a, Vec<DdsShared<T>>>,
    index: usize,
}

impl<T> Iterator for DdsListIterator<'_, T> {
    type Item = DdsShared<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.list.get(self.index) {
            Some(out) => {
                self.index += 1;
                Some(out.clone())
            }
            None => None,
        }
    }
}

impl<'a, T> DdsListIterator<'a, T> {
    pub fn new(list: RwLockReadGuard<'a, Vec<DdsShared<T>>>) -> Self {
        Self { list, index: 0 }
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
