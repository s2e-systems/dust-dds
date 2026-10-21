use crate::transport::types::{Locator, TransportHandle};

pub trait Write {
    fn write(&mut self, buf: &[u8]);
    fn flush(&mut self);
}

pub trait RtpsParticipant {
    type Writer<'a>: Write
    where
        Self: 'a;

    fn default_unicast_locator_list(&self) -> &[Locator];
    fn metatraffic_unicast_locator_list(&self) -> &[Locator];
    fn metatraffic_multicast_locator_list(&self) -> &[Locator];
    fn default_multicast_locator_list(&self) -> &[Locator];
    fn fragment_size(&self) -> usize;

    fn writer<'a>(&'a mut self, locator: &'a [Locator]) -> Self::Writer<'a>;
}
pub trait Transport: Send + 'static {
    type Participant: RtpsParticipant;
    fn create_participant(&mut self, domain_id: i32) -> TransportHandle;
    fn get_participant_mut(&mut self, handle: &TransportHandle) -> &mut Self::Participant;
    fn delete_participant(&mut self, handle: TransportHandle);

    fn read(&mut self) -> impl Future<Output = (TransportHandle, &[u8])> + Send;
}
