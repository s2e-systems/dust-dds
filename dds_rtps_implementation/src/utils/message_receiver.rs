use rust_rtps_pim::{
    behavior::stateless_reader_behavior::StatelessReaderBehavior,
    messages::{
        submessage_elements::TimestampSubmessageElementType,
        submessages::{
            DataSubmessage, InfoTimestampSubmessage, RtpsSubmessagePIM, RtpsSubmessageType,
        },
        types::{Time, TIME_INVALID},
        RTPSMessage,
    },
    structure::{
        types::{
            GuidPrefix, Locator, ProtocolVersion, VendorId, GUIDPREFIX_UNKNOWN,
            LOCATOR_ADDRESS_INVALID, LOCATOR_PORT_INVALID, PROTOCOLVERSION, VENDOR_ID_UNKNOWN,
        },
        RTPSEntity, RTPSGroup, ReaderGroupCollection,
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

    pub fn process_message<'a, PSM, Message, Participant, Group, Reader>(
        mut self,
        participant: Participant,
        source_locator: Locator,
        message: Message,
    ) where
        Participant: RTPSEntity + ReaderGroupCollection + Copy,
        Participant::ReaderGroupsType: IntoIterator<Item = Group>,
        Group: RTPSGroup + 'a,
        Group::Endpoints: IntoIterator<Item = Reader>,
        Reader: StatelessReaderBehavior<PSM::DataSubmessageType> + 'a,
        PSM: RtpsSubmessagePIM<'a>,
        Message: RTPSMessage<SubmessageType = RtpsSubmessageType<'a, PSM>>,
        PSM::DataSubmessageType: DataSubmessage<'a>,
        PSM::InfoTimestampSubmessageType: InfoTimestampSubmessage,
    {
        self.dest_guid_prefix = *participant.guid().prefix();
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
                RtpsSubmessageType::Data(data) => self.process_data(data, participant),
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

    fn process_data<'a, Data, Participant, Group, Reader>(
        &mut self,
        data: &Data,
        participant: Participant,
    ) where
        Data: DataSubmessage<'a>,
        Participant: ReaderGroupCollection,
        Participant::ReaderGroupsType: IntoIterator<Item = Group>,
        Group: RTPSGroup,
        Group::Endpoints: IntoIterator<Item = Reader>,
        Reader: StatelessReaderBehavior<Data>,
    {
        for reader_group in participant.reader_groups() {
            for mut reader in reader_group.endpoints() {
                reader.receive_data(self.source_guid_prefix, data);
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
    use rust_rtps_pim::{
        messages::{
            submessage_elements::{
                EntityIdSubmessageElementType, Parameter, ParameterListSubmessageElementType,
                SequenceNumberSubmessageElementType, SerializedDataSubmessageElementType,
            },
            types::SubmessageFlag,
        },
        structure::types::{EntityId, SequenceNumber},
    };

    use super::*;

    struct MockEntityIdSubmessageElement(EntityId);

    impl EntityIdSubmessageElementType for MockEntityIdSubmessageElement {
        fn new(_value: &EntityId) -> Self {
            todo!()
        }

        fn value(&self) -> EntityId {
            todo!()
        }
    }

    struct MockSequenceNumberSubmessageElement(SequenceNumber);

    impl SequenceNumberSubmessageElementType for MockSequenceNumberSubmessageElement {
        fn new(_value: &SequenceNumber) -> Self {
            todo!()
        }

        fn value(&self) -> SequenceNumber {
            todo!()
        }
    }

    struct MockParameterListSubmessageElement;

    impl<'a> ParameterListSubmessageElementType<'a> for MockParameterListSubmessageElement {
        type IntoIter = Option<Parameter<'a>>;

        fn new(_parameter: &[Parameter]) -> Self {
            todo!()
        }

        fn parameter(&'a self) -> Self::IntoIter {
            todo!()
        }
    }

    struct MockSerializedDataSubmessageElement;

    impl<'a> SerializedDataSubmessageElementType<'a> for MockSerializedDataSubmessageElement {
        type Value = &'a [u8];

        fn new(_value: &Self::Value) -> Self {
            todo!()
        }

        fn value(&self) -> Self::Value {
            todo!()
        }
    }

    struct MockTimestampSubmessageElement(Time);

    impl TimestampSubmessageElementType for MockTimestampSubmessageElement {
        fn new(_value: &Time) -> Self {
            todo!()
        }

        fn value(&self) -> Time {
            self.0.clone()
        }
    }

    struct MockDataSubmessage;

    impl<'a> DataSubmessage<'a> for MockDataSubmessage {
        type EntityIdSubmessageElementType = MockEntityIdSubmessageElement;
        type SequenceNumberSubmessageElementType = MockSequenceNumberSubmessageElement;
        type ParameterListSubmessageElementType = MockParameterListSubmessageElement;
        type SerializedDataSubmessageElementType = MockSerializedDataSubmessageElement;

        fn new(
            _endianness_flag: SubmessageFlag,
            _inline_qos_flag: SubmessageFlag,
            _data_flag: SubmessageFlag,
            _key_flag: SubmessageFlag,
            _non_standard_payload_flag: SubmessageFlag,
            _reader_id: Self::EntityIdSubmessageElementType,
            _writer_id: Self::EntityIdSubmessageElementType,
            _writer_sn: Self::SequenceNumberSubmessageElementType,
            _inline_qos: Self::ParameterListSubmessageElementType,
            _serialized_payload: Self::SerializedDataSubmessageElementType,
        ) -> Self {
            todo!()
        }

        fn endianness_flag(&self) -> SubmessageFlag {
            todo!()
        }

        fn inline_qos_flag(&self) -> SubmessageFlag {
            todo!()
        }

        fn data_flag(&self) -> SubmessageFlag {
            todo!()
        }

        fn key_flag(&self) -> SubmessageFlag {
            todo!()
        }

        fn non_standard_payload_flag(&self) -> SubmessageFlag {
            todo!()
        }

        fn reader_id(&self) -> &Self::EntityIdSubmessageElementType {
            todo!()
        }

        fn writer_id(&self) -> &Self::EntityIdSubmessageElementType {
            todo!()
        }

        fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType {
            todo!()
        }

        fn inline_qos(&self) -> &Self::ParameterListSubmessageElementType {
            todo!()
        }

        fn serialized_payload(&self) -> &Self::SerializedDataSubmessageElementType {
            todo!()
        }
    }

    struct MockInfoTimestampSubmessage {
        invalidate_flag: SubmessageFlag,
        timestamp: MockTimestampSubmessageElement,
    }

    impl InfoTimestampSubmessage for MockInfoTimestampSubmessage {
        type TimestampSubmessageElementType = MockTimestampSubmessageElement;

        fn new(
            _endianness_flag: SubmessageFlag,
            _invalidate_flag: SubmessageFlag,
            _timestamp: Self::TimestampSubmessageElementType,
        ) -> Self {
            todo!()
        }

        fn endianness_flag(&self) -> SubmessageFlag {
            todo!()
        }

        fn invalidate_flag(&self) -> SubmessageFlag {
            self.invalidate_flag
        }

        fn timestamp(&self) -> &Self::TimestampSubmessageElementType {
            &self.timestamp
        }
    }

    #[test]
    fn process_info_timestamp_submessage_valid_time() {
        let mut message_receiver = MessageReceiver::new();
        let info_timestamp = MockInfoTimestampSubmessage {
            invalidate_flag: false,
            timestamp: MockTimestampSubmessageElement(Time(100)),
        };
        message_receiver.process_info_timestamp_submessage(&info_timestamp);

        assert_eq!(message_receiver.have_timestamp, true);
        assert_eq!(message_receiver.timestamp, Time(100));
    }

    #[test]
    fn process_info_timestamp_submessage_invalid_time() {
        let mut message_receiver = MessageReceiver::new();
        let info_timestamp = MockInfoTimestampSubmessage {
            invalidate_flag: true,
            timestamp: MockTimestampSubmessageElement(Time(100)),
        };
        message_receiver.process_info_timestamp_submessage(&info_timestamp);

        assert_eq!(message_receiver.have_timestamp, false);
        assert_eq!(message_receiver.timestamp, TIME_INVALID);
    }

    #[test]
    fn process_data() {
        struct MockParticipant(Option<MockReaderGroup>);

        impl<'a> ReaderGroupCollection for &'a mut MockParticipant {
            type ReaderGroupsType = core::option::IterMut<'a, MockReaderGroup>;

            fn reader_groups(self) -> Self::ReaderGroupsType {
                self.0.iter_mut()
            }
        }

        struct MockReaderGroup(Option<MockReader>);

        impl<'a> RTPSGroup for &'a mut MockReaderGroup {
            type Endpoints = core::option::IterMut<'a, MockReader>;

            fn endpoints(self) -> Self::Endpoints {
                self.0.iter_mut()
            }
        }

        struct MockReader(bool);

        impl<'a> StatelessReaderBehavior<MockDataSubmessage> for &'a mut MockReader {
            fn receive_data(
                &mut self,
                _source_guid_prefix: GuidPrefix,
                _data: &MockDataSubmessage,
            ) {
                self.0 = true;
            }
        }

        let mut message_receiver = MessageReceiver::new();
        let data = MockDataSubmessage;
        let reader = MockReader(false);
        let reader_groups = MockReaderGroup(Some(reader));
        let mut participant = MockParticipant(Some(reader_groups));
        message_receiver.process_data(&data, &mut participant);
        assert_eq!(participant.0.unwrap().0.unwrap().0, true);
    }
}
