use core::sync::atomic;
use std::cell::RefCell;

use crate::types::{ReturnCode, ReturnCodes};

pub enum RtpsObjectStorage<T> {
    Some(T),
    None,
}

impl<T> Default for RtpsObjectStorage<T> {
    fn default() -> Self {
        RtpsObjectStorage::None
    }
}

impl<T> std::ops::Deref for RtpsObjectStorage<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            RtpsObjectStorage::Some(value) => value,
            RtpsObjectStorage::None => panic!("Access prevented by the container!"),
        }
    }
}

pub struct RtpsObject<T> {
    value: RefCell<RtpsObjectStorage<T>>,
    valid: atomic::AtomicBool,
    reference_count: atomic::AtomicUsize,
}

impl<T> Default for RtpsObject<T> {
    fn default() -> Self {
        Self {
            value: RefCell::new(RtpsObjectStorage::default()),
            valid: atomic::AtomicBool::new(false),
            reference_count: atomic::AtomicUsize::new(0),
        }
    }
}

impl<T> RtpsObject<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: RefCell::new(RtpsObjectStorage::Some(value)),
            valid: atomic::AtomicBool::new(true),
            reference_count: atomic::AtomicUsize::new(0),
        }
    }

    pub fn get_reference(&self) -> ReturnCode<RtpsObjectReference<T>> {
        if self.is_valid() {
            self.reference_count.fetch_add(1, atomic::Ordering::Acquire); // Inspired by std::sync::Arc
            Ok(RtpsObjectReference(self))
        } else {
            Err(ReturnCodes::AlreadyDeleted)
        }
    }

    pub fn initialize(&self, value:T) -> ReturnCode<()> {
        if self.is_empty(){
            // Initialize only gets here if there are no read references so it would be a panic to not be able to get the lock
            *self.value.try_write().unwrap() = RtpsObjectStorage::Some(value);
            self.valid.store(true, atomic::Ordering::Release); // Inspired by std::sync::Arc
            Ok(())
        } else {
            Err(ReturnCodes::PreconditionNotMet("Object must be empty"))
        }
    }

    fn is_valid(&self) -> bool {
        self.valid.load(atomic::Ordering::SeqCst)
    }

    fn reference_count(&self) -> usize {
        self.reference_count.load(atomic::Ordering::SeqCst)
    }

    pub fn is_empty(&self) -> bool {
        self.is_valid() == false && self.reference_count() == 0
    }

    pub fn delete(&self) {
        self.valid.store(false, atomic::Ordering::Release) // Inspired by std::sync::Arc
    }
}

pub struct RtpsObjectReference<'a, T>(&'a RtpsObject<T>);

impl<'a, T> RtpsObjectReference<'a, T> {
    pub fn value(&self) -> ReturnCode<RwLockReadGuard<RtpsObjectStorage<T>>> {
        if self.0.is_valid() {
            // Only read locks are present when the value is initialized so it would be a panic to not be able to get the lock
            Ok(self.0.value.try_read().unwrap())
        } else {
            Err(ReturnCodes::AlreadyDeleted)
        }
    }
}

impl<'a, T> std::ops::Drop for RtpsObjectReference<'a, T> {
    fn drop(&mut self) {
        self.0
            .reference_count
            .fetch_sub(1, atomic::Ordering::Acquire); // Inspired by std::sync::Arc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_count() {
        let object = RtpsObject::new(0);
        assert_eq!(object.reference_count(), 0);
        {
            let _reference1 = object.get_reference();
            assert_eq!(object.reference_count(), 1);
            {
                let _reference2 = object.get_reference();
                assert_eq!(object.reference_count(), 2);
            }
            assert_eq!(object.reference_count(), 1);
        }
        assert_eq!(object.reference_count.load(atomic::Ordering::Relaxed), 0);
    }

    #[test]
    fn deleted_object() {
        let object = RtpsObject::new(100i32);
        let reference = object.get_reference().expect("Valid reference expected");
        assert_eq!(**reference.value().unwrap(), 100i32);
        assert!(reference.value().unwrap().is_positive());

        object.delete();

        match reference.value() {
            Err(ReturnCodes::AlreadyDeleted) => assert!(true),
            _ => assert!(false, "Value should return Already Deleted"),
        };

        match object.get_reference() {
            Err(ReturnCodes::AlreadyDeleted) => assert!(true),
            _ => assert!(false, "Object should return Already Deleted"),
        };
    }

    #[test]
    fn empty_object() {
        let object = RtpsObject::default();
        assert_eq!(object.is_empty(), true);

        object.initialize(1.250).unwrap();
        assert_eq!(object.is_empty(), false);
        {
            let _reference1 = object.get_reference();
            assert_eq!(object.is_empty(), false);

            object.delete();
            assert_eq!(object.is_empty(), false);
        }
        assert_eq!(object.is_empty(), true);
    }

    #[test]
    fn object_with_reference() {
        #[derive(Default)]
        struct TestObject<'a> {
            value_ref: Option<&'a u32>,
        }

        impl<'a> TestObject<'a> {
            fn my_value(&self) {
                println!("{:?}", self.value_ref);
            }
        }

        let value = 5;

        let object = RtpsObject::new(TestObject{value_ref: Some(&value)});
        let object_ref = object.get_reference().unwrap();
        object_ref.value().unwrap().my_value();
        println!("{:?}", object_ref.value().unwrap().value_ref.unwrap())
    }
        
}
