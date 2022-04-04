use std::{cell::RefCell, ops::DerefMut};

use dds_implementation::{
    dds_impl::{
        data_reader_proxy::RtpsReader, data_writer_proxy::RtpsWriter,
        publisher_proxy::PublisherAttributes, subscriber_proxy::SubscriberAttributes,
    },
    utils::shared_object::DdsShared,
};
use rtps_implementation::{
    rtps_reader_proxy_impl::RtpsReaderProxyOperationsImpl, utils::clock::Timer,
};
use rtps_pim::{
    behavior::{
        reader::writer_proxy::RtpsWriterProxyAttributes,
        stateful_writer_behavior::{
            BestEffortStatefulWriterBehavior, ReliableStatefulWriterBehavior,
        },
        stateless_writer_behavior::{
            BestEffortStatelessWriterBehavior, ReliableStatelessWriterBehavior,
        },
        writer::{
            reader_locator::RtpsReaderLocatorAttributes, reader_proxy::RtpsReaderProxyAttributes,
            stateful_writer::RtpsStatefulWriterAttributes,
            stateless_writer::RtpsStatelessWriterAttributes,
        },
    },
    messages::{
        overall_structure::{RtpsMessage, RtpsMessageHeader, RtpsSubmessageType},
        submessage_elements::{Parameter, TimestampSubmessageElement},
        submessages::InfoTimestampSubmessage,
        types::{Count, FragmentNumber, TIME_INVALID},
    },
    structure::{
        entity::RtpsEntityAttributes,
        types::{
            GuidPrefix, Locator, ProtocolVersion, ReliabilityKind, SequenceNumber, VendorId,
            PROTOCOLVERSION, VENDOR_ID_S2E,
        },
    },
    transport::{TransportRead, TransportWrite},
};

use crate::{domain_participant_factory::RtpsStructureImpl, message_receiver::MessageReceiver};

pub struct Communication<T> {
    pub version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
    pub transport: T,
}

