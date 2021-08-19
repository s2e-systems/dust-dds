use rust_rtps_pim::{
    behavior::{reader::reader::RtpsReader, stateless_reader_behavior::StatelessReaderBehavior},
    messages::{
        submessage_elements::{Parameter, TimestampSubmessageElementType},
        submessages::{
            DataSubmessage, InfoTimestampSubmessage, RtpsSubmessagePIM, RtpsSubmessageType,
        },
        types::{Time, TIME_INVALID},
        RtpsMessage,
    },
    structure::types::{
        GuidPrefix, Locator, ProtocolVersion, VendorId, GUIDPREFIX_UNKNOWN,
        LOCATOR_ADDRESS_INVALID, LOCATOR_PORT_INVALID, PROTOCOLVERSION, VENDOR_ID_UNKNOWN,
    },
};

use crate::dds_impl::subscriber_impl::SubscriberStorage;

use super::shared_object::RtpsShared;

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

    pub fn process_message<'a, PSM, Message>(
        mut self,
        participant_guid_prefix: GuidPrefix,
        reader_group_list: &'a [RtpsShared<SubscriberStorage>],
        source_locator: Locator,
        message: &'a Message,
    ) where
        Message: RtpsMessage<SubmessageType = RtpsSubmessageType<'a, PSM>> + 'a,
        PSM: RtpsSubmessagePIM<'a, DataSubmessageType = DataSubmessage<'a, &'a [Parameter<'a>]>>
            + 'a,
        PSM::InfoTimestampSubmessageType: InfoTimestampSubmessage,
    {
        self.dest_guid_prefix = participant_guid_prefix;
        self.source_version = message.header().version;
        self.source_vendor_id = message.header().vendor_id;
        self.source_guid_prefix = message.header().guid_prefix;
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

        for submessage in message.submessages() {
            match submessage {
                RtpsSubmessageType::AckNack(_) => todo!(),
                RtpsSubmessageType::Data(data) => self.process_data(data, reader_group_list),
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

    fn process_data<'a>(
        &mut self,
        data: &'a DataSubmessage<&'a [Parameter<'a>]>,
        reader_group_list: &'a [RtpsShared<SubscriberStorage>],
    ) {
        for subscriber in reader_group_list {
            let subscriber_lock = subscriber.lock();
            for reader in subscriber_lock.readers() {
                let mut reader_lock = reader.lock();
                reader_lock
                    .rtps_reader_mut()
                    .reader_cache_mut()
                    .set_source_timestamp(Some(self.timestamp));
                reader_lock
                    .rtps_reader_mut()
                    .receive_data(self.source_guid_prefix, data);
            }
        }
    }

    fn process_info_timestamp_submessage<InfoTimestamp>(&mut self, info_timestamp: &InfoTimestamp)
    where
        InfoTimestamp: InfoTimestampSubmessage,
    {
        if info_timestamp.invalidate_flag() == false {
            self.have_timestamp = true;
            self.timestamp = info_timestamp.timestamp().value();
        } else {
            self.have_timestamp = false;
            self.timestamp = TIME_INVALID;
        }
    }
}

#[cfg(test)]
mod tests {
    // use rust_rtps_pim::{
    //     messages::{
    //         submessage_elements::{
    //             EntityIdSubmessageElementType, Parameter, ParameterListSubmessageElementType,
    //             SequenceNumberSubmessageElementType, SerializedDataSubmessageElementType,
    //         },
    //         types::SubmessageFlag,
    //     },
    //     structure::types::{EntityId, SequenceNumber},
    // };

    // use super::*;

    // struct MockEntityIdSubmessageElement(EntityId);

    // impl EntityIdSubmessageElementType for MockEntityIdSubmessageElement {
    //     fn new(_value: &EntityId) -> Self {
    //         todo!()
    //     }

    //     fn value(&self) -> EntityId {
    //         todo!()
    //     }
    // }

    // struct MockSequenceNumberSubmessageElement(SequenceNumber);

    // impl SequenceNumberSubmessageElementType for MockSequenceNumberSubmessageElement {
    //     fn new(_value: &SequenceNumber) -> Self {
    //         todo!()
    //     }

    //     fn value(&self) -> SequenceNumber {
    //         todo!()
    //     }
    // }

    // struct MockParameterListSubmessageElement;

    // impl<'a> ParameterListSubmessageElementType<'a> for MockParameterListSubmessageElement {
    //     fn new(_parameter: &[Parameter]) -> Self {
    //         todo!()
    //     }

    //     fn parameter(&self) -> &[Parameter<'a>] {
    //         todo!()
    //     }
    // }

    // struct MockSerializedDataSubmessageElement;

    // impl<'a> SerializedDataSubmessageElementType<'a> for MockSerializedDataSubmessageElement {
    //     fn new(_value: &'a [u8]) -> Self {
    //         todo!()
    //     }

    //     fn value(&self) -> &'a [u8] {
    //         todo!()
    //     }
    // }

    // struct MockTimestampSubmessageElement(Time);

    // impl TimestampSubmessageElementType for MockTimestampSubmessageElement {
    //     fn new(_value: &Time) -> Self {
    //         todo!()
    //     }

    //     fn value(&self) -> Time {
    //         self.0.clone()
    //     }
    // }

    // struct MockDataSubmessage;

    // impl<'a> DataSubmessage<'a> for MockDataSubmessage {
    //     type EntityIdSubmessageElementType = MockEntityIdSubmessageElement;
    //     type SequenceNumberSubmessageElementType = MockSequenceNumberSubmessageElement;
    //     type ParameterListSubmessageElementType = MockParameterListSubmessageElement;
    //     type SerializedDataSubmessageElementType = MockSerializedDataSubmessageElement;

    //     fn new(
    //         _endianness_flag: SubmessageFlag,
    //         _inline_qos_flag: SubmessageFlag,
    //         _data_flag: SubmessageFlag,
    //         _key_flag: SubmessageFlag,
    //         _non_standard_payload_flag: SubmessageFlag,
    //         _reader_id: Self::EntityIdSubmessageElementType,
    //         _writer_id: Self::EntityIdSubmessageElementType,
    //         _writer_sn: Self::SequenceNumberSubmessageElementType,
    //         _inline_qos: Self::ParameterListSubmessageElementType,
    //         _serialized_payload: Self::SerializedDataSubmessageElementType,
    //     ) -> Self {
    //         todo!()
    //     }

    //     fn endianness_flag(&self) -> SubmessageFlag {
    //         todo!()
    //     }

    //     fn inline_qos_flag(&self) -> SubmessageFlag {
    //         todo!()
    //     }

    //     fn data_flag(&self) -> SubmessageFlag {
    //         todo!()
    //     }

    //     fn key_flag(&self) -> SubmessageFlag {
    //         todo!()
    //     }

    //     fn non_standard_payload_flag(&self) -> SubmessageFlag {
    //         todo!()
    //     }

    //     fn reader_id(&self) -> &Self::EntityIdSubmessageElementType {
    //         todo!()
    //     }

    //     fn writer_id(&self) -> &Self::EntityIdSubmessageElementType {
    //         todo!()
    //     }

    //     fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType {
    //         todo!()
    //     }

    //     fn inline_qos(&self) -> &Self::ParameterListSubmessageElementType {
    //         todo!()
    //     }

    //     fn serialized_payload(&self) -> &Self::SerializedDataSubmessageElementType {
    //         todo!()
    //     }
    // }

    // struct MockInfoTimestampSubmessage {
    //     invalidate_flag: SubmessageFlag,
    //     timestamp: MockTimestampSubmessageElement,
    // }

    // impl InfoTimestampSubmessage for MockInfoTimestampSubmessage {
    //     type TimestampSubmessageElementType = MockTimestampSubmessageElement;

    //     fn new(
    //         _endianness_flag: SubmessageFlag,
    //         _invalidate_flag: SubmessageFlag,
    //         _timestamp: Self::TimestampSubmessageElementType,
    //     ) -> Self {
    //         todo!()
    //     }

    //     fn endianness_flag(&self) -> SubmessageFlag {
    //         todo!()
    //     }

    //     fn invalidate_flag(&self) -> SubmessageFlag {
    //         self.invalidate_flag
    //     }

    //     fn timestamp(&self) -> &Self::TimestampSubmessageElementType {
    //         &self.timestamp
    //     }
    // }

    // #[test]
    // fn process_info_timestamp_submessage_valid_time() {
    //     let mut message_receiver = MessageReceiver::new();
    //     let info_timestamp = MockInfoTimestampSubmessage {
    //         invalidate_flag: false,
    //         timestamp: MockTimestampSubmessageElement(Time(100)),
    //     };
    //     message_receiver.process_info_timestamp_submessage(&info_timestamp);

    //     assert_eq!(message_receiver.have_timestamp, true);
    //     assert_eq!(message_receiver.timestamp, Time(100));
    // }

    // #[test]
    // fn process_info_timestamp_submessage_invalid_time() {
    //     let mut message_receiver = MessageReceiver::new();
    //     let info_timestamp = MockInfoTimestampSubmessage {
    //         invalidate_flag: true,
    //         timestamp: MockTimestampSubmessageElement(Time(100)),
    //     };
    //     message_receiver.process_info_timestamp_submessage(&info_timestamp);

    //     assert_eq!(message_receiver.have_timestamp, false);
    //     assert_eq!(message_receiver.timestamp, TIME_INVALID);
    // }

    // #[test]
    // fn process_data() {
    //     struct MockReaderGroup(Option<MockReader>);

    //     impl<'a> RTPSGroup for &'a mut MockReaderGroup {
    //         type Endpoints = core::option::IterMut<'a, MockReader>;

    //         fn endpoints(self) -> Self::Endpoints {
    //             self.0.iter_mut()
    //         }
    //     }

    //     struct MockReader(bool);

    //     impl<'a> StatelessReaderBehavior<MockDataSubmessage> for MockReader {
    //         fn receive_data(
    //             &mut self,
    //             _source_guid_prefix: GuidPrefix,
    //             _data: &MockDataSubmessage,
    //         ) {
    //             self.0 = true;
    //         }
    //     }

    //     let mut message_receiver = MessageReceiver::new();
    //     let data = MockDataSubmessage;
    //     let reader = MockReader(false);
    //     let mut reader_groups = MockReaderGroup(Some(reader));
    //     message_receiver.process_data(&data, &mut reader_groups);
    //     assert_eq!(reader_groups.0.unwrap().0, true);
    // }
}
