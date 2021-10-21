use crate::utils::shared_object::rtps_shared_write_lock;

use super::{shared_object::RtpsShared, transport::RtpsMessageRead};
use rust_rtps_pim::{
    messages::{
        submessage_elements::Parameter,
        submessages::{DataSubmessage, InfoTimestampSubmessage, RtpsSubmessageType},
        types::{Time, TIME_INVALID},
    },
    structure::types::{
        GuidPrefix, Locator, ProtocolVersion, VendorId, GUIDPREFIX_UNKNOWN,
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

    pub fn process_message<'a>(
        mut self,
        participant_guid_prefix: GuidPrefix,
        list: &'a [RtpsShared<impl ProcessDataSubmessage>],
        source_locator: Locator,
        message: &'a RtpsMessageRead,
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
                    for element in list {
                        rtps_shared_write_lock(&element)
                            .process_data_submessage(self.source_guid_prefix, data)
                    }
                }
                RtpsSubmessageType::DataFrag(_) => todo!(),
                RtpsSubmessageType::Gap(_) => todo!(),
                RtpsSubmessageType::Heartbeat(_) => todo!(),
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

pub trait ProcessDataSubmessage {
    fn process_data_submessage(
        &self,
        source_guid_prefix: GuidPrefix,
        _data: &DataSubmessage<Vec<Parameter<&'_ [u8]>>, &[u8]>,
    );
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use rust_rtps_pim::{
        messages::{
            submessage_elements::{
                EntityIdSubmessageElement, ParameterListSubmessageElement,
                SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
                TimestampSubmessageElement,
            },
            types::ProtocolId,
            RtpsMessage, RtpsMessageHeader,
        },
        structure::types::{
            EntityId, BUILT_IN_READER_WITH_KEY, BUILT_IN_WRITER_WITH_KEY, PROTOCOLVERSION_2_4,
        },
    };

    use crate::utils::shared_object::{rtps_shared_new, rtps_shared_read_lock};

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

    #[test]
    fn process_data() {
        struct MockProcessDataSubmessage {
            called: RefCell<bool>,
        }

        impl ProcessDataSubmessage for MockProcessDataSubmessage {
            fn process_data_submessage(
                &self,
                _source_guid_prefix: GuidPrefix,
                _data: &DataSubmessage<Vec<Parameter<&'_ [u8]>>, &[u8]>,
            ) {
                *self.called.borrow_mut() = true
            }
        }

        let data_submessage = DataSubmessage {
            endianness_flag: true,
            inline_qos_flag: false,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: EntityId::new([1; 3], BUILT_IN_READER_WITH_KEY),
            },
            writer_id: EntityIdSubmessageElement {
                value: EntityId::new([1; 3], BUILT_IN_WRITER_WITH_KEY),
            },
            writer_sn: SequenceNumberSubmessageElement { value: 1 },
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement {
                value: &[1, 2, 3][..],
            },
        };
        let participant_guid_prefix = GuidPrefix([1; 12]);
        let reader_group_list = vec![rtps_shared_new(MockProcessDataSubmessage {
            called: RefCell::new(false),
        })];
        let source_locator = Locator::new(1, 7400, [1; 16]);
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: PROTOCOLVERSION_2_4,
            vendor_id: [99, 99],
            guid_prefix: GuidPrefix([1; 12]),
        };
        let submessages: Vec<
            rust_rtps_pim::messages::submessages::RtpsSubmessageType<
                Vec<i64>,
                Vec<Parameter<&'_ [u8]>>,
                &[u8],
                (),
                (),
            >,
        > = vec![RtpsSubmessageType::Data(data_submessage)];
        let message = RtpsMessage {
            header,
            submessages,
        };

        MessageReceiver::new().process_message(
            participant_guid_prefix,
            &reader_group_list,
            source_locator,
            &message,
        );

        assert_eq!(
            *rtps_shared_read_lock(&reader_group_list[0]).called.borrow(),
            true
        );
    }
}
