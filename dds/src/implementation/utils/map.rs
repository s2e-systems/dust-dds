use std::{collections::HashMap, hash::Hash};

use super::shared_object::DdsRwLock;

pub struct DdsMapObjectIter<'a, K, T, V> {
    iterator: std::collections::hash_map::Values<'a, K, (T, V)>,
}

impl<'a, K, T, V> Iterator for DdsMapObjectIter<'a, K, T, V> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|(t, _)| t)
    }
}

pub struct DdsMapObjectIterMut<'a, K, T, V> {
    iterator: std::collections::hash_map::ValuesMut<'a, K, (T, V)>,
}

impl<'a, K, T, V> Iterator for DdsMapObjectIterMut<'a, K, T, V> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|(t, _)| t)
    }
}

#[allow(dead_code)]
pub struct DdsMap<K, T, V> {
    list: DdsRwLock<HashMap<K, (T, V)>>,
}

#[allow(dead_code)]
impl<K, T, V> DdsMap<K, T, V> {
    fn add(&self, key: K, object: T, value: V)
    where
        K: Hash + Eq,
    {
        self.list.write_lock().insert(key, (object, value));
    }

    fn remove(&self, key: &K)
    where
        K: Hash + Eq,
    {
        self.list.write_lock().remove(key);
    }

    fn get_object<F, O>(&self, key: &K, mut f: F) -> O
    where
        F: FnMut(Option<&T>) -> O,
        K: Hash + Eq,
    {
        f(self.list.write_lock().get(key).map(|(t, _)| t))
    }

    fn get_object_mut<F, O>(&self, key: &K, mut f: F) -> O
    where
        F: FnMut(Option<&mut T>) -> O,
        K: Hash + Eq,
    {
        f(self.list.write_lock().get_mut(key).map(|(t, _)| t))
    }

    fn iter_object<F>(&self, f: F)
    where
        F: for<'a> Fn(DdsMapObjectIter<'a, K, T, V>),
    {
        f(DdsMapObjectIter {
            iterator: self.list.read_lock().values(),
        });
    }

    fn iter_object_mut<F>(&self, f: F)
    where
        F: for<'a> Fn(DdsMapObjectIterMut<'a, K, T, V>),
    {
        f(DdsMapObjectIterMut {
            iterator: self.list.write_lock().values_mut(),
        });
    }

    fn get_value(&self, key: &K) -> Option<V>
    where
        K: Hash + Eq,
        V: Clone,
    {
        self.list.read_lock().get(key).map(|(_, v)| v).cloned()
    }
}
