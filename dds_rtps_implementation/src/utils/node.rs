use rust_dds_api::return_type::{DDSError, DDSResult};

use super::shared_maybe_valid::MaybeValidReadRef;

pub struct Node<'a, P, I> {
    parent: P,
    impl_ref: MaybeValidReadRef<'a, I>,
}

impl<'a, P, I> Node<'a, P, I> {
    pub fn new(parent: P, impl_ref: MaybeValidReadRef<'a, I>) -> Self {
        Self { parent, impl_ref }
    }

    pub fn _impl(&self) -> DDSResult<&I> {
        self.impl_ref.get().ok_or(DDSError::AlreadyDeleted)
    }

    pub fn _parent(&self) -> &P {
        &self.parent
    }

    pub fn _impl_ref(&self) -> &MaybeValidReadRef<'a, I> {
        &self.impl_ref
    }
}
