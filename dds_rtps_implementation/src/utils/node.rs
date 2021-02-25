use std::sync::Weak;

pub struct Node<P, I> {
    pub(crate) parent: P,
    pub(crate) impl_ref: Weak<I>,
}