use std::{collections::HashMap, hash::Hash};

use super::shared_object::DdsRwLock;

#[allow(dead_code)]
pub struct DdsMap<K, V> {
    list: DdsRwLock<HashMap<K, V>>,
}

#[allow(dead_code)]
impl<K, V> DdsMap<K, V> {
    pub fn new() -> Self {
        Self {
            list: DdsRwLock::new(HashMap::new()),
        }
    }

    pub fn add(&self, key: K, value: V)
    where
        K: Hash + Eq,
    {
        self.list.write_lock().insert(key, value);
    }

    pub fn remove(&self, key: &K)
    where
        K: Hash + Eq,
    {
        self.list.write_lock().remove(key);
    }

    pub fn get<F, O>(&self, key: &K, f: F) -> O
    where
        F: FnOnce(Option<&V>) -> O,
        K: Hash + Eq,
    {
        f(self.list.read_lock().get(key))
    }

    pub fn get_mut<F, O>(&self, key: &K, f: F) -> O
    where
        F: FnOnce(Option<&mut V>) -> O,
        K: Hash + Eq,
    {
        f(self.list.write_lock().get_mut(key))
    }

    pub fn iter<F, O>(&self, f: F) -> O
    where
        F: for<'a> FnOnce(&DdsMapIter<'a, K, V>) -> O,
    {
        f(&DdsMapIter {
            iterator: self.list.read_lock().iter(),
        })
    }

    pub fn iter_mut<F, O>(&self, f: F) -> O
    where
        F: for<'a> FnOnce(&DdsMapIterMut<'a, K, V>) -> O,
    {
        f(&DdsMapIterMut {
            iterator: self.list.write_lock().iter_mut(),
        })
    }

    pub fn values<F, O>(&self, f: F) -> O
    where
        F: for<'a> FnOnce(&mut DdsMapValueIter<'a, K, V>) -> O,
    {
        f(&mut DdsMapValueIter {
            iterator: self.list.read_lock().values(),
        })
    }

    pub fn values_mut<F, O>(&self, f: F) -> O
    where
        F: for<'a> FnOnce(&mut DdsMapValueIterMut<'a, K, V>) -> O,
    {
        f(&mut DdsMapValueIterMut {
            iterator: self.list.write_lock().values_mut(),
        })
    }
}

pub struct DdsMapIter<'a, K, V> {
    iterator: std::collections::hash_map::Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for DdsMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

pub struct DdsMapIterMut<'a, K, V> {
    iterator: std::collections::hash_map::IterMut<'a, K, V>,
}

impl<'a, K, V> Iterator for DdsMapIterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

pub struct DdsMapValueIter<'a, K, V> {
    iterator: std::collections::hash_map::Values<'a, K, V>,
}

impl<'a, K, V> Iterator for DdsMapValueIter<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

pub struct DdsMapValueIterMut<'a, K, V> {
    iterator: std::collections::hash_map::ValuesMut<'a, K, V>,
}

impl<'a, K, V> Iterator for DdsMapValueIterMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
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
        map.add(1, TestObject::new());
        map.add(2, TestObject::new());
        let total_elements = map.values(|x| x.count());
        assert_eq!(total_elements, 2)
    }

    #[test]
    fn remove_objects() {
        let map = DdsMap::new();
        map.add(1, TestObject::new());
        map.add(2, TestObject::new());
        map.remove(&2);
        let total_elements = map.values(|x| x.count());
        assert_eq!(total_elements, 1)
    }

    #[test]
    fn get_object() {
        let map = DdsMap::new();
        map.add(1, TestObject::new());

        map.get_mut(&1, |x| x.unwrap().enable());
        assert!(map.get(&1, |x| x.unwrap().is_enabled()));
    }

    #[test]
    fn iter_mut_objects() {
        let map = DdsMap::new();
        map.add(1, TestObject::new());
        map.add(2, TestObject::new());

        map.values_mut(|list| {
            for o in list {
                o.enable()
            }
        });

        assert!(map.get(&1, |x| x.unwrap().is_enabled()));
        assert!(map.get(&2, |x| x.unwrap().is_enabled()));
    }
}
