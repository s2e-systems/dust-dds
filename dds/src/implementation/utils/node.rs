use crate::infrastructure::error::DdsResult;

use super::shared_object::{DdsShared, DdsWeak};

pub struct RootNode<T> {
    node: DdsWeak<T>,
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
    parent: DdsWeak<U>,
}

impl<T, U> ChildNode<T, U> {
    pub fn new(node: DdsWeak<T>, parent: DdsWeak<U>) -> Self {
        Self { node, parent }
    }

    pub fn get(&self) -> DdsResult<DdsShared<T>> {
        self.node.upgrade()
    }

    pub fn get_parent(&self) -> DdsResult<DdsShared<U>> {
        self.parent.upgrade()
    }
}
