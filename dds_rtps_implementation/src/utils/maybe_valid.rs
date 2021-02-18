use core::sync::atomic;
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

    pub fn is_valid(&self) -> bool {
        self.valid.load(atomic::Ordering::Acquire)
    }

    pub fn get(&self) -> Option<&T> {
        if self.is_valid() {
            Some(self.value.as_ref().unwrap())
        } else {
            None
        }
    }

    pub fn invalidate(&self) {
        self.valid.store(false, atomic::Ordering::Release) // Inspired by std::sync::Arc
    }

    pub fn set(&mut self, value: T) {
        self.value = Some(value);
        self.valid.store(true, atomic::Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_delete() {
        let object = MaybeValid::new(10);
        assert!(MaybeValid::get(&object).is_some());
        object.invalidate();
        assert!(MaybeValid::get(&object).is_none());
    }

    #[test]
    fn value_ok() {
        let object = MaybeValid::new(100i32);
        assert_eq!(MaybeValid::get(&object).unwrap(), &100i32);
    }

    #[test]
    fn value_invalidated() {
        let object = MaybeValid::new(100i32);
        object.invalidate();
        assert!(MaybeValid::get(&object).is_none());
    }

    #[test]
    fn value_invalidated_and_initialized() {
        let mut object = MaybeValid::new(100i32);
        object.invalidate();
        object.set(-10i32);
        assert_eq!(MaybeValid::get(&object).unwrap(), &-10i32);
    }
}
