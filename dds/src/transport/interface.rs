use crate::transport::types::{Locator, TransportHandle};

pub trait Write {
    fn write(&mut self, buf: &[u8]);
    fn flush(&mut self);
}

pub trait RtpsParticipant {
    fn default_unicast_locator_list(&self, handle: TransportHandle) -> &[Locator];
    fn metatraffic_unicast_locator_list(&self, handle: TransportHandle) -> &[Locator];
    fn metatraffic_multicast_locator_list(&self, handle: TransportHandle) -> &[Locator];
    fn default_multicast_locator_list(&self, handle: TransportHandle) -> &[Locator];
    fn fragment_size(&self, handle: TransportHandle) -> usize;
}
pub trait Transport: Send + 'static {
    fn create_participant(&mut self, domain_id: i32) -> TransportHandle;

    fn delete_participant(&mut self, handle: TransportHandle);

    fn writer<'a>(&'a mut self, handle: TransportHandle, locator: &'a [Locator])
    -> impl Write + 'a;

    fn read(&mut self) -> impl Future<Output = &[u8]> + Send;
}
