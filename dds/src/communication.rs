use std::cell::RefCell;

use rust_dds_rtps_implementation::{
    dds_impl::{
        data_writer_proxy::RtpsWriter, publisher_proxy::PublisherAttributes,
        subscriber_proxy::SubscriberAttributes,
    },
    utils::shared_object::RtpsShared,
};
use rust_rtps_pim::{
    behavior::{
        stateful_writer_behavior::StatefulWriterBehavior,
        stateless_writer_behavior::StatelessWriterBehavior,
        writer::{
            reader_proxy::RtpsReaderProxyAttributes, reader_locator::RtpsReaderLocatorAttributes,
        },
    },
    messages::overall_structure::RtpsMessageHeader,
    structure::types::{
        GuidPrefix, ProtocolVersion, VendorId, GUIDPREFIX_UNKNOWN, PROTOCOLVERSION, VENDOR_ID_S2E,
    },
};
use rust_rtps_udp_psm::messages::{
    overall_structure::{RtpsMessageWrite, RtpsSubmessageTypeWrite},
    submessages::DataSubmessageWrite,
};

use crate::{
    domain_participant_factory::RtpsStructureImpl, message_receiver::MessageReceiver,
    transport::{TransportWrite, TransportRead},
};

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
    pub fn send(&mut self, list: &[RtpsShared<PublisherAttributes<RtpsStructureImpl>>]) {
        for publisher in list {
            let mut publisher_lock = publisher.write_lock();

            let message_header = RtpsMessageHeader {
                protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
                version: PROTOCOLVERSION,
                vendor_id: VENDOR_ID_S2E,
                guid_prefix: GUIDPREFIX_UNKNOWN,
            };

            for any_data_writer in &mut publisher_lock.data_writer_list {
                let mut rtps_writer_lock = any_data_writer.write_lock();
                let rtps_writer = &mut rtps_writer_lock.rtps_writer;

                match rtps_writer {
                    RtpsWriter::Stateless(stateless_rtps_writer) => {
                        let mut destined_submessages = Vec::new();
                        for behavior in stateless_rtps_writer {
                            match behavior {
                                StatelessWriterBehavior::BestEffort(mut best_effort_behavior) => {
                                    let submessages = RefCell::new(Vec::new());
                                    best_effort_behavior.send_unsent_changes(
                                        |data: DataSubmessageWrite| {
                                            submessages
                                                .borrow_mut()
                                                .push(RtpsSubmessageTypeWrite::Data(data))
                                        },
                                        |gap| {
                                            submessages
                                                .borrow_mut()
                                                .push(RtpsSubmessageTypeWrite::Gap(gap))
                                        },
                                    );
                                    let submessages = submessages.take();
                                    if !submessages.is_empty() {
                                        destined_submessages.push((
                                            best_effort_behavior
                                                .reader_locator
                                                .reader_locator_attributes
                                                .locator(),
                                            submessages,
                                        ));
                                    }
                                }
                                StatelessWriterBehavior::Reliable(_) => todo!(),
                            };
                        }

                        for (locator, submessage) in destined_submessages {
                            let message = RtpsMessageWrite::new(message_header.clone(), submessage);
                            self.transport.write(&message, locator);
                        }
                    }
                    RtpsWriter::Stateful(stateful_rtps_writer) => {
                        let mut destined_submessages = Vec::new();

                        for behavior in stateful_rtps_writer {
                            match behavior {
                                StatefulWriterBehavior::BestEffort(_) => todo!(),
                                StatefulWriterBehavior::Reliable(mut reliable_behavior) => {
                                    let submessages = RefCell::new(Vec::new());
                                    reliable_behavior.send_heartbeat(&mut |heartbeat| {
                                        submessages
                                            .borrow_mut()
                                            .push(RtpsSubmessageTypeWrite::Heartbeat(heartbeat));
                                    });

                                    reliable_behavior.send_unsent_changes(
                                        |data| {
                                            submessages
                                                .borrow_mut()
                                                .push(RtpsSubmessageTypeWrite::Data(data))
                                        },
                                        |gap| {
                                            submessages
                                                .borrow_mut()
                                                .push(RtpsSubmessageTypeWrite::Gap(gap))
                                        },
                                    );

                                    let submessages = submessages.take();

                                    if !submessages.is_empty() {
                                        let reader_proxy_attributes: &dyn RtpsReaderProxyAttributes =
                                                    reliable_behavior.reader_proxy.reader_proxy_attributes;
                                        destined_submessages
                                            .push((reader_proxy_attributes, submessages));
                                    }
                                }
                            }
                        }
                        for (reader_proxy, submessage) in destined_submessages {
                            let mut message_header = message_header.clone();
                            message_header.guid_prefix = reader_proxy.remote_reader_guid().prefix;
                            let message = RtpsMessageWrite::new(message_header, submessage);
                            self.transport
                                .write(&message, &reader_proxy.unicast_locator_list()[0]);
                        }
                    }
                }
            }
        }
    }
}
impl<T> Communication<T>
where
    T: TransportRead,
{
    pub fn receive(&mut self, list: &[RtpsShared<SubscriberAttributes<RtpsStructureImpl>>]) {
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