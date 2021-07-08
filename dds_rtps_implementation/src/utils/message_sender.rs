use rust_rtps_pim::{
    behavior::stateless_writer::BestEffortBehavior,
    messages::{
        submessages::{GapSubmessagePIM, RtpsSubmessageType},
        RTPSMessage,
    },
    structure::{
        types::{Locator, SequenceNumber},
        RTPSCacheChange, RTPSHistoryCache,
    },
};

use crate::transport::TransportWrite;

pub fn send_data<HistoryCache, ReaderLocator, Transport>(
    writer_cache: &HistoryCache,
    reader_locators: &mut [ReaderLocator],
    last_change_sequence_number: SequenceNumber,
    transport: &mut Transport,
    header: &<<Transport as TransportWrite>::RTPSMessageType as RTPSMessage>::RtpsMessageHeaderType,
) where
    HistoryCache: RTPSHistoryCache,
    <HistoryCache as rust_rtps_pim::structure::RTPSHistoryCache>::CacheChange: RTPSCacheChange,
    ReaderLocator: BestEffortBehavior,
    Transport: TransportWrite,
{
    for reader_locator in reader_locators {
        let mut data_submessage_list: Vec<
            RtpsSubmessageType<
                <<Transport as TransportWrite>::RTPSMessageType as RTPSMessage>::PSM,
            >,
        > = vec![];
        let mut gap_submessage_list: Vec<
            RtpsSubmessageType<
                <<Transport as TransportWrite>::RTPSMessageType as RTPSMessage>::PSM,
            >,
        > = vec![];

        let mut submessages = vec![];

        reader_locator.best_effort_send_unsent_data(
                                &last_change_sequence_number,
                                writer_cache,
                                |data_submessage| {
                                    data_submessage_list
                                        .push(RtpsSubmessageType::Data(data_submessage))
                                },
                                |gap_submessage: <<<Transport as  TransportWrite>::RTPSMessageType as RTPSMessage>::PSM as GapSubmessagePIM>::GapSubmessageType| {
                                    gap_submessage_list
                                        .push(RtpsSubmessageType::Gap(gap_submessage))
                                },
                            );

        for data_submessage in data_submessage_list {
            submessages.push(data_submessage)
        }
        for gap_submessage in gap_submessage_list {
            submessages.push(gap_submessage);
        }

        let message = Transport::RTPSMessageType::new(header, submessages);
        let destination_locator = Locator::new([0; 4], [1; 4], [0; 16]);
        transport.write(&message, &destination_locator);
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
