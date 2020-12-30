use core::sync::atomic;
use std::sync::{RwLock, RwLockReadGuard};

pub struct MaybeValid<T> {
    value: Option<T>,
    valid: atomic::AtomicBool,
}

impl<T> Default for MaybeValid<T> {
    fn default() -> Self {
        Self {
            value: None,
            valid: atomic::AtomicBool::new(false),
        }
    }
}

impl<T> MaybeValid<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Some(value),
            valid: atomic::AtomicBool::new(true),
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.is_valid() {
            Some(self.value.as_ref().unwrap())
        } else {
            None
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid.load(atomic::Ordering::Acquire)
    }

    pub fn delete(&self) {
        self.valid.store(false, atomic::Ordering::Release) // Inspired by std::sync::Arc
    }

    pub fn initialize(&mut self, value: T) {
        self.value = Some(value);
        self.valid.store(true, atomic::Ordering::Release);
    }
}

pub struct MaybeValidList<T>([RwLock<MaybeValid<T>>; 32]);
pub struct MaybeValidRef<'a, T>(RwLockReadGuard<'a, MaybeValid<T>>);

impl<'a, T> std::ops::Deref for MaybeValidRef<'a, T> {
    type Target = RwLockReadGuard<'a, MaybeValid<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Default for MaybeValidList<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> MaybeValidList<T> {
    pub fn add(&self, value: T) -> Option<MaybeValidRef<T>> {
        let index = self.initialize_free_object(value)?;
        Some(MaybeValidRef(self.0[index].read().unwrap()))
    }

    pub fn is_empty(&self) -> bool {
        self.0
            .iter()
            .find(|&x| x.read().unwrap().is_valid())
            .is_none()
    }

    pub fn contains(&self, object: &MaybeValidRef<T>) -> bool {
        self.0
            .iter()
            .find(|&x| std::ptr::eq(&*x.read().unwrap(), &*object.0))
            .is_some()
    }

    fn initialize_free_object(&self, value: T) -> Option<usize> {
        // Find an object in the list which can be borrow mutably (meaning there are no other references to it)
        // and that is marked as invalid (meaning that it has either been deleted on never initialized)
        for (index, object) in self.0.iter().enumerate() {
            if let Some(mut borrowed_object) = object.try_write().ok() {
                if !borrowed_object.is_valid() {
                    borrowed_object.initialize(value);
                    return Some(index);
                }
            }
        }
        // If it was never found then return None
        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_delete() {
        let object = MaybeValid::new(10);
        assert!(object.get().is_some());
        object.delete();
        assert!(object.get().is_none());
    }

    #[test]
    fn value_ok() {
        let object = MaybeValid::new(100i32);
        assert_eq!(object.get().unwrap(), &100i32);
    }

    #[test]
    fn value_deleted() {
        let object = MaybeValid::new(100i32);
        object.delete();
        assert!(object.get().is_none());
    }

    #[test]
    fn value_deleted_and_initialized() {
        let mut object = MaybeValid::new(100i32);
        object.delete();
        object.initialize(-10i32);
        assert_eq!(object.get().unwrap(), &-10i32);
    }

    #[test]
    fn object_list_initialize_free_object_positions() {
        let object_list: MaybeValidList<i32> = MaybeValidList::default();
        let index0 = object_list.initialize_free_object(10).unwrap();
        let index1 = object_list.initialize_free_object(20).unwrap();
        let index2 = object_list.initialize_free_object(-5).unwrap();

        assert_eq!(index0, 0);
        assert_eq!(index1, 1);
        assert_eq!(index2, 2);
    }

    #[test]
    fn object_list_initialize_free_object_positions_with_deletion() {
        let object_list: MaybeValidList<i32> = MaybeValidList::default();
        {
            let _object0 = object_list.add(0).unwrap();
            let object1 = object_list.add(10).unwrap();
            let _object2 = object_list.add(20).unwrap();
            let object3 = object_list.add(30).unwrap();

            object1.delete();
            object3.delete();
        }

        let index1 = object_list.initialize_free_object(10).unwrap();
        let index3 = object_list.initialize_free_object(30).unwrap();
        let index4 = object_list.initialize_free_object(40).unwrap();

        assert_eq!(index1, 1);
        assert_eq!(index3, 3);
        assert_eq!(index4, 4);
    }

    #[test]
    fn object_list_initialize_free_object_deleted_with_references() {
        let object_list: MaybeValidList<i32> = MaybeValidList::default();

        let _object0 = object_list.add(0).unwrap();
        let object1 = object_list.add(10).unwrap();
        let _object2 = object_list.add(20).unwrap();
        let object3 = object_list.add(30).unwrap();

        object1.delete();
        object3.delete();

        let index4 = object_list.initialize_free_object(10).unwrap();
        let index5 = object_list.initialize_free_object(30).unwrap();

        assert_eq!(index4, 4);
        assert_eq!(index5, 5);
    }

    #[test]
    fn contains() {
        let object_list1: MaybeValidList<i32> = MaybeValidList::default();
        let object_list2: MaybeValidList<i32> = MaybeValidList::default();

        let object11 = object_list1.add(10).unwrap();
        let object21 = object_list2.add(10).unwrap();

        assert_eq!(object_list1.contains(&object11), true);
        assert_eq!(object_list2.contains(&object21), true);
        assert_eq!(object_list2.contains(&object11), false);
        assert_eq!(object_list1.contains(&object21), false);
    }
}
