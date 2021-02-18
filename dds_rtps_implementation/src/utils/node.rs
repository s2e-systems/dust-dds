use rust_dds_api::return_type::{DDSError, DDSResult};

use crate::utils::maybe_valid::{MaybeValid, MaybeValidRef};

pub struct Node<'a, P, I> {
    parent: &'a P,
    impl_ref: MaybeValidRef<'a, I>,
}

impl<'a, P, I> Node<'a, P, I> {
    pub fn new(parent: &'a P, impl_ref: MaybeValidRef<'a, I>,) -> Self {
        Self {
            parent,
            impl_ref,
        }
    }

    pub fn get_impl(&self) -> DDSResult<&I> {
        MaybeValid::get(&self.impl_ref).ok_or(DDSError::AlreadyDeleted)
    }

    pub fn get_parent(&self) -> &P {
        self.parent
    }
}
