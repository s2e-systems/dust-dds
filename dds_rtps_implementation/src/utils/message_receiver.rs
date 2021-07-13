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
        RTPSEntity, RTPSGroup, RTPSParticipant,
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
        participant: &Participant,
        source_locator: Locator,
        message: Message,
    ) where
        Participant: RTPSEntity + RTPSParticipant,
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
        participant: &Participant,
    ) where
        Data: DataSubmessage<'a>,
        Participant: RTPSParticipant,
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
    use rust_rtps_pim::messages::types::SubmessageFlag;

    use super::*;

    struct MockTimestampSubmessageElement(Time);

    impl TimestampSubmessageElementType for MockTimestampSubmessageElement {
        fn new(_value: &Time) -> Self {
            todo!()
        }

        fn value(&self) -> Time {
            self.0.clone()
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
}
