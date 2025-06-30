use core::future::Future;

use crate::transport::types::{GuidPrefix, Locator};

pub trait WriteMessage {
    fn write_message(
        &mut self,
        datagram: &[u8],
        locator_list: &[Locator],
    );
    fn guid_prefix(&self) -> GuidPrefix;
}

pub trait Clock {
    fn now(&self) -> core::time::Duration;
}
