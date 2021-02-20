pub struct Node<P, I> {
    pub(crate) parent: P,
    pub(crate) impl_ref: I,
}