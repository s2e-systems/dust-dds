use rust_rtps_pim::structure::types::{GuidPrefix, ProtocolVersion, VendorId};

use crate::dds_impl::{domain_participant_impl::Transport, publisher_impl::PublisherImpl};

use super::{message_receiver::ProcessDataSubmessage, shared_object::RtpsShared};

pub struct Communication {
    pub version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
    pub transport: Box<dyn Transport>,
}

impl Communication {
    pub fn send(&mut self, list: &[RtpsShared<PublisherImpl>]) {
        for publisher in list {
            publisher
                .write()
                .unwrap()
                .send_message(self.transport.as_mut());
        }
        // if let Ok((dst_locator, submessages)) =
        //     self.locator_message_channel_receiver.try_recv()
        // {
        //     let message = RtpsMessageWrite::new(
        //         RtpsMessageHeader {
        //             protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
        //             version: self.version,
        //             vendor_id: self.vendor_id,
        //             guid_prefix: self.guid_prefix,
        //         },
        //         submessages,
        //     );
        //     self.transport.write(&message, &dst_locator);
        // };
    }

    pub fn receive(&mut self, list: &[RtpsShared<impl ProcessDataSubmessage>]) {
        if let Some((source_locator, message)) = self.transport.read() {
            crate::utils::message_receiver::MessageReceiver::new().process_message(
                self.guid_prefix,
                list,
                source_locator,
                &message,
            );
        }
    }
}
