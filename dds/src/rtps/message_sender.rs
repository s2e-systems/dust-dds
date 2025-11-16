use crate::transport::types::{GuidPrefix, Locator};
use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

pub trait WriteMessage {
    fn write_message(
        &self,
        buf: &[u8],
        locators: &[Locator],
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
    fn guid_prefix(&self) -> GuidPrefix;
}

pub trait Clock {
    fn now(&self) -> core::time::Duration;
}
