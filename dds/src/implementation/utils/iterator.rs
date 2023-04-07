use super::shared_object::{DdsRwLock, DdsShared};

pub struct DdsIterator<'a, T> {
    list: &'a DdsRwLock<Vec<DdsShared<T>>>,
    index: usize,
}

impl<T> Iterator for DdsIterator<'_, T> {
    type Item = DdsShared<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.list.read_lock().get(self.index) {
            Some(out) => {
                self.index += 1;
                Some(out.clone())
            }
            None => None,
        }
    }
}

impl<'a, T> DdsIterator<'a, T> {
    pub fn new(list: &'a DdsRwLock<Vec<DdsShared<T>>>) -> Self {
        Self { list, index: 0 }
    }
}
