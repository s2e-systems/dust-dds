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

impl<T> Clone for RootNode<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
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
