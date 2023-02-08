use crate::{
    implementation::rtps::{
        messages::{
            submessages::{
                AckNackSubmessage, DataFragSubmessage, DataSubmessage, HeartbeatFragSubmessage,
                HeartbeatSubmessage, InfoDestinationSubmessage, InfoTimestampSubmessage,
                NackFragSubmessage,
            },
            RtpsMessage, RtpsSubmessageKind,
        },
        types::{
            GuidPrefix, Locator, ProtocolVersion, VendorId, GUIDPREFIX_UNKNOWN,
            LOCATOR_ADDRESS_INVALID, LOCATOR_PORT_INVALID, PROTOCOLVERSION, VENDOR_ID_UNKNOWN,
        },
    },
    infrastructure::{
        error::DdsResult,
        time::{Time, TIME_INVALID},
    },
};

pub trait PublisherMessageReceiver {
    fn on_acknack_submessage_received(
        &self,
        acknack_submessage: &AckNackSubmessage,
        message_receiver: &MessageReceiver,
    );

    fn on_nack_frag_submessage_received(
        &self,
        nackfrag_submessage: &NackFragSubmessage,
        message_receiver: &MessageReceiver,
    );
}

pub trait SubscriberSubmessageReceiver {
    fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    );

    fn on_heartbeat_frag_submessage_received(
        &self,
        heartbeat_frag_submessage: &HeartbeatFragSubmessage,
        source_guid_prefix: GuidPrefix,
    );

    fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        message_receiver: &MessageReceiver,
    );

    fn on_data_frag_submessage_received(
        &self,
        data_frag_submessage: &DataFragSubmessage<'_>,
        message_receiver: &MessageReceiver,
    );
}

pub struct MessageReceiver {
    source_version: ProtocolVersion,
    source_vendor_id: VendorId,
    source_guid_prefix: GuidPrefix,
    dest_guid_prefix: GuidPrefix,
    unicast_reply_locator_list: Vec<Locator>,
    multicast_reply_locator_list: Vec<Locator>,
    have_timestamp: bool,
    timestamp: Time,
    reception_timestamp: Time,
}

impl MessageReceiver {
    pub fn new(reception_timestamp: Time) -> Self {
        Self {
            source_version: PROTOCOLVERSION,
            source_vendor_id: VENDOR_ID_UNKNOWN,
            source_guid_prefix: GUIDPREFIX_UNKNOWN,
            dest_guid_prefix: GUIDPREFIX_UNKNOWN,
            unicast_reply_locator_list: Vec::new(),
            multicast_reply_locator_list: Vec::new(),
            have_timestamp: false,
            timestamp: TIME_INVALID,
            reception_timestamp,
        }
    }

    pub fn process_message(
        &mut self,
        participant_guid_prefix: GuidPrefix,
        publisher_list: &[impl PublisherMessageReceiver],
        subscriber_list: &[impl SubscriberSubmessageReceiver],
        source_locator: Locator,
        message: &RtpsMessage<'_>,
    ) -> DdsResult<()> {
        self.dest_guid_prefix = participant_guid_prefix;
        self.source_version = message.header().version;
        self.source_vendor_id = message.header().vendor_id;
        self.source_guid_prefix = message.header().guid_prefix;
        self.unicast_reply_locator_list.push(Locator::new(
            source_locator.kind(),
            LOCATOR_PORT_INVALID,
            source_locator.address(),
        ));
        self.multicast_reply_locator_list.push(Locator::new(
            source_locator.kind(),
            LOCATOR_PORT_INVALID,
            LOCATOR_ADDRESS_INVALID,
        ));

        for submessage in message.submessages() {
            match submessage {
                RtpsSubmessageKind::AckNack(acknack_submessage) => {
                    for publisher in publisher_list {
                        publisher.on_acknack_submessage_received(acknack_submessage, self)
                    }
                }
                RtpsSubmessageKind::Data(data_submessage) => {
                    for subscriber in subscriber_list {
                        subscriber.on_data_submessage_received(data_submessage, self)
                    }
                }
                RtpsSubmessageKind::DataFrag(data_frag_submessage) => {
                    for subscriber in subscriber_list {
                        subscriber.on_data_frag_submessage_received(data_frag_submessage, self)
                    }
                }
                RtpsSubmessageKind::Gap(_) => todo!(),
                RtpsSubmessageKind::Heartbeat(heartbeat_submessage) => {
                    for subscriber in subscriber_list {
                        subscriber.on_heartbeat_submessage_received(
                            heartbeat_submessage,
                            self.source_guid_prefix,
                        )
                    }
                }
                RtpsSubmessageKind::HeartbeatFrag(heartbeat_frag) => {
                    for subscriber in subscriber_list {
                        subscriber.on_heartbeat_frag_submessage_received(
                            heartbeat_frag,
                            self.source_guid_prefix,
                        );
                    }
                }
                RtpsSubmessageKind::InfoDestination(info_dst) => {
                    self.process_info_destination_submessage(info_dst)
                }
                RtpsSubmessageKind::InfoReply(_) => todo!(),
                RtpsSubmessageKind::InfoSource(_) => todo!(),
                RtpsSubmessageKind::InfoTimestamp(info_timestamp) => {
                    self.process_info_timestamp_submessage(info_timestamp)
                }
                RtpsSubmessageKind::NackFrag(nack_frag_submessage) => {
                    for publisher in publisher_list {
                        publisher.on_nack_frag_submessage_received(nack_frag_submessage, self)
                    }
                }
                RtpsSubmessageKind::Pad(_) => (),
            }
        }

        Ok(())
    }

