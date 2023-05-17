use crate::{
    implementation::rtps::{
        messages::{
            submessages::{
                InfoDestinationSubmessage, InfoSourceSubmessage, InfoTimestampSubmessage,
            },
            RtpsMessage, RtpsSubmessageKind,
        },
        types::{
            Guid, GuidPrefix, Locator, ProtocolVersion, VendorId, GUIDPREFIX_UNKNOWN,
            LOCATOR_ADDRESS_INVALID, LOCATOR_PORT_INVALID, PROTOCOLVERSION, VENDOR_ID_UNKNOWN,
        },
    },
    infrastructure::{
        error::DdsResult,
        time::{Time, TIME_INVALID},
    },
};

use super::{
    dds_publisher::DdsPublisher, dds_subscriber::DdsSubscriber,
    status_listener::ListenerTriggerKind,
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
        participant_guid: Guid,
        publisher_list: &mut [DdsPublisher],
        subscriber_list: &mut [DdsSubscriber],
        source_locator: Locator,
        message: &RtpsMessage<'_>,
        listener_sender: &tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) -> DdsResult<()> {
        self.dest_guid_prefix = participant_guid.prefix();
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
                    for publisher in publisher_list.iter_mut() {
                        for stateful_data_writer in
                            publisher.stateful_data_writer_list_mut().iter_mut()
                        {
                            stateful_data_writer
                                .on_acknack_submessage_received(acknack_submessage, self);
                        }
                    }
                }
                RtpsSubmessageKind::Data(data_submessage) => {
                    for subscriber in subscriber_list.iter_mut() {
                        subscriber.on_data_submessage_received(
                            data_submessage,
                            self,
                            participant_guid,
                            listener_sender,
                        )
                    }
                }
                RtpsSubmessageKind::DataFrag(data_frag_submessage) => {
                    for subscriber in subscriber_list.iter_mut() {
                        subscriber.on_data_frag_submessage_received(
                            data_frag_submessage,
                            self,
                            participant_guid,
                            listener_sender,
                        )
                    }
                }
                RtpsSubmessageKind::Gap(gap_submessage) => {
                    for subscriber in subscriber_list.iter_mut() {
                        subscriber.on_gap_submessage_received(gap_submessage, self)
                    }
                }
                RtpsSubmessageKind::Heartbeat(heartbeat_submessage) => {
                    for subscriber in subscriber_list.iter_mut() {
                        subscriber.on_heartbeat_submessage_received(
                            heartbeat_submessage,
                            self.source_guid_prefix,
                        )
                    }
                }
                RtpsSubmessageKind::HeartbeatFrag(heartbeat_frag) => {
                    for subscriber in subscriber_list.iter_mut() {
                        subscriber.on_heartbeat_frag_submessage_received(
                            heartbeat_frag,
                            self.source_guid_prefix,
                        );
                    }
                }
                RtpsSubmessageKind::InfoDestination(info_dst) => {
                    self.process_info_destination_submessage(info_dst)
                }
                RtpsSubmessageKind::InfoReply(_) => (),
                RtpsSubmessageKind::InfoSource(info_source) => {
                    self.process_info_source_submessage(info_source)
                }
                RtpsSubmessageKind::InfoTimestamp(info_timestamp) => {
                    self.process_info_timestamp_submessage(info_timestamp)
                }
                RtpsSubmessageKind::NackFrag(nack_frag_submessage) => {
                    for publisher in publisher_list.iter_mut() {
                        for stateful_data_writer in
                            publisher.stateful_data_writer_list_mut().iter_mut()
                        {
                            stateful_data_writer
                                .on_nack_frag_submessage_received(nack_frag_submessage, self);
                        }
                    }
                }
                RtpsSubmessageKind::Pad(_) => (),
            }
        }

        Ok(())
    }

    fn process_info_source_submessage(&mut self, info_source: &InfoSourceSubmessage) {
        self.source_vendor_id = info_source.vendor_id;
        self.source_version = info_source.protocol_version;
        self.source_vendor_id = info_source.vendor_id;
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

    pub fn source_guid_prefix(&self) -> GuidPrefix {
        self.source_guid_prefix
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

        assert!(message_receiver.have_timestamp);
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

        assert!(!message_receiver.have_timestamp);
        assert_eq!(message_receiver.timestamp, TIME_INVALID);
    }
}
