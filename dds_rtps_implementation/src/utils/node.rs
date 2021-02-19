use std::ops::Deref;

use super::{shared_option::SharedOptionReadRef};

pub struct Node<'a, P, I> {
    parent: P,
    impl_ref: SharedOptionReadRef<'a, I>,
}

impl<'a, P, I> Node<'a, P, I> {
    pub fn new(parent: P, impl_ref: SharedOptionReadRef<'a, I>) -> Self {
        Self { parent, impl_ref }
    }

    pub fn parent(&self) -> &P {
        &self.parent
    }

    pub fn impl_ref(&self) -> &SharedOptionReadRef<'a, I> {
        &self.impl_ref
    }
}

impl<'a, P, I> Deref for Node<'a, P, I>{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &*self.impl_ref
    }
}