    fn process_info_timestamp_submessage(&mut self, info_timestamp: &InfoTimestampSubmessage) {
        if !info_timestamp.invalidate_flag {
            self.have_timestamp = true;
            self.timestamp = Time::new(
                info_timestamp.timestamp.seconds(),
                info_timestamp.timestamp.fraction(),
            );
        } else {
            self.have_timestamp = false;
            self.timestamp = TIME_INVALID;
        }
    }

    fn process_info_destination_submessage(
        &mut self,
        info_destination: &InfoDestinationSubmessage,
    ) {
        self.dest_guid_prefix = info_destination.guid_prefix;
    }

    #[allow(dead_code)]
    pub fn source_version(&self) -> ProtocolVersion {
        self.source_version
    }

    #[allow(dead_code)]
    pub fn source_vendor_id(&self) -> VendorId {
        self.source_vendor_id
    }

    pub fn source_guid_prefix(&self) -> GuidPrefix {
        self.source_guid_prefix
    }

    #[allow(dead_code)]
    pub fn dest_guid_prefix(&self) -> GuidPrefix {
        self.dest_guid_prefix
    }

    #[allow(dead_code)]
    pub fn unicast_reply_locator_list(&self) -> &[Locator] {
        self.unicast_reply_locator_list.as_ref()
    }

    #[allow(dead_code)]
    pub fn multicast_reply_locator_list(&self) -> &[Locator] {
        self.multicast_reply_locator_list.as_ref()
    }

    #[allow(dead_code)]
    pub fn have_timestamp(&self) -> bool {
        self.have_timestamp
    }

    pub fn timestamp(&self) -> Time {
        self.timestamp
    }

    pub fn reception_timestamp(&self) -> Time {
        self.reception_timestamp
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::rtps::messages::submessages::InfoTimestampSubmessage;

    use super::*;

    #[test]
    fn process_info_timestamp_submessage_valid_time() {
        let mut message_receiver = MessageReceiver::new(TIME_INVALID);
        let info_timestamp = InfoTimestampSubmessage {
            endianness_flag: true,
            invalidate_flag: false,
            timestamp: crate::implementation::rtps::messages::types::Time::new(0, 100),
        };
        message_receiver.process_info_timestamp_submessage(&info_timestamp);

        assert_eq!(message_receiver.have_timestamp, true);
        assert_eq!(message_receiver.timestamp, Time::new(0, 100));
    }

    #[test]
    fn process_info_timestamp_submessage_invalid_time() {
        let mut message_receiver = MessageReceiver::new(TIME_INVALID);
        let info_timestamp = InfoTimestampSubmessage {
            endianness_flag: true,
            invalidate_flag: true,
            timestamp: crate::implementation::rtps::messages::types::Time::new(0, 100),
        };
        message_receiver.process_info_timestamp_submessage(&info_timestamp);

        assert_eq!(message_receiver.have_timestamp, false);
        assert_eq!(message_receiver.timestamp, TIME_INVALID);
    }
}
