use std::sync::RwLockReadGuard;

use super::shared_object::DdsShared;

pub struct DdsIterator<'a, T> {
    list: RwLockReadGuard<'a, Vec<DdsShared<T>>>,
    index: usize,
}

impl<T> Iterator for DdsIterator<'_, T> {
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

impl<'a, T> DdsIterator<'a, T> {
    pub fn new(list: RwLockReadGuard<'a, Vec<DdsShared<T>>>) -> Self {
        Self { list, index: 0 }
    }
}
