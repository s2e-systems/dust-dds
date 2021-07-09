use rust_rtps_pim::{
    behavior::{
        stateless_writer::{BestEffortBehavior, RTPSReaderLocator, RTPSStatelessWriter},
        RTPSWriter,
    },
    messages::{
        submessages::{RtpsSubmessagePIM, RtpsSubmessageType},
        RTPSMessage, RtpsMessageHeader,
    },
    structure::types::{Locator, SequenceNumber},
};

pub trait ReaderLocatorMessageSender<'a, HistoryCache, PSM>
where
    PSM: RtpsSubmessagePIM<'a>,
{
    fn create_submessages(
        &mut self,
        writer_cache: &'a HistoryCache,
        last_change_sequence_number: SequenceNumber,
    ) -> Vec<RtpsSubmessageType<'a, PSM>>;
}

impl<'a, HistoryCache, PSM, T> ReaderLocatorMessageSender<'a, HistoryCache, PSM> for T
where
    PSM: RtpsSubmessagePIM<'a>,
    T: BestEffortBehavior<'a, HistoryCache, PSM::DataSubmessageType, PSM::GapSubmessageType> + 'a,
{
    fn create_submessages(
        &mut self,
        writer_cache: &'a HistoryCache,
        last_change_sequence_number: SequenceNumber,
    ) -> Vec<RtpsSubmessageType<'a, PSM>> {
        let mut data_submessage_list: Vec<RtpsSubmessageType<'a, PSM>> = vec![];
        let mut gap_submessage_list: Vec<RtpsSubmessageType<'a, PSM>> = vec![];

        let mut submessages = vec![];

        self.best_effort_send_unsent_data(
            &last_change_sequence_number,
            writer_cache,
            |data_submessage| data_submessage_list.push(RtpsSubmessageType::Data(data_submessage)),
            |gap_submessage| gap_submessage_list.push(RtpsSubmessageType::Gap(gap_submessage)),
        );

        for data_submessage in data_submessage_list {
            submessages.push(data_submessage)
        }
        for gap_submessage in gap_submessage_list {
            submessages.push(gap_submessage);
        }
        submessages
    }
}

pub trait StatelessWriterMessageSender<Message, Sender> {
    fn create_messages(self, header: &RtpsMessageHeader, send_message: Sender) -> Vec<Message>;
}

impl<'a, Message, Sender, PSM, T> StatelessWriterMessageSender<Message, Sender> for &'a mut T
where
    T: RTPSStatelessWriter + RTPSWriter,
    Sender: Fn(Message, &'a Locator),
    PSM: RtpsSubmessagePIM<'a>,
    T::ReaderLocatorType: RTPSReaderLocator,
    T::ReaderLocatorType: ReaderLocatorMessageSender<'a, T::HistoryCacheType, PSM>,
    Message: RTPSMessage<SubmessageType = RtpsSubmessageType<'a, PSM>>,
{
    fn create_messages(self, header: &RtpsMessageHeader, send_message: Sender) -> Vec<Message> {
        let mut messages = vec![];
        let last_change_sequence_number = *self.last_change_sequence_number();
        let (writer_cache, reader_locators) = self.writer_cache_and_reader_locators();
        for reader_locator in reader_locators {
            let submessages =
                reader_locator.create_submessages(writer_cache, last_change_sequence_number);
            let message = Message::new(header, submessages);
            messages.push(message);
            // send_message(message, reader_locator.locator())
        }
        messages
    }
}
// impl<T> StatelessWriterMessageSender for T
// where
//     T: RTPSStatelessWriter + RTPSWriter,
//     T::ReaderLocatorType:
//         RTPSReaderLocator + for<'a> ReaderLocatorMessageSender<'a, T::HistoryCacheType>,
// {
//     fn send_data(&mut self, header: &RtpsMessageHeader, transport: &mut Transport) {
//         let last_change_sequence_number = *self.last_change_sequence_number();
//         let (writer_cache, reader_locators) = self.writer_cache_and_reader_locators();
//         for reader_locator in reader_locators {
//             let submessages =
//                 reader_locator.create_messages(writer_cache, last_change_sequence_number);
//             let message = Transport::RTPSMessageType::new(header, submessages);
//             transport.write(&message, reader_locator.locator());
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct MockDataSubmessage;
    #[derive(Debug, PartialEq)]
    struct MockGapSubmessage;

    #[derive(Debug, PartialEq)]
    struct MockPSM;

    impl<'a> RtpsSubmessagePIM<'a> for MockPSM {
        type AckNackSubmessageType = ();
        type DataSubmessageType = MockDataSubmessage;
        type DataFragSubmessageType = ();
        type GapSubmessageType = MockGapSubmessage;
        type HeartbeatSubmessageType = ();
        type HeartbeatFragSubmessageType = ();
        type InfoDestinationSubmessageType = ();
        type InfoReplySubmessageType = ();
        type InfoSourceSubmessageType = ();
        type InfoTimestampSubmessageType = ();
        type NackFragSubmessageType = ();
        type PadSubmessageType = ();
    }

    #[test]
    fn reader_locator_send_data_happy() {
        struct MockReaderLocator;
        impl<'a, HistoryCache>
            BestEffortBehavior<'a, HistoryCache, MockDataSubmessage, MockGapSubmessage>
            for MockReaderLocator
        {
            fn best_effort_send_unsent_data(
                &mut self,
                _last_change_sequence_number: &SequenceNumber,
                _writer_cache: &'a HistoryCache,
                mut send_data: impl FnMut(MockDataSubmessage),
                mut send_gap: impl FnMut(MockGapSubmessage),
            ) {
                send_data(MockDataSubmessage);
                send_data(MockDataSubmessage);
                send_data(MockDataSubmessage);
                send_gap(MockGapSubmessage);
                send_gap(MockGapSubmessage);
            }
        }
        let mut reader_locator = MockReaderLocator;
        let last_change_sequence_number = 1_i64;
        let submessages: Vec<RtpsSubmessageType<MockPSM>> =
            reader_locator.create_submessages(&(), last_change_sequence_number);
        assert_eq!(
            submessages,
            vec![
                RtpsSubmessageType::Data(MockDataSubmessage),
                RtpsSubmessageType::Data(MockDataSubmessage),
                RtpsSubmessageType::Data(MockDataSubmessage),
                RtpsSubmessageType::Gap(MockGapSubmessage),
                RtpsSubmessageType::Gap(MockGapSubmessage),
            ]
        )
    }
}
