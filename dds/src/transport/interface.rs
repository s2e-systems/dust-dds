use crate::transport::types::{Locator, TransportHandle};

pub trait Write {
    fn write(&mut self, buf: &[u8]);
    fn flush(&mut self);
}

pub trait RtpsParticipant: Send + 'static {
    type Writer<'a>: Write
    where
        Self: 'a;

    fn default_unicast_locator_list(&self) -> &[Locator];
    fn metatraffic_unicast_locator_list(&self) -> &[Locator];
    fn metatraffic_multicast_locator_list(&self) -> &[Locator];
    fn default_multicast_locator_list(&self) -> &[Locator];
    fn fragment_size(&self) -> usize;

    fn writer<'a>(&'a mut self, locator: &'a [Locator]) -> Self::Writer<'a>;

    fn read(&mut self) -> impl Future<Output = &[u8]> + Send;
}
pub trait Transport: Send + 'static {
    type Participant: RtpsParticipant;
    fn create_participant(&mut self, domain_id: i32) -> Self::Participant;

    fn delete_participant(&mut self, handle: TransportHandle);
}
