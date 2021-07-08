use rust_rtps_pim::{
    behavior::stateless_writer::{RTPSReaderLocator, BestEffortBehavior},
    messages::{
        submessages::{
            AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessagePIM,
            HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM,
            InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM,
            NackFragSubmessagePIM, PadSubmessagePIM, RtpsSubmessageType,
        },
        RTPSMessage,
    },
    structure::{
        types::{Locator, SequenceNumber},
        RTPSCacheChange, RTPSHistoryCache,
    },
};

use crate::transport::TransportWrite;

pub fn create_messages<'a, PSM, HistoryCache, ReaderLocator>(
    writer_cache: &'a HistoryCache,
    reader_locator: &mut ReaderLocator,
    last_change_sequence_number: SequenceNumber,
) -> Vec<RtpsSubmessageType<'a, PSM>>
where
    PSM: AckNackSubmessagePIM
        + DataSubmessagePIM<'a>
        + DataFragSubmessagePIM
        + GapSubmessagePIM
        + HeartbeatSubmessagePIM
        + HeartbeatFragSubmessagePIM
        + InfoDestinationSubmessagePIM
        + InfoReplySubmessagePIM
        + InfoSourceSubmessagePIM
        + InfoTimestampSubmessagePIM
        + NackFragSubmessagePIM
        + PadSubmessagePIM,

        HistoryCache: RTPSHistoryCache,
        <HistoryCache as rust_rtps_pim::structure::RTPSHistoryCache>::CacheChange: RTPSCacheChange,
    ReaderLocator: BestEffortBehavior<'a, HistoryCache, PSM::DataSubmessageType, PSM::GapSubmessageType>,
{
    let mut data_submessage_list: Vec<RtpsSubmessageType<'a, PSM>> = vec![];
    let mut gap_submessage_list: Vec<RtpsSubmessageType<'a, PSM>> = vec![];

    let mut submessages = vec![];

    reader_locator.best_effort_send_unsent_data(
        &last_change_sequence_number,
        writer_cache,
        |data_submessage| data_submessage_list.push(RtpsSubmessageType::Data(data_submessage)),
        |gap_submessage: <PSM as GapSubmessagePIM>::GapSubmessageType| {
            gap_submessage_list.push(RtpsSubmessageType::Gap(gap_submessage))
        },
    );

    for data_submessage in data_submessage_list {
        submessages.push(data_submessage)
    }
    for gap_submessage in gap_submessage_list {
        submessages.push(gap_submessage);
    }
    submessages
}

pub fn send_data<HistoryCache, ReaderLocator, Transport>(
    writer_cache: &HistoryCache,
    reader_locators: &mut [ReaderLocator],
    last_change_sequence_number: SequenceNumber,
    transport: &mut Transport,
    header: &<<Transport as TransportWrite>::RTPSMessageType as RTPSMessage>::RtpsMessageHeaderType,
) where
    HistoryCache: RTPSHistoryCache,
    <HistoryCache as rust_rtps_pim::structure::RTPSHistoryCache>::CacheChange: RTPSCacheChange,
    ReaderLocator: RTPSReaderLocator,
    for<'a> ReaderLocator: BestEffortBehavior<'a, HistoryCache, <<<Transport as TransportWrite>::RTPSMessageType as RTPSMessage<'a>>::PSM as DataSubmessagePIM<'a>>::DataSubmessageType, <<<Transport as TransportWrite>::RTPSMessageType as RTPSMessage<'a>>::PSM as GapSubmessagePIM>::GapSubmessageType>,
    Transport: TransportWrite,
{
    for reader_locator in reader_locators {
        let submessages = create_messages(writer_cache, reader_locator, last_change_sequence_number);
        let message = Transport::RTPSMessageType::new(header, submessages);
        transport.write(&message, reader_locator.locator());
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::RTPSHistoryCache;

    use crate::transport::TransportWrite;

    struct MockCacheChange;
    struct MockHistoryCache;
    impl RTPSHistoryCache for MockHistoryCache {
        type CacheChange = MockCacheChange;

        fn new() -> Self
        where
            Self: Sized,
        {
            todo!()
        }

        fn add_change(&mut self, change: Self::CacheChange) {
            todo!()
        }

        fn remove_change(&mut self, seq_num: &rust_rtps_pim::structure::types::SequenceNumber) {
            todo!()
        }

        fn get_change(
            &self,
            seq_num: &rust_rtps_pim::structure::types::SequenceNumber,
        ) -> Option<&Self::CacheChange> {
            todo!()
        }

        fn get_seq_num_min(&self) -> Option<&rust_rtps_pim::structure::types::SequenceNumber> {
            todo!()
        }

        fn get_seq_num_max(&self) -> Option<&rust_rtps_pim::structure::types::SequenceNumber> {
            todo!()
        }
    }

    // struct MockTransport;
    // impl TransportWrite for MockTransport {
    //     type RtpsMessageHeaderType = ();
    //     type RTPSMessageType = ();

    //     fn write<'a>(
    //     &mut self,
    //     message: &rust_rtps_pim::messages::RTPSMessage::Constructed,
    //     destination_locator: &rust_rtps_pim::structure::types::Locator,
    // ) {
    //     todo!()
    // }
    // }

    #[test]
    fn send_data_happy() {
        let writer_cache = MockHistoryCache;
        let reader_locators = [0];
        let last_change_sequence_number = 1_i64;
        // let transport = MockTransport;
    }
}
