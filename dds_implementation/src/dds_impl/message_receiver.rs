use crate::{
    dds_impl::{
        publisher_attributes::PublisherAttributes, subscriber_attributes::SubscriberAttributes,
    },
    utils::{
        rtps_communication_traits::{
            ReceiveRtpsAckNackSubmessage, ReceiveRtpsDataSubmessage, ReceiveRtpsHeartbeatSubmessage,
        },
        shared_object::DdsShared,
    },
};
use rtps_pim::{
    messages::{
        overall_structure::{RtpsMessage, RtpsSubmessageType},
        submessage_elements::Parameter,
        submessages::InfoTimestampSubmessage,
        types::{FragmentNumber, Time, TIME_INVALID},
    },
    structure::types::{
        GuidPrefix, Locator, ProtocolVersion, SequenceNumber, VendorId, GUIDPREFIX_UNKNOWN,
        LOCATOR_ADDRESS_INVALID, LOCATOR_PORT_INVALID, PROTOCOLVERSION, VENDOR_ID_UNKNOWN,
    },
};

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
        publisher_list: &[DdsShared<PublisherAttributes>],
        subscriber_list: &[DdsShared<SubscriberAttributes>],
        source_locator: Locator,
        message: &RtpsMessage<
            Vec<
                RtpsSubmessageType<
                    Vec<SequenceNumber>,
                    Vec<Parameter<'_>>,
                    &'_ [u8],
                    Vec<Locator>,
                    Vec<FragmentNumber>,
                >,
            >,
        >,
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
                RtpsSubmessageType::AckNack(acknack_submessage) => {
                    for publisher in publisher_list {
                        publisher.on_acknack_submessage_received(
                            acknack_submessage,
                            self.source_guid_prefix,
                        )
                    }
                }
                RtpsSubmessageType::Data(data_submessage) => {
                    for subscriber in subscriber_list {
                        subscriber
                            .on_data_submessage_received(data_submessage, self.source_guid_prefix)
                    }
                }
                RtpsSubmessageType::DataFrag(_) => todo!(),
                RtpsSubmessageType::Gap(_) => todo!(),
                RtpsSubmessageType::Heartbeat(heartbeat_submessage) => {
                    for subscriber in subscriber_list {
                        subscriber.on_heartbeat_submessage_received(
                            heartbeat_submessage,
                            self.source_guid_prefix,
                        )
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
