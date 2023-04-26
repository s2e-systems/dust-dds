use std::{collections::HashMap, hash::Hash};

use super::shared_object::DdsRwLock;

#[allow(dead_code)]
pub struct DdsMap<K, T, V> {
    list: DdsRwLock<HashMap<K, (T, V)>>,
}

#[allow(dead_code)]
impl<K, T, V> DdsMap<K, T, V> {
    pub fn new() -> Self {
        Self {
            list: DdsRwLock::new(HashMap::new()),
        }
    }

    pub fn add(&self, key: K, object: T, value: V)
    where
        K: Hash + Eq,
    {
        self.list.write_lock().insert(key, (object, value));
    }

    pub fn remove(&self, key: &K)
    where
        K: Hash + Eq,
    {
        self.list.write_lock().remove(key);
    }

    pub fn get_object<F, O>(&self, key: &K, mut f: F) -> O
    where
        F: FnMut(Option<&T>) -> O,
        K: Hash + Eq,
    {
        f(self.list.write_lock().get(key).map(|(t, _)| t))
    }

    pub fn get_object_mut<F, O>(&self, key: &K, mut f: F) -> O
    where
        F: FnMut(Option<&mut T>) -> O,
        K: Hash + Eq,
    {
        f(self.list.write_lock().get_mut(key).map(|(t, _)| t))
    }

    pub fn iter_object<F, O>(&self, mut f: F) -> O
    where
        F: for<'a> FnMut(DdsMapObjectIter<'a, K, T, V>) -> O,
    {
        f(DdsMapObjectIter {
            iterator: self.list.read_lock().values(),
        })
    }

    pub fn iter_object_mut<F>(&self, mut f: F)
    where
        F: for<'a> FnMut(DdsMapObjectIterMut<'a, K, T, V>),
    {
        f(DdsMapObjectIterMut {
            iterator: self.list.write_lock().values_mut(),
        });
    }

    pub fn get_value(&self, key: &K) -> Option<V>
    where
        K: Hash + Eq,
        V: Clone,
    {
        self.list.read_lock().get(key).map(|(_, v)| v).cloned()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    struct TestObject {
        enabled: bool,
    }

    impl TestObject {
        fn new() -> Self {
            Self { enabled: false }
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }

        fn enable(&mut self) {
            self.enabled = true;
        }
    }

    #[test]
    fn add_objects() {
        let map = DdsMap::new();
        map.add(1, TestObject::new(), (1, 2));
        map.add(2, TestObject::new(), (1, 2));
        let total_elements = map.iter_object(|x| x.count());
        assert_eq!(total_elements, 2)
    }

    #[test]
    fn remove_objects() {
        let map = DdsMap::new();
        map.add(1, TestObject::new(), (1, 2));
        map.add(2, TestObject::new(), (1, 2));
        map.remove(&2);
        let total_elements = map.iter_object(|x| x.count());
        assert_eq!(total_elements, 1)
    }

    #[test]
    fn get_object() {
        let map = DdsMap::new();
        map.add(1, TestObject::new(), (1, 2));

        map.get_object_mut(&1, |x| x.unwrap().enable());
        assert!(map.get_object(&1, |x| x.unwrap().is_enabled()));
    }

    #[test]
    fn iter_mut_objects() {
        let map = DdsMap::new();
        map.add(1, TestObject::new(), (1, 2));
        map.add(2, TestObject::new(), (3, 4));

        map.iter_object_mut(|list| {
            for o in list {
                o.enable()
            }
        });

        assert!(map.get_object(&1, |x| x.unwrap().is_enabled()));
        assert!(map.get_object(&2, |x| x.unwrap().is_enabled()));
    }

    #[test]
    fn get_values() {
        let map = DdsMap::new();
        map.add(1, TestObject::new(), (1, 2));
        map.add(2, TestObject::new(), (3, 4));

        assert_eq!(map.get_value(&2), Some((3, 4)));
    }
}
