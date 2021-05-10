use std::{ops::Deref, sync::{Arc, Weak}};

use rust_dds_api::return_type::{DDSError, DDSResult};

pub struct RtpsShared<T>(Arc<T>);

impl<T> RtpsShared<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(value))
    }

    pub fn downgrade(&self) -> RtpsWeak<T> {
        RtpsWeak(Arc::downgrade(&self.0))
    }
}

impl<T> Clone for RtpsShared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for RtpsShared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RtpsWeak<T>(Weak<T>);

impl<T> RtpsWeak<T> {
    pub fn upgrade(&self) -> DDSResult<RtpsShared<T>> {
        Ok(RtpsShared(
            self.0.upgrade().ok_or(DDSError::AlreadyDeleted)?,
        ))
    }
}
