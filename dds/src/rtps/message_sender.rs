use crate::transport::types::{GuidPrefix, Locator};

pub trait WriteMessageMut {
    fn write_message_mut(&mut self, datagram: &[u8], locator_list: &[Locator]);
    fn guid_prefix(&self) -> GuidPrefix;
}

pub trait WriteMessage {
    fn write_message(&self, datagram: &[u8], locator_list: &[Locator]);
    fn guid_prefix(&self) -> GuidPrefix;
}
impl<A: WriteMessage> WriteMessageMut for A {
    fn guid_prefix(&self) -> GuidPrefix {
        self.guid_prefix()
    }

    fn write_message_mut(&mut self, datagram: &[u8], locator_list: &[Locator]) {
        self.write_message(datagram, locator_list);
    }
}

pub trait Clock {
    fn now(&self) -> core::time::Duration;
}