impl<T> Communication<T>
where
    T: for<'a> TransportWrite<
        Vec<
            RtpsSubmessageType<
                Vec<SequenceNumber>,
                Vec<Parameter<'a>>,
                &'a [u8],
                Vec<Locator>,
                Vec<FragmentNumber>,
            >,
        >,
    >,
{
    pub fn send_publisher_message(&mut self, publisher: &PublisherAttributes<RtpsStructureImpl>) {
        let message_header = RtpsMessageHeader {
            protocol: rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
            version: PROTOCOLVERSION,
            vendor_id: VENDOR_ID_S2E,
            guid_prefix: publisher.rtps_group.entity.guid.prefix(),
        };

        for any_data_writer in publisher.data_writer_list.write_lock().iter_mut() {
            let mut rtps_writer = any_data_writer.rtps_writer.write_lock();

            match rtps_writer.deref_mut() {
                RtpsWriter::Stateless(stateless_rtps_writer) => {
                    let message_header = RtpsMessageHeader {
                        guid_prefix: stateless_rtps_writer.writer.endpoint.entity.guid.prefix,
                        ..message_header.clone()
                    };

                    let mut destined_submessages = Vec::new();

                    let time_for_heartbeat = stateless_rtps_writer.heartbeat_timer.elapsed()
                        >= std::time::Duration::from_secs(
                            stateless_rtps_writer.writer.heartbeat_period.seconds as u64,
                        ) + std::time::Duration::from_nanos(
                            stateless_rtps_writer.writer.heartbeat_period.fraction as u64,
                        );
                    if time_for_heartbeat {
                        stateless_rtps_writer.heartbeat_timer.reset();
                    }
                    let reliability_level = stateless_rtps_writer.writer.endpoint.reliability_level;
                    for reader_locator in &mut stateless_rtps_writer.reader_locators() {
                        let locator = reader_locator.locator();

                        match reliability_level {
                            ReliabilityKind::BestEffort => {
                                let submessages = RefCell::new(Vec::new());
                                BestEffortStatelessWriterBehavior::send_unsent_changes(
                                    reader_locator,
                                    |data| {
                                        let info_ts = if let Some(time) = any_data_writer
                                            .sample_info
                                            .read_lock()
                                            .get(&data.writer_sn.value)
                                        {
                                            InfoTimestampSubmessage {
                                                endianness_flag: true,
                                                invalidate_flag: false,
                                                timestamp: TimestampSubmessageElement {
                                                    value: rtps_pim::messages::types::Time(
                                                        ((time.sec as u64) << 32)
                                                            + time.nanosec as u64,
                                                    ),
                                                },
                                            }
                                        } else {
                                            InfoTimestampSubmessage {
                                                endianness_flag: true,
                                                invalidate_flag: true,
                                                timestamp: TimestampSubmessageElement {
                                                    value: TIME_INVALID,
                                                },
                                            }
                                        };
                                        submessages
                                            .borrow_mut()
                                            .push(RtpsSubmessageType::InfoTimestamp(info_ts));
                                        submessages
                                            .borrow_mut()
                                            .push(RtpsSubmessageType::Data(data));
                                    },
                                    |gap| {
                                        submessages.borrow_mut().push(RtpsSubmessageType::Gap(gap));
                                    },
                                );

                                let submessages = submessages.take();
                                if !submessages.is_empty() {
                                    destined_submessages.push((locator, submessages));
                                }
                            }

                            ReliabilityKind::Reliable => {
                                let submessages = RefCell::new(Vec::new());

                                // if time_for_heartbeat {
                                //     stateless_rtps_writer.heartbeat_count = Count(
                                //         stateless_rtps_writer.heartbeat_count.0.wrapping_add(1),
                                //     );

                                //     ReliableStatelessWriterBehavior::send_heartbeat(
                                //         &stateless_rtps_writer.writer.writer_cache,
                                //         stateless_rtps_writer.writer.endpoint.entity.guid.entity_id,
                                //         stateless_rtps_writer.heartbeat_count,
                                //         |heartbeat| {
                                //             submessages
                                //                 .borrow_mut()
                                //                 .push(RtpsSubmessageType::Heartbeat(heartbeat));
                                //         },
                                //     );
                                // }

                                ReliableStatelessWriterBehavior::send_unsent_changes(
                                    reader_locator,
                                    |data| {
                                        submessages
                                            .borrow_mut()
                                            .push(RtpsSubmessageType::Data(data));
                                    },
                                );

                                // ReliableStatelessWriterBehavior::send_requested_changes(
                                //     &mut RtpsReaderLocatorOperationsImpl::new(reader_locator, writer_cache),
                                //     writer_cache,
                                //     |data| {
                                //         submessages
                                //             .borrow_mut()
                                //             .push(RtpsStatelessSubmessage::Data(data))
                                //     },
                                //     |gap| {
                                //         submessages
                                //             .borrow_mut()
                                //             .push(RtpsStatelessSubmessage::Gap(gap))
                                //     },
                                // );

                                let submessages = submessages.take();
                                if !submessages.is_empty() {
                                    destined_submessages.push((locator, submessages));
                                }
                            }
                        }
                    }

                    for (locator, submessages) in destined_submessages {
                        self.transport.write(
                            &RtpsMessage {
                                header: message_header.clone(),
                                submessages,
                            },
                            locator,
                        );
                    }
                }
                RtpsWriter::Stateful(stateful_rtps_writer) => {
                    let message_header = RtpsMessageHeader {
                        guid_prefix: stateful_rtps_writer.writer.endpoint.entity.guid.prefix,
                        ..message_header.clone()
                    };

                    let mut destined_submessages = Vec::new();

                    let reliability_level = stateful_rtps_writer.writer.endpoint.reliability_level;
                    for reader_proxy in &mut stateful_rtps_writer.matched_readers() {
                        let unicast_locator_list = reader_proxy.unicast_locator_list().to_vec();
                        match reliability_level {
                            ReliabilityKind::BestEffort => todo!(),
                            ReliabilityKind::Reliable => {
                                let submessages = RefCell::new(Vec::new());
                                ReliableStatefulWriterBehavior::send_requested_changes(
                                    reader_proxy,
                                    |data| {
                                        submessages
                                            .borrow_mut()
                                            .push(RtpsSubmessageType::Data(data))
                                    },
                                    |gap| {
                                        submessages.borrow_mut().push(RtpsSubmessageType::Gap(gap))
                                    },
                                );
                                let submessages = submessages.take();

                                if !submessages.is_empty() {
                                    destined_submessages.push((unicast_locator_list, submessages));
                                }
                            }
                        }
                    }

                    for (locator_list, submessages) in destined_submessages {
                        self.transport.write(
                            &RtpsMessage {
                                header: message_header.clone(),
                                submessages,
                            },
                            locator_list[0],
                        )
                    }

                    let mut destined_submessages = Vec::new();

                    let time_for_heartbeat = stateful_rtps_writer.heartbeat_timer.elapsed()
                        >= std::time::Duration::from_secs(
                            stateful_rtps_writer.writer.heartbeat_period.seconds as u64,
                        ) + std::time::Duration::from_nanos(
                            stateful_rtps_writer.writer.heartbeat_period.fraction as u64,
                        );
                    if time_for_heartbeat {
                        stateful_rtps_writer.heartbeat_timer.reset();
                    }

                    for reader_proxy in &mut stateful_rtps_writer.matched_readers {
                        let unicast_locator_list = reader_proxy.unicast_locator_list().to_vec();

                        match stateful_rtps_writer.writer.endpoint.reliability_level {
                            ReliabilityKind::BestEffort => {
                                let submessages = RefCell::new(Vec::new());
                                BestEffortStatefulWriterBehavior::send_unsent_changes(
                                    &mut RtpsReaderProxyOperationsImpl::new(
                                        reader_proxy,
                                        &stateful_rtps_writer.writer.writer_cache,
                                    ),
                                    |data| {
                                        submessages
                                            .borrow_mut()
                                            .push(RtpsSubmessageType::Data(data))
                                    },
                                    |gap| {
                                        submessages.borrow_mut().push(RtpsSubmessageType::Gap(gap))
                                    },
                                );

                                let submessages = submessages.take();

                                if !submessages.is_empty() {
                                    destined_submessages.push((unicast_locator_list, submessages));
                                }
                            }

                            ReliabilityKind::Reliable => {
                                let submessages = RefCell::new(Vec::new());

                                if time_for_heartbeat {
                                    stateful_rtps_writer.heartbeat_count = Count(
                                        stateful_rtps_writer.heartbeat_count.0.wrapping_add(1),
                                    );

                                    ReliableStatefulWriterBehavior::send_heartbeat(
                                        &stateful_rtps_writer.writer.writer_cache,
                                        stateful_rtps_writer.writer.endpoint.entity.guid.entity_id,
                                        stateful_rtps_writer.heartbeat_count,
                                        |heartbeat| {
                                            submessages
                                                .borrow_mut()
                                                .push(RtpsSubmessageType::Heartbeat(heartbeat));
                                        },
                                    );
                                }

                                ReliableStatefulWriterBehavior::send_unsent_changes(
                                    &mut RtpsReaderProxyOperationsImpl::new(
                                        reader_proxy,
                                        &stateful_rtps_writer.writer.writer_cache,
                                    ),
                                    |data| {
                                        let info_ts = if let Some(time) = any_data_writer
                                            .sample_info
                                            .read_lock()
                                            .get(&data.writer_sn.value)
                                        {
                                            InfoTimestampSubmessage {
                                                endianness_flag: true,
                                                invalidate_flag: false,
                                                timestamp: TimestampSubmessageElement {
                                                    value: rtps_pim::messages::types::Time(
                                                        ((time.sec as u64) << 32)
                                                            + time.nanosec as u64,
                                                    ),
                                                },
                                            }
                                        } else {
                                            InfoTimestampSubmessage {
                                                endianness_flag: true,
                                                invalidate_flag: true,
                                                timestamp: TimestampSubmessageElement {
                                                    value: TIME_INVALID,
                                                },
                                            }
                                        };
                                        submessages
                                            .borrow_mut()
                                            .push(RtpsSubmessageType::InfoTimestamp(info_ts));
                                        submessages
                                            .borrow_mut()
                                            .push(RtpsSubmessageType::Data(data));
                                    },
                                    |gap| {
                                        submessages.borrow_mut().push(RtpsSubmessageType::Gap(gap));
                                    },
                                );

                                let submessages = submessages.take();

                                if !submessages.is_empty() {
                                    destined_submessages.push((unicast_locator_list, submessages));
                                }
                            }
                        }
                    }

                    for (locator_list, submessages) in destined_submessages {
                        let message = RtpsMessage {
                            header: message_header.clone(),
                            submessages,
                        };

                        for locator in locator_list {
                            self.transport.write(&message, locator);
                        }
                    }
                }
            }
        }
    }

    pub fn send_subscriber_message(
        &mut self,
        subscriber: &SubscriberAttributes<RtpsStructureImpl>,
    ) {
        for any_data_reader in subscriber.data_reader_list.write_lock().iter_mut() {
            if let RtpsReader::Stateful(stateful_rtps_reader) =
                any_data_reader.rtps_reader.write_lock().deref_mut()
            {
                let message_header = RtpsMessageHeader {
                    protocol: rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
                    version: PROTOCOLVERSION,
                    vendor_id: VENDOR_ID_S2E,
                    guid_prefix: stateful_rtps_reader.guid().prefix,
                };

                for (writer_proxy, acknacks) in stateful_rtps_reader.produce_acknack_submessages() {
                    let message = RtpsMessage {
                        header: message_header.clone(),
                        submessages: acknacks
                            .into_iter()
                            .map(|acknack| RtpsSubmessageType::AckNack(acknack))
                            .collect(),
                    };

                    for &locator in writer_proxy.unicast_locator_list() {
                        self.transport.write(&message, locator);
                    }
                }
            }
        }
    }
}

impl<T> Communication<T>
where
    T: for<'a> TransportRead<'a,
        Vec<
            RtpsSubmessageType<
                Vec<SequenceNumber>,
                Vec<Parameter<'a>>,
                &'a [u8],
                Vec<Locator>,
                Vec<FragmentNumber>,
            >,
        >,
    >,
{
    pub fn receive(
        &mut self,
        publisher_list: &[DdsShared<PublisherAttributes<RtpsStructureImpl>>],
        subscriber_list: &[DdsShared<SubscriberAttributes<RtpsStructureImpl>>],
    ) {
        while let Some((source_locator, message)) = self.transport.read() {
            MessageReceiver::new().process_message(
                self.guid_prefix,
                publisher_list,
                subscriber_list,
                source_locator,
                &message,
            );
        }
    }
}
