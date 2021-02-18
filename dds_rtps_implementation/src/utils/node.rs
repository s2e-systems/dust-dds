use super::shared_maybe_valid::MaybeValidReadRef;

pub struct Node<'a, P, I> {
    parent: P,
    impl_ref: MaybeValidReadRef<'a, I>,
}

impl<'a, P, I> Node<'a, P, I> {
    pub fn new(parent: P, impl_ref: MaybeValidReadRef<'a, I>) -> Self {
        Self { parent, impl_ref }
    }

    pub fn parent(&self) -> &P {
        &self.parent
    }

    pub fn impl_ref(&self) -> &MaybeValidReadRef<'a, I> {
        &self.impl_ref
    }

    pub fn get(&self) -> Option<&I> {
        self.impl_ref.get()
    }
}
