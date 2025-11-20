use super::types::CacheChange;
use crate::{dcps::channels::mpsc::MpscSender, transport::types::Locator};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{future::Future, pin::Pin};

pub trait WriteMessage {
    fn write_message(
        &self,
        buf: &[u8],
        locators: &[Locator],
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

pub struct RtpsTransportParticipant {
    pub message_writer: Box<dyn WriteMessage + Send + Sync>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub metatraffic_unicast_locator_list: Vec<Locator>,
    pub metatraffic_multicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub fragment_size: usize,
}

pub trait TransportParticipantFactory: Send + 'static {
    fn create_participant(
        &self,
        domain_id: i32,
        data_channel_sender: MpscSender<Arc<[u8]>>,
    ) -> impl Future<Output = RtpsTransportParticipant> + Send;
}

pub trait HistoryCache: Send {
    fn add_change(
        &mut self,
        cache_change: CacheChange,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

    fn remove_change(&mut self, sequence_number: i64) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}
