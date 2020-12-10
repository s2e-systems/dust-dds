use core::sync::atomic;
use std::cell::UnsafeCell;
use crate::types::{ReturnCode, ReturnCodes};

pub struct RtpsObject<T: Default> {
    value: UnsafeCell<T>,
    valid: atomic::AtomicBool,
    reference_count: atomic::AtomicUsize,
}

impl<T: Default> Default for RtpsObject<T> {
    fn default() -> Self {
        Self {
            value: UnsafeCell::new(T::default()),
            valid: atomic::AtomicBool::new(false),
            reference_count: atomic::AtomicUsize::new(0),
        }
    }
}

impl<T: Default> RtpsObject<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
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
            unsafe { *self.value.get() = value};
            self.valid.store(true, atomic::Ordering::Release); // Inspired by std::sync::Arc
            Ok(())
        } else {
            Err(ReturnCodes::PreconditionNotMet("Object must be empty"))
        }
    }

    pub fn as_ref(&self) -> ReturnCode<&T> {
        if self.is_valid() {
            Ok(unsafe{&*self.value.get()})
        } else {
            Err(ReturnCodes::AlreadyDeleted)
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

pub struct RtpsObjectReference<'a, T: Default>(&'a RtpsObject<T>);

impl<'a, T: Default> RtpsObjectReference<'a, T> {
    pub fn value(&'a self) -> ReturnCode<&T> {
        self.0.as_ref()
    }
}

impl<'a, T: Default> std::ops::Drop for RtpsObjectReference<'a, T> {
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
        assert_eq!(*reference.value().unwrap(), 100i32);
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
