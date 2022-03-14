use std::cell::RefCell;

use rust_dds_rtps_implementation::{
    dds_impl::{
        data_writer_proxy::RtpsWriter, publisher_proxy::PublisherAttributes,
        subscriber_proxy::SubscriberAttributes,
    },
    rtps_impl::{
        rtps_reader_locator_impl::RtpsReaderLocatorOperationsImpl,
        rtps_reader_proxy_impl::RtpsReaderProxyOperationsImpl,
    },
    utils::shared_object::RtpsShared,
};
use rust_rtps_pim::{
    behavior::{
        stateful_writer_behavior::{ReliableStatefulWriterBehavior, BestEffortStatefulWriterBehavior},
        stateless_writer_behavior::{
            BestEffortStatelessWriterBehavior, ReliableStatelessWriterBehavior,
        },
        writer::{
            reader_locator::RtpsReaderLocatorAttributes, reader_proxy::RtpsReaderProxyAttributes,
        },
    },
    messages::overall_structure::RtpsMessageHeader,
    structure::types::{
        GuidPrefix, ProtocolVersion, ReliabilityKind, VendorId, PROTOCOLVERSION, VENDOR_ID_S2E,
    },
};
use rust_rtps_udp_psm::messages::overall_structure::{RtpsMessage, RtpsSubmessageType};

use crate::{
    domain_participant_factory::RtpsStructureImpl,
    message_receiver::MessageReceiver,
    transport::{TransportRead, TransportWrite},
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
                guid_prefix: publisher_lock.rtps_group.entity.guid.prefix(),
            };

            for any_data_writer in &mut publisher_lock.data_writer_list {
                let mut rtps_writer_lock = any_data_writer.write_lock();
                let rtps_writer = &mut rtps_writer_lock.rtps_writer;

                match rtps_writer {
                    RtpsWriter::Stateless(stateless_rtps_writer) => {
                        let mut destined_submessages = Vec::new();
                        for reader_locator in &mut stateless_rtps_writer.reader_locators {
                            match stateless_rtps_writer.writer.endpoint.reliability_level {
                                ReliabilityKind::BestEffort => {
                                    let submessages = RefCell::new(Vec::new());
                                    let writer_cache = &stateless_rtps_writer.writer.writer_cache;
                                    BestEffortStatelessWriterBehavior::send_unsent_changes(
                                        &mut RtpsReaderLocatorOperationsImpl::new(
                                            reader_locator,
                                            writer_cache,
                                        ),
                                        writer_cache,
                                        |data| {
                                            submessages
                                                .borrow_mut()
                                                .push(RtpsSubmessageType::Data(data))
                                        },
                                        |gap| {
                                            submessages
                                                .borrow_mut()
                                                .push(RtpsSubmessageType::Gap(gap))
                                        },
                                    );

                                    let submessages = submessages.take();
                                    if !submessages.is_empty() {
                                        destined_submessages
                                            .push((reader_locator.locator(), submessages));
                                    }
                                }
                                ReliabilityKind::Reliable => {
                                    let submessages = RefCell::new(Vec::new());
                                    let writer_cache = &stateless_rtps_writer.writer.writer_cache;
                                    ReliableStatelessWriterBehavior::send_unsent_changes(
                                        &mut RtpsReaderLocatorOperationsImpl::new(
                                            reader_locator,
                                            writer_cache,
                                        ),
                                        writer_cache,
                                        |data| {
                                            submessages
                                                .borrow_mut()
                                                .push(RtpsSubmessageType::Data(data))
                                        },
                                        |gap| {
                                            submessages
                                                .borrow_mut()
                                                .push(RtpsSubmessageType::Gap(gap))
                                        },
                                    );

                                    let submessages = submessages.take();
                                    if !submessages.is_empty() {
                                        destined_submessages
                                            .push((reader_locator.locator(), submessages));
                                    }
                                }
                            };
                        }

                        for (locator, submessage) in destined_submessages {
                            let mut message_header = message_header.clone();
                            message_header.guid_prefix = stateless_rtps_writer.writer.endpoint.entity.guid.prefix;
                            let message = RtpsMessage::new(message_header.clone(), submessage);
                            self.transport.write(&message, locator);
                        }
                    }
                    RtpsWriter::Stateful(stateful_rtps_writer) => {
                        let mut destined_submessages = Vec::new();

                        for reader_proxy in &mut stateful_rtps_writer.matched_readers {
                            match stateful_rtps_writer.writer.endpoint.reliability_level {
                                ReliabilityKind::BestEffort => {
                                    let submessages = RefCell::new(Vec::new());
                                    let reader_id = reader_proxy.remote_reader_guid().entity_id();
                                    BestEffortStatefulWriterBehavior::send_unsent_changes(
                                        &mut RtpsReaderProxyOperationsImpl::new(
                                            reader_proxy,
                                            &stateful_rtps_writer.writer.writer_cache,
                                            stateful_rtps_writer.writer.push_mode,
                                        ),
                                        &stateful_rtps_writer.writer.writer_cache,
                                        reader_id,
                                        |data| {
                                            submessages
                                                .borrow_mut()
                                                .push(RtpsSubmessageType::Data(data))
                                        },
                                        |gap| {
                                            submessages
                                                .borrow_mut()
                                                .push(RtpsSubmessageType::Gap(gap))
                                        },
                                    );

                                    let submessages = submessages.take();

                                    if !submessages.is_empty() {
                                        destined_submessages.push((reader_proxy, submessages));
                                    }
                                },

                                ReliabilityKind::Reliable => {
                                    let submessages = RefCell::new(Vec::new());
                                    // ReliableStatefulWriterBehavior::send_heartbeat(
                                    //     &stateful_rtps_writer.writer.writer_cache,
                                    //     stateful_rtps_writer.writer.endpoint.entity.guid.entity_id,
                                    //     stateful_rtps_writer.heartbeat_count,
                                    //     &mut |heartbeat| {
                                    //         submessages
                                    //             .borrow_mut()
                                    //             .push(RtpsSubmessageType::Heartbeat(heartbeat));
                                    //     },
                                    // );
                                    let reader_id = reader_proxy.remote_reader_guid().entity_id();
                                    ReliableStatefulWriterBehavior::send_unsent_changes(
                                        &mut RtpsReaderProxyOperationsImpl::new(
                                            reader_proxy,
                                            &stateful_rtps_writer.writer.writer_cache,
                                            stateful_rtps_writer.writer.push_mode,
                                        ),
                                        &stateful_rtps_writer.writer.writer_cache,
                                        reader_id,
                                        |data| {
                                            submessages
                                                .borrow_mut()
                                                .push(RtpsSubmessageType::Data(data))
                                        },
                                        |gap| {
                                            submessages
                                                .borrow_mut()
                                                .push(RtpsSubmessageType::Gap(gap))
                                        },
                                    );

                                    let submessages = submessages.take();

                                    if !submessages.is_empty() {
                                        destined_submessages.push((reader_proxy, submessages));
                                    }
                                }
                            }
                        }
                        for (reader_proxy, submessage) in destined_submessages {
                            let mut message_header = message_header.clone();
                            message_header.guid_prefix =
                                stateful_rtps_writer.writer.endpoint.entity.guid.prefix;
                            let message = RtpsMessage::new(message_header, submessage);
                            self.transport
                                .write(&message, reader_proxy.unicast_locator_list()[0]);
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
        while let Some((source_locator, message)) = self.transport.read() {
            MessageReceiver::new().process_message(
                self.guid_prefix,
                list,
                source_locator,
                &message,
            );
        }
    }
}
