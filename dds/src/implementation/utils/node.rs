use std::fmt::Debug;

use crate::infrastructure::error::DdsResult;

use super::shared_object::{DdsShared, DdsWeak};

pub struct RootNode<T> {
    node: DdsWeak<T>,
}

impl<T> PartialEq for RootNode<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<T> Debug for RootNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChildNode")
            .field("node", &self.node)
            .finish()
    }
}

impl<T> RootNode<T> {
    pub fn new(node: DdsWeak<T>) -> Self {
        Self { node }
    }

    pub fn get(&self) -> DdsResult<DdsShared<T>> {
        self.node.upgrade()
    }
}

pub struct ChildNode<T, U> {
    node: DdsWeak<T>,
    parent: U,
}

impl<T, U> PartialEq for ChildNode<T, U>
where
    U: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.parent == other.parent
    }
}

impl<T, U> Debug for ChildNode<T, U>
where
    U: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChildNode")
            .field("node", &self.node)
            .field("parent", &self.parent)
            .finish()
    }
}

impl<T, U> ChildNode<T, U> {
    pub fn new(node: DdsWeak<T>, parent: U) -> Self {
        Self { node, parent }
    }

    pub fn get(&self) -> DdsResult<DdsShared<T>> {
        self.node.upgrade()
    }

    pub fn get_parent(&self) -> &U {
        &self.parent
    }
}
