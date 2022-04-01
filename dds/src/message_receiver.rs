use std::ops::DerefMut;

use dds_implementation::{
    dds_impl::{
        data_reader_proxy::RtpsReader, data_writer_proxy::RtpsWriter,
        publisher_proxy::PublisherAttributes, subscriber_proxy::SubscriberAttributes,
    },
    utils::shared_object::DdsShared,
};
use rtps_implementation::{
    rtps_reader_locator_impl::RtpsReaderLocatorOperationsImpl,
    rtps_reader_proxy_impl::RtpsReaderProxyOperationsImpl,
};
use rtps_pim::{
    behavior::{
        stateful_writer_behavior::ReliableStatefulWriterBehavior,
        stateless_writer_behavior::ReliableStatelessWriterBehavior,
        writer::reader_proxy::RtpsReaderProxyAttributes,
    },
    messages::{
        submessages::InfoTimestampSubmessage,
        types::{Time, TIME_INVALID},
    },
    structure::{
        history_cache::RtpsHistoryCacheAttributes,
        types::{
            Guid, GuidPrefix, Locator, ProtocolVersion, ReliabilityKind, VendorId,
            ENTITYID_UNKNOWN, GUIDPREFIX_UNKNOWN, LOCATOR_ADDRESS_INVALID, LOCATOR_PORT_INVALID,
            PROTOCOLVERSION, VENDOR_ID_UNKNOWN,
        },
    },
};
use rtps_udp_psm::messages::overall_structure::{RtpsMessage, RtpsSubmessageType};

use crate::domain_participant_factory::RtpsStructureImpl;

pub struct MessageReceiver {
    source_version: ProtocolVersion,
    source_vendor_id: VendorId,
    source_guid_prefix: GuidPrefix,
    dest_guid_prefix: GuidPrefix,
    unicast_reply_locator_list: Vec<Locator>,
    multicast_reply_locator_list: Vec<Locator>,
    have_timestamp: bool,
    timestamp: Time,
}

impl MessageReceiver {
    pub fn new() -> Self {
        Self {
            source_version: PROTOCOLVERSION,
            source_vendor_id: VENDOR_ID_UNKNOWN,
            source_guid_prefix: GUIDPREFIX_UNKNOWN,
            dest_guid_prefix: GUIDPREFIX_UNKNOWN,
            unicast_reply_locator_list: Vec::new(),
            multicast_reply_locator_list: Vec::new(),
            have_timestamp: false,
            timestamp: TIME_INVALID,
        }
    }

