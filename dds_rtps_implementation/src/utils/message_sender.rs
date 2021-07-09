use rust_rtps_pim::{
    behavior::{
        stateless_writer::{BestEffortBehavior, RTPSReaderLocator, RTPSStatelessWriter},
        RTPSWriter,
    },
    messages::{
        submessages::{RtpsSubmessagePIM, RtpsSubmessageType},
        RTPSMessage,
    },
    structure::types::SequenceNumber,
};

use crate::transport::TransportWrite;

pub trait ReaderLocatorMessageSender<'a, PSM, HistoryCache>
where
    PSM: RtpsSubmessagePIM<'a>,
{
    fn create_messages(
        &mut self,
        writer_cache: &'a HistoryCache,
        last_change_sequence_number: SequenceNumber,
    ) -> Vec<RtpsSubmessageType<'a, PSM>>;
}

impl<'a, PSM, HistoryCache, T> ReaderLocatorMessageSender<'a, PSM, HistoryCache> for T
where
    PSM: RtpsSubmessagePIM<'a>,
    T: BestEffortBehavior<'a, HistoryCache, PSM::DataSubmessageType, PSM::GapSubmessageType>,
{
    fn create_messages(
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

pub trait StatelessWriterMessageSender<Transport>
where
    Transport: TransportWrite,
{
    fn send_data(
        &mut self,
        header: &<<Transport as TransportWrite>::RTPSMessageType as RTPSMessage>::RtpsMessageHeaderType,
        transport: &mut Transport,
    );
}

impl<Transport, T> StatelessWriterMessageSender<Transport> for T
where
    T: RTPSStatelessWriter + RTPSWriter,
    Transport: TransportWrite,
    T::ReaderLocatorPIM: for<'a> BestEffortBehavior<'a, T::HistoryCacheType, <<Transport::RTPSMessageType as RTPSMessage<'a>>::PSM as RtpsSubmessagePIM<'a>>::DataSubmessageType, <<Transport::RTPSMessageType as RTPSMessage<'a>>::PSM as RtpsSubmessagePIM<'a>>::GapSubmessageType> + RTPSReaderLocator,
{
    fn send_data(
        &mut self,
        header: &<<Transport as TransportWrite>::RTPSMessageType as RTPSMessage>::RtpsMessageHeaderType,
        transport: &mut Transport,
    ) {
        let last_change_sequence_number = *self.last_change_sequence_number();
        let (writer_cache, reader_locators) = self.writer_cache_and_reader_locators();
        for reader_locator in reader_locators {
            let submessages =
                reader_locator.create_messages(writer_cache, last_change_sequence_number);
            let message = Transport::RTPSMessageType::new(header, submessages);
            transport.write(&message, reader_locator.locator());
        }
    }
}

// pub fn send_data<HistoryCache, ReaderLocator, Transport>(
//     writer_cache: &HistoryCache,
//     reader_locators: &mut [ReaderLocator],
//     last_change_sequence_number: SequenceNumber,
//     transport: &mut Transport,
//     header: &<<Transport as TransportWrite>::RTPSMessageType as RTPSMessage>::RtpsMessageHeaderType,
// ) where
//     HistoryCache: RTPSHistoryCache,
//     <HistoryCache as rust_rtps_pim::structure::RTPSHistoryCache>::CacheChange: RTPSCacheChange,
//     ReaderLocator: RTPSReaderLocator,
//     for<'a> ReaderLocator: BestEffortBehavior<'a, HistoryCache, <<<Transport as TransportWrite>::RTPSMessageType as RTPSMessage<'a>>::PSM as RtpsSubmessagePIM<'a>>::DataSubmessageType, <<<Transport as TransportWrite>::RTPSMessageType as RTPSMessage<'a>>::PSM as RtpsSubmessagePIM<'a>>::GapSubmessageType>,
//     Transport: TransportWrite,
// {
//     for reader_locator in reader_locators {
//         let submessages = reader_locator.create_messages(writer_cache, last_change_sequence_number);
//         let message = Transport::RTPSMessageType::new(header, submessages);
//         transport.write(&message, reader_locator.locator());
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct MockPSM;

    impl<'a> RtpsSubmessagePIM<'a> for MockPSM {
        type AckNackSubmessageType = ();
        type DataSubmessageType = ();
        type DataFragSubmessageType = ();
        type GapSubmessageType = u8;
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
    fn send_data_happy() {
        struct MockReaderLocator;
        impl<'a, HistoryCache> BestEffortBehavior<'a, HistoryCache, (), u8> for MockReaderLocator {
            fn best_effort_send_unsent_data(
                &mut self,
                _last_change_sequence_number: &SequenceNumber,
                _writer_cache: &'a HistoryCache,
                mut send_data: impl FnMut(()),
                mut send_gap: impl FnMut(u8),
            ) {
                send_data(());
                send_data(());
                send_data(());
                send_gap(1);
                send_gap(2);
            }
        }
        let mut reader_locator = MockReaderLocator;
        let last_change_sequence_number = 1_i64;
        let submessages: Vec<RtpsSubmessageType<MockPSM>> =
            reader_locator.create_messages(&(), last_change_sequence_number);
        assert_eq!(
            submessages,
            vec![
                RtpsSubmessageType::Data(()),
                RtpsSubmessageType::Data(()),
                RtpsSubmessageType::Data(()),
                RtpsSubmessageType::Gap(1),
                RtpsSubmessageType::Gap(2),
            ]
        )
        // let transport = MockTransport;
    }
}
