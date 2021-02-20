use std::ops::Deref;

use super::shared_option::SharedOptionReadRef;

pub struct Node<'a, P, I> {
    pub(crate) parent: P,
    pub(crate) impl_ref: SharedOptionReadRef<'a, I>,
}

impl<'a, P, I> Deref for Node<'a, P, I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &*self.impl_ref
    }
}
