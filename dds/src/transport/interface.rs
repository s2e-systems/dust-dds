use crate::transport::types::{Locator, TransportHandle};
use alloc::{boxed::Box, vec::Vec};

pub trait WriteMessage {
    fn write_buffer_mut(&mut self) -> &mut [u8];
    fn write_message(&mut self, len: usize, locators: &[Locator]);
}

pub struct RtpsTransportParticipant {
    pub transport_handle: TransportHandle,
    pub message_writer: Box<dyn WriteMessage + Send + Sync>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub metatraffic_unicast_locator_list: Vec<Locator>,
    pub metatraffic_multicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub fragment_size: usize,
}
pub trait TransportParticipantFactory: Send + 'static {
    fn create_participant(&mut self, domain_id: i32) -> RtpsTransportParticipant;

    fn read(&mut self) -> impl Future<Output = (TransportHandle, &[u8])> + Send;
}
