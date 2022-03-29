use std::ops::DerefMut;

use dds_implementation::{
    dds_impl::{
        data_reader_proxy::RtpsReader, data_writer_proxy::RtpsWriter,
        publisher_proxy::PublisherAttributes, subscriber_proxy::SubscriberAttributes,
    },
    utils::shared_object::DdsShared,
};
use rtps_implementation::{
    rtps_stateful_writer_impl::RtpsStatefulSubmessage,
    rtps_stateless_writer_impl::RtpsStatelessSubmessage,
};
use rtps_pim::{
    behavior::reader::writer_proxy::RtpsWriterProxyAttributes,
    messages::{
        overall_structure::RtpsMessageHeader, submessage_elements::TimestampSubmessageElement,
        submessages::InfoTimestampSubmessage, types::TIME_INVALID,
    },
    structure::{
        entity::RtpsEntityAttributes,
        types::{GuidPrefix, ProtocolVersion, VendorId, PROTOCOLVERSION, VENDOR_ID_S2E},
    },
};
use rtps_udp_psm::messages::overall_structure::{RtpsMessage, RtpsSubmessageType};

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

                    for (locator, submessages) in
                        stateless_rtps_writer.produce_destined_submessages()
                    {
                        self.transport.write(
                            &RtpsMessage::new(
                                message_header.clone(),
                                submessages
                                    .into_iter()
                                    .flat_map(|submessage| match submessage {
                                        RtpsStatelessSubmessage::Data(data) => {
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
                                            vec![
                                                RtpsSubmessageType::InfoTimestamp(info_ts),
                                                RtpsSubmessageType::Data(data),
                                            ]
                                        }
                                        RtpsStatelessSubmessage::Gap(gap) => {
                                            vec![RtpsSubmessageType::Gap(gap)]
                                        }
                                        RtpsStatelessSubmessage::Heartbeat(heartbeat) => {
                                            vec![RtpsSubmessageType::Heartbeat(heartbeat)]
                                        }
                                    })
                                    .collect(),
                            ),
                            locator,
                        );
                    }
                }
                RtpsWriter::Stateful(stateful_rtps_writer) => {
                    let message_header = RtpsMessageHeader {
                        guid_prefix: stateful_rtps_writer.writer.endpoint.entity.guid.prefix,
                        ..message_header.clone()
                    };

                    for (locator_list, submessages) in
                        stateful_rtps_writer.produce_destined_requested_submessages()
                    {
                        self.transport.write(
                            &RtpsMessage::new(
                                message_header.clone(),
                                submessages
                                    .into_iter()
                                    .flat_map(|submessage| match submessage {
                                        RtpsStatefulSubmessage::Data(data) => {
                                            vec![RtpsSubmessageType::Data(data)]
                                        }
                                        RtpsStatefulSubmessage::Gap(gap) => {
                                            vec![RtpsSubmessageType::Gap(gap)]
                                        }
                                        RtpsStatefulSubmessage::Heartbeat(_) => unimplemented!(),
                                    })
                                    .collect(),
                            ),
                            locator_list[0],
                        )
                    }

                    for (locator_list, submessages) in
                        stateful_rtps_writer.produce_destined_submessages()
                    {
                        self.transport.write(
                            &RtpsMessage::new(
                                message_header.clone(),
                                submessages
                                    .into_iter()
                                    .flat_map(|submessage| match submessage {
                                        RtpsStatefulSubmessage::Data(data) => {
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

                                            vec![
                                                RtpsSubmessageType::InfoTimestamp(info_ts),
                                                RtpsSubmessageType::Data(data),
                                            ]
                                        }
                                        RtpsStatefulSubmessage::Gap(gap) => {
                                            vec![RtpsSubmessageType::Gap(gap)]
                                        }
                                        RtpsStatefulSubmessage::Heartbeat(heartbeat) => {
                                            vec![RtpsSubmessageType::Heartbeat(heartbeat)]
                                        }
                                    })
                                    .collect(),
                            ),
                            locator_list[0],
                        );
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
                    self.transport.write(
                        &RtpsMessage::new(
                            message_header.clone(),
                            acknacks
                                .into_iter()
                                .map(|acknack| RtpsSubmessageType::AckNack(acknack))
                                .collect(),
                        ),
                        writer_proxy.unicast_locator_list()[0],
                    );
                }
            }
        }
    }
}

impl<T> Communication<T>
where
    T: TransportRead,
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
