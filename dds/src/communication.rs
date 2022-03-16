use dds_implementation::{
    dds_impl::{
        data_writer_proxy::RtpsWriter, publisher_proxy::PublisherAttributes,
        subscriber_proxy::SubscriberAttributes,
    },
    utils::shared_object::RtpsShared,
};
use rtps_implementation::{
    rtps_stateful_writer_impl::RtpsStatefulSubmessage,
    rtps_stateless_writer_impl::RtpsStatelessSubmessage,
};
use rtps_pim::{
    behavior::writer::reader_proxy::RtpsReaderProxyAttributes,
    messages::overall_structure::RtpsMessageHeader,
    structure::types::{GuidPrefix, ProtocolVersion, VendorId, PROTOCOLVERSION, VENDOR_ID_S2E},
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
    pub fn send(&mut self, list: &[RtpsShared<PublisherAttributes<RtpsStructureImpl>>]) {
        for publisher in list {
            let mut publisher_lock = publisher.write_lock();

            let message_header = RtpsMessageHeader {
                protocol: rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
                version: PROTOCOLVERSION,
                vendor_id: VENDOR_ID_S2E,
                guid_prefix: publisher_lock.rtps_group.entity.guid.prefix(),
            };

            for any_data_writer in &mut publisher_lock.data_writer_list {
                let mut rtps_writer_lock = any_data_writer.write_lock();
                let rtps_writer = &mut rtps_writer_lock.rtps_writer;

                match rtps_writer {
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
                                        .map(|submessage| match submessage {
                                            RtpsStatelessSubmessage::Data(data) => {
                                                RtpsSubmessageType::Data(data)
                                            }
                                            RtpsStatelessSubmessage::Gap(gap) => {
                                                RtpsSubmessageType::Gap(gap)
                                            }
                                            RtpsStatelessSubmessage::Heartbeat(heartbeat) => {
                                                RtpsSubmessageType::Heartbeat(heartbeat)
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

                        for (reader_proxy, submessages) in
                            stateful_rtps_writer.produce_destined_submessages()
                        {
                            self.transport.write(
                                &RtpsMessage::new(
                                    message_header.clone(),
                                    submessages
                                        .into_iter()
                                        .map(|submessage| match submessage {
                                            RtpsStatefulSubmessage::Data(data) => {
                                                RtpsSubmessageType::Data(data)
                                            }
                                            RtpsStatefulSubmessage::Gap(gap) => {
                                                RtpsSubmessageType::Gap(gap)
                                            }
                                            RtpsStatefulSubmessage::Heartbeat(heartbeat) => {
                                                RtpsSubmessageType::Heartbeat(heartbeat)
                                            }
                                        })
                                        .collect(),
                                ),
                                reader_proxy.unicast_locator_list()[0],
                            );
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
