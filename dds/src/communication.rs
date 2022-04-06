use std::ops::DerefMut;

use dds_implementation::{
    dds_impl::{
        data_reader_proxy::RtpsReader, publisher_proxy::PublisherAttributes,
        subscriber_proxy::SubscriberAttributes,
    },
    utils::shared_object::DdsShared,
};

use rtps_pim::{
    behavior::reader::writer_proxy::RtpsWriterProxyAttributes,
    messages::{
        overall_structure::{RtpsMessage, RtpsMessageHeader, RtpsSubmessageType},
        submessage_elements::Parameter,
        types::FragmentNumber,
    },
    structure::{
        entity::RtpsEntityAttributes,
        types::{
            GuidPrefix, Locator, ProtocolVersion, SequenceNumber, VendorId, PROTOCOLVERSION,
            VENDOR_ID_S2E,
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
            let mut extended_rtps_writer = any_data_writer.extended_rtps_writer.write_lock();
            let writer_destined_submessages = extended_rtps_writer.produce_submessages();
            for (locator, submessages) in writer_destined_submessages {
                self.transport.write(
                    &RtpsMessage {
                        header: message_header.clone(),
                        submessages,
                    },
                    locator,
                );
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
    T: for<'a> TransportRead<
        'a,
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
