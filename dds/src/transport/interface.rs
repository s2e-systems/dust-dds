use super::types::{CacheChange, Guid, GuidPrefix, Locator, ProtocolVersion, VendorId};
use crate::{
    dcps::channels::mpsc::MpscSender,
    rtps::{
        message_sender::WriteMessage,
        stateful_reader::RtpsStatefulReader,
        stateful_writer::RtpsStatefulWriter,
        stateless_reader::RtpsStatelessReader,
        types::{PROTOCOLVERSION, VENDOR_ID_S2E},
    },
    transport::types::{ReaderProxy, WriterProxy},
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{cell::RefCell, future::Future, pin::Pin};
use critical_section::Mutex;

pub enum ChannelMessageKind {
    AddStatelessReader(RtpsStatelessReader),
    AddStatefulReader(Arc<Mutex<RefCell<RtpsStatefulReader>>>),
    AddStatefulWriter(Arc<Mutex<RefCell<RtpsStatefulWriter>>>),
    DataArrived(Arc<[u8]>),
    Poke,
}

pub struct RtpsTransportParticipant {
    pub guid: Guid,
    pub message_writer: Box<dyn WriteMessage + Send + Sync + 'static>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub metatraffic_unicast_locator_list: Vec<Locator>,
    pub metatraffic_multicast_locator_list: Vec<Locator>,
    pub fragment_size: usize,
    pub chanel_message_sender: MpscSender<ChannelMessageKind>,
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

pub struct RtpsTransportStatefulReader {
    guid: Guid,
    rtps_stateful_reader: Arc<Mutex<RefCell<RtpsStatefulReader>>>,
}
impl RtpsTransportStatefulReader {
    pub fn guid(&self) -> Guid {
        self.guid
    }
    pub async fn is_historical_data_received(&self) -> bool {
        critical_section::with(|cs| {
            self.rtps_stateful_reader
                .borrow(cs)
                .borrow()
                .is_historical_data_received()
        })
    }
    pub async fn add_matched_writer(&mut self, writer_proxy: WriterProxy) {
        critical_section::with(|cs| {
            self.rtps_stateful_reader
                .borrow(cs)
                .borrow_mut()
                .add_matched_writer(&writer_proxy)
        })
    }
    pub async fn remove_matched_writer(&mut self, remote_writer_guid: Guid) {
        critical_section::with(|cs| {
            self.rtps_stateful_reader
                .borrow(cs)
                .borrow_mut()
                .delete_matched_writer(remote_writer_guid)
        })
    }
}

pub struct RtpsTransportStatefulWriter {
    pub guid: Guid,
    pub rtps_stateful_writer: Arc<Mutex<RefCell<RtpsStatefulWriter>>>,
    pub message_writer: Box<dyn WriteMessage + Send + Sync>,
    pub default_unicast_locator_list: Vec<Locator>,
}
impl RtpsTransportStatefulWriter {
    pub fn guid(&self) -> Guid {
        self.guid
    }
    pub async fn is_change_acknowledged(&self, sequence_number: i64) -> bool {
        critical_section::with(|cs| {
            self.rtps_stateful_writer
                .borrow(cs)
                .borrow()
                .is_change_acknowledged(sequence_number)
        })
    }
    pub async fn add_matched_reader(&self, mut reader_proxy: ReaderProxy) {
        if reader_proxy.unicast_locator_list.is_empty() {
            reader_proxy
                .unicast_locator_list
                .clone_from(&self.default_unicast_locator_list);
        }
        critical_section::with(move |cs| {
            self.rtps_stateful_writer
                .borrow(cs)
                .borrow_mut()
                .add_matched_reader(reader_proxy);
        })
    }
    pub async fn remove_matched_reader(&mut self, remote_reader_guid: Guid) {
        critical_section::with(move |cs| {
            self.rtps_stateful_writer
                .borrow(cs)
                .borrow_mut()
                .delete_matched_reader(remote_reader_guid);
        })
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
