use rust_rtps_pim::{
    behavior::stateless_writer::{BestEffortBehavior, RTPSReaderLocator},
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
    ReaderLocator:
        BestEffortBehavior<'a, HistoryCache, PSM::DataSubmessageType, PSM::GapSubmessageType>,
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
        let submessages =
            create_messages(writer_cache, reader_locator, last_change_sequence_number);
        let message = Transport::RTPSMessageType::new(header, submessages);
        transport.write(&message, reader_locator.locator());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_rtps_pim::structure::RTPSHistoryCache;

    struct MockCacheChange;
    impl RTPSCacheChange for MockCacheChange {
        type DataType = Vec<u8>;
        type InstanceHandleType = ();
        type InlineQosType = ();

        fn kind(&self) -> rust_rtps_pim::structure::types::ChangeKind {
            todo!()
        }

        fn writer_guid(&self) -> &rust_rtps_pim::structure::types::GUID {
            todo!()
        }

        fn instance_handle(&self) -> &Self::InstanceHandleType {
            todo!()
        }

        fn sequence_number(&self) -> &SequenceNumber {
            todo!()
        }

        fn data_value(&self) -> &Self::DataType {
            todo!()
        }

        fn inline_qos(&self) -> &Self::InlineQosType {
            todo!()
        }
    }

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

    #[derive(Debug, PartialEq)]
    struct MockPSM;

    impl AckNackSubmessagePIM for MockPSM {
        type AckNackSubmessageType = ();
    }

    impl<'a> DataSubmessagePIM<'a> for MockPSM {
        type DataSubmessageType = ();
    }
    impl DataFragSubmessagePIM for MockPSM {
        type DataFragSubmessageType = ();
    }
    impl GapSubmessagePIM for MockPSM {
        type GapSubmessageType = u8;
    }
    impl HeartbeatSubmessagePIM for MockPSM {
        type HeartbeatSubmessageType = ();
    }
    impl HeartbeatFragSubmessagePIM for MockPSM {
        type HeartbeatFragSubmessageType = ();
    }
    impl InfoDestinationSubmessagePIM for MockPSM {
        type InfoDestinationSubmessageType = ();
    }
    impl InfoReplySubmessagePIM for MockPSM {
        type InfoReplySubmessageType = ();
    }
    impl InfoSourceSubmessagePIM for MockPSM {
        type InfoSourceSubmessageType = ();
    }
    impl InfoTimestampSubmessagePIM for MockPSM {
        type InfoTimestampSubmessageType = ();
    }
    impl NackFragSubmessagePIM for MockPSM {
        type NackFragSubmessageType = ();
    }
    impl PadSubmessagePIM for MockPSM {
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

        let writer_cache = MockHistoryCache;
        let mut reader_locator = MockReaderLocator;
        let last_change_sequence_number = 1_i64;
        let submessages = create_messages::<MockPSM, _, _>(
            &writer_cache,
            &mut reader_locator,
            last_change_sequence_number,
        );
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
