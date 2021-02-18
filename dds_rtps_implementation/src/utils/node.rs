use rust_dds_api::return_type::{DDSError, DDSResult};

use crate::utils::maybe_valid::MaybeValidRef;

pub struct Node<'a, P, I> {
    parent: P,
    impl_ref: MaybeValidRef<'a, I>,
}

impl<'a, P, I> Node<'a, P, I> {
    pub fn new(parent: P, impl_ref: MaybeValidRef<'a, I>) -> Self {
        Self { parent, impl_ref }
    }

    pub fn _impl(&self) -> DDSResult<&I> {
        self.impl_ref.get().ok_or(DDSError::AlreadyDeleted)
    }

    pub fn _parent(&self) -> &P {
        &self.parent
    }

    pub fn _impl_ref(&self) -> &MaybeValidRef<'a, I> {
        &self.impl_ref
    }
}
