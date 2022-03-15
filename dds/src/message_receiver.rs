use dds_implementation::{
    dds_impl::{data_reader_proxy::RtpsReader, subscriber_proxy::SubscriberAttributes},
    utils::shared_object::RtpsShared,
};
use rtps_pim::{
    messages::{
        submessages::{AckNackSubmessage, InfoTimestampSubmessage},
        types::{Time, TIME_INVALID},
    },
    structure::{
        history_cache::RtpsHistoryCacheAttributes,
        types::{
            GuidPrefix, Locator, ProtocolVersion, SequenceNumber, VendorId, GUIDPREFIX_UNKNOWN,
            LOCATOR_ADDRESS_INVALID, LOCATOR_PORT_INVALID, PROTOCOLVERSION, VENDOR_ID_UNKNOWN,
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

    pub fn process_message<'a>(
        &mut self,
        participant_guid_prefix: GuidPrefix,
        list: &'a [RtpsShared<SubscriberAttributes<RtpsStructureImpl>>],
        source_locator: Locator,
        message: &'a RtpsMessage<'a>,
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
                RtpsSubmessageType::AckNack(_) => todo!(),
                RtpsSubmessageType::Data(data) => {
                    for subscriber in list {
                        let subscriber_lock = subscriber.read_lock();
                        for data_reader in &subscriber_lock.data_reader_list {
                            let mut data_reader_lock = data_reader.write_lock();
                            let rtps_reader = &mut data_reader_lock.rtps_reader;
                            match rtps_reader {
                                RtpsReader::Stateless(stateless_rtps_reader) => {
                                    let cache_len =
                                        stateless_rtps_reader.reader_cache.changes().len();

                                    stateless_rtps_reader
                                        .process_submessage(data, self.source_guid_prefix);

                                    if stateless_rtps_reader.reader_cache.changes().len()
                                        > cache_len
                                    {
                                        data_reader_lock.listener.as_ref().map(|l| {
                                            l.trigger_on_data_available(data_reader.clone())
                                        });
                                    }
                                }
                                RtpsReader::Stateful(stateful_rtps_reader) => {
                                    let cache_len =
                                        stateful_rtps_reader.reader.reader_cache.changes().len();

                                    stateful_rtps_reader
                                        .process_submessage(data, self.source_guid_prefix);

                                    if stateful_rtps_reader.reader.reader_cache.changes().len()
                                        > cache_len
                                    {
                                        data_reader_lock.listener.as_ref().map(|l| {
                                            l.trigger_on_data_available(data_reader.clone())
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                RtpsSubmessageType::DataFrag(_) => todo!(),
                RtpsSubmessageType::Gap(_) => todo!(),
                RtpsSubmessageType::Heartbeat(_) => (),
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

pub trait ProcessAckNackSubmessage {
    fn process_acknack_submessage(
        &self,
        source_guid_prefix: GuidPrefix,
        _acknack: &AckNackSubmessage<Vec<SequenceNumber>>,
    );
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
