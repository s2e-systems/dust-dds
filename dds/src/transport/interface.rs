use super::types::{CacheChange, Guid, GuidPrefix, Locator, ProtocolVersion, VendorId};
use crate::{
    dcps::channels::mpsc::MpscSender,
    rtps::types::{PROTOCOLVERSION, VENDOR_ID_S2E},
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{future::Future, pin::Pin};

pub trait WriteMessage {
    fn write_message(
        &self,
        buf: &[u8],
        locators: &[Locator],
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
    fn guid_prefix(&self) -> GuidPrefix;

    fn box_clone(&self) -> Box<dyn WriteMessage + Send + Sync + 'static>;
}

pub struct RtpsTransportParticipant {
    pub guid: Guid,
    pub message_writer: Box<dyn WriteMessage + Send + Sync + 'static>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub metatraffic_unicast_locator_list: Vec<Locator>,
    pub metatraffic_multicast_locator_list: Vec<Locator>,
    pub fragment_size: usize,
}

impl RtpsTransportParticipant {
    pub fn guid(&self) -> Guid {
        self.guid
    }
    pub fn protocol_version(&self) -> ProtocolVersion {
        PROTOCOLVERSION
    }
    pub fn vendor_id(&self) -> VendorId {
        VENDOR_ID_S2E
    }
    pub fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        &self.metatraffic_unicast_locator_list
    }
    pub fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        &self.metatraffic_multicast_locator_list
    }
    pub fn default_unicast_locator_list(&self) -> &[Locator] {
        &self.default_unicast_locator_list
    }
    pub fn default_multicast_locator_list(&self) -> &[Locator] {
        &[]
    }
}

pub trait TransportParticipantFactory: Send + 'static {
    fn create_participant(
        &self,
        guid_prefix: GuidPrefix,
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