    pub fn process_message(
        &mut self,
        participant_guid_prefix: GuidPrefix,
        publisher_list: &[DdsShared<PublisherAttributes<RtpsStructureImpl>>],
        subscriber_list: &[DdsShared<SubscriberAttributes<RtpsStructureImpl>>],
        source_locator: Locator,
        message: &RtpsMessage<'_>,
    ) {
        self.dest_guid_prefix = participant_guid_prefix;
        self.source_version = message.header.version;
        self.source_vendor_id = message.header.vendor_id;
        self.source_guid_prefix = message.header.guid_prefix;
        self.unicast_reply_locator_list.push(Locator::new(
            *source_locator.kind(),
            LOCATOR_PORT_INVALID,
            *source_locator.address(),
        ));
        self.multicast_reply_locator_list.push(Locator::new(
            *source_locator.kind(),
            LOCATOR_PORT_INVALID,
            LOCATOR_ADDRESS_INVALID,
        ));

        for submessage in &message.submessages {
            match submessage {
                RtpsSubmessageType::AckNack(acknack) => {
                    for publisher in publisher_list {
                        for data_writer in publisher.data_writer_list.read_lock().iter() {
                            match data_writer.rtps_writer.write_lock().deref_mut() {
                                RtpsWriter::Stateless(stateless_rtps_writer) => {
                                    if stateless_rtps_writer.writer.endpoint.reliability_level
                                        == ReliabilityKind::Reliable
                                    {
                                        if acknack.reader_id.value == ENTITYID_UNKNOWN
                                            || acknack.reader_id.value
                                                == stateless_rtps_writer
                                                    .writer
                                                    .endpoint
                                                    .entity
                                                    .guid
                                                    .entity_id()
                                        {
                                            for reader_locator in
                                                stateless_rtps_writer.reader_locators.iter_mut()
                                            {
                                                if reader_locator.last_received_acknack_count
                                                    != acknack.count.value
                                                {
                                                    ReliableStatelessWriterBehavior::receive_acknack(
                                                        &mut RtpsReaderLocatorOperationsImpl::new(
                                                            reader_locator,
                                                            &stateless_rtps_writer.writer.writer_cache,
                                                        ),
                                                        acknack,
                                                    );

                                                    reader_locator.last_received_acknack_count =
                                                        acknack.count.value;
                                                }
                                            }
                                        }
                                    }
                                }
                                RtpsWriter::Stateful(stateful_rtps_writer) => {
                                    if stateful_rtps_writer.writer.endpoint.reliability_level
                                        == ReliabilityKind::Reliable
                                    {
                                        let reader_guid = Guid::new(
                                            self.source_guid_prefix,
                                            acknack.reader_id.value,
                                        );

                                        println!("reader guid = {:?}", reader_guid);
                                        for proxy in stateful_rtps_writer.matched_readers.iter() {
                                            println!("remote reader guid: {:?}", proxy.remote_reader_guid());
                                        }
                                        println!("---");

                                        if let Some(reader_proxy) = stateful_rtps_writer
                                            .matched_readers
                                            .iter_mut()
                                            .find(|x| x.remote_reader_guid() == reader_guid)
                                        {
                                            if reader_proxy.last_received_acknack_count
                                                != acknack.count.value
                                            {
                                                ReliableStatefulWriterBehavior::receive_acknack(
                                                    &mut RtpsReaderProxyOperationsImpl::new(
                                                        reader_proxy,
                                                        &stateful_rtps_writer.writer.writer_cache,
                                                    ),
                                                    acknack,
                                                );

                                                reader_proxy.last_received_acknack_count =
                                                    acknack.count.value;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                RtpsSubmessageType::Data(data) => {
                    for subscriber in subscriber_list {
                        for data_reader in subscriber.data_reader_list.read_lock().iter() {
                            let before_data_cache_len;
                            let after_data_cache_len;
                            let mut rtps_reader = data_reader.rtps_reader.write_lock();
                            match rtps_reader.deref_mut() {
                                RtpsReader::Stateless(stateless_rtps_reader) => {
                                    before_data_cache_len =
                                        stateless_rtps_reader.0.reader_cache.changes().len();

                                    stateless_rtps_reader
                                        .process_submessage(data, self.source_guid_prefix);

                                    after_data_cache_len =
                                        stateless_rtps_reader.0.reader_cache.changes().len();
                                }
                                RtpsReader::Stateful(stateful_rtps_reader) => {
                                    before_data_cache_len =
                                        stateful_rtps_reader.reader.reader_cache.changes().len();

                                    stateful_rtps_reader
                                        .process_data_submessage(data, self.source_guid_prefix);

                                    after_data_cache_len =
                                        stateful_rtps_reader.reader.reader_cache.changes().len();
                                }
                            }
                            // Call the listener after dropping the rtps_reader lock to avoid deadlock
                            drop(rtps_reader);
                            if before_data_cache_len < after_data_cache_len {
                                data_reader
                                    .listener
                                    .read_lock()
                                    .as_ref()
                                    .map(|l| l.trigger_on_data_available(data_reader.clone()));
                            }
                        }
                    }
                }
                RtpsSubmessageType::DataFrag(_) => todo!(),
                RtpsSubmessageType::Gap(_) => todo!(),
                RtpsSubmessageType::Heartbeat(heartbeat) => {
                    for subscriber in subscriber_list {
                        for data_reader in subscriber.data_reader_list.read_lock().iter() {
                            let mut rtps_reader = data_reader.rtps_reader.write_lock();
                            if let RtpsReader::Stateful(stateful_rtps_reader) =
                                rtps_reader.deref_mut()
                            {
                                stateful_rtps_reader.process_heartbeat_submessage(
                                    heartbeat,
                                    self.source_guid_prefix,
                                );
                            }
                        }
                    }
                }
                RtpsSubmessageType::HeartbeatFrag(_) => todo!(),
                RtpsSubmessageType::InfoDestination(_) => todo!(),
                RtpsSubmessageType::InfoReply(_) => todo!(),
                RtpsSubmessageType::InfoSource(_) => todo!(),
                RtpsSubmessageType::InfoTimestamp(info_timestamp) => {
                    self.process_info_timestamp_submessage(info_timestamp)
                }
                RtpsSubmessageType::NackFrag(_) => todo!(),
                RtpsSubmessageType::Pad(_) => todo!(),
            }
        }
    }

    fn process_info_timestamp_submessage(&mut self, info_timestamp: &InfoTimestampSubmessage) {
        if info_timestamp.invalidate_flag == false {
            self.have_timestamp = true;
            self.timestamp = info_timestamp.timestamp.value;
        } else {
            self.have_timestamp = false;
            self.timestamp = TIME_INVALID;
        }
    }
}

#[cfg(test)]
mod tests {

    use rtps_pim::messages::submessage_elements::TimestampSubmessageElement;

    use super::*;

    #[test]
    fn process_info_timestamp_submessage_valid_time() {
        let mut message_receiver = MessageReceiver::new();
        let info_timestamp = InfoTimestampSubmessage {
            endianness_flag: true,
            invalidate_flag: false,
            timestamp: TimestampSubmessageElement { value: Time(100) },
        };
        message_receiver.process_info_timestamp_submessage(&info_timestamp);

        assert_eq!(message_receiver.have_timestamp, true);
        assert_eq!(message_receiver.timestamp, Time(100));
    }

    #[test]
    fn process_info_timestamp_submessage_invalid_time() {
        let mut message_receiver = MessageReceiver::new();
        let info_timestamp = InfoTimestampSubmessage {
            endianness_flag: true,
            invalidate_flag: true,
            timestamp: TimestampSubmessageElement { value: Time(100) },
        };
        message_receiver.process_info_timestamp_submessage(&info_timestamp);

        assert_eq!(message_receiver.have_timestamp, false);
        assert_eq!(message_receiver.timestamp, TIME_INVALID);
    }
}
