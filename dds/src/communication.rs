use rust_dds_rtps_implementation::{
    dds_impl::publisher_impl::PublisherImpl,
    utils::{
        message_receiver::{MessageReceiver, ProcessDataSubmessage},
        shared_object::RtpsShared,
        transport::{TransportRead, TransportWrite},
    },
};
use rust_rtps_pim::structure::types::{GuidPrefix, ProtocolVersion, VendorId};

pub struct Communication<T> {
    pub version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
    pub transport: T,
}

impl<T> Communication<T>
where
    T: TransportWrite,
{
    pub fn send(&mut self, list: &[RtpsShared<PublisherImpl>]) {
        for publisher in list {
            publisher.write().unwrap().send_message(&mut self.transport);
        }
    }
}
impl<T> Communication<T>
where
    T: TransportRead,
{
    pub fn receive(&mut self, list: &[RtpsShared<impl ProcessDataSubmessage>]) {
        if let Some((source_locator, message)) = self.transport.read() {
            MessageReceiver::new().process_message(
                self.guid_prefix,
                list,
                source_locator,
                &message,
            );
        }
    }
}
