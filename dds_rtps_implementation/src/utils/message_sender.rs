use std::cell::RefCell;

use rust_rtps_pim::{
    behavior::{
        stateless_writer_behavior::StatelessWriterBehavior,
        writer::reader_locator::RtpsReaderLocator,
    },
    messages::{
        submessages::{DataSubmessage, GapSubmessageTrait, RtpsSubmessagePIM, RtpsSubmessageType},
        RtpsMessage, RtpsMessageHeader,
    },
    structure::{types::Locator, RtpsEntity, RtpsParticipant},
};

use crate::rtps_impl::rtps_writer_impl::RtpsWriterImpl;

use super::transport::TransportWrite;

pub trait RtpsSubmessageSender<'a, PSM>
where
    PSM: RtpsSubmessagePIM<'a>,
{
    fn create_submessages(&'a mut self) -> Vec<(Locator, Vec<RtpsSubmessageType<'a, PSM>>)>;
}

impl<'a, PSM, T> RtpsSubmessageSender<'a, PSM> for T
where
    T: StatelessWriterBehavior<'a, PSM::DataSubmessageType, PSM::GapSubmessageType>,
    T::ReaderLocator: RtpsReaderLocator,
    PSM: RtpsSubmessagePIM<'a>,
{
    fn create_submessages(&'a mut self) -> Vec<(Locator, Vec<RtpsSubmessageType<'a, PSM>>)> {
        let destined_submessages: Vec<(Locator, Vec<RtpsSubmessageType<'a, PSM>>)> = Vec::new();
        let destined_submessages = RefCell::new(destined_submessages);
        self.send_unsent_data(
            |reader_locator, data| {
                let mut destined_submessages_borrow = destined_submessages.borrow_mut();
                match destined_submessages_borrow
                    .iter_mut()
                    .find(|(locator, _)| locator == reader_locator.locator())
                {
                    Some((_, submessages)) => {
                        submessages.push(RtpsSubmessageType::<PSM>::Data(data))
                    }
                    None => destined_submessages_borrow.push((
                        *reader_locator.locator(),
                        vec![RtpsSubmessageType::<PSM>::Data(data)],
                    )),
                }
            },
            |reader_locator, gap| {
                let mut destined_submessages_borrow = destined_submessages.borrow_mut();
                match destined_submessages_borrow
                    .iter_mut()
                    .find(|(locator, _)| locator == reader_locator.locator())
                {
                    Some((_, submessages)) => submessages.push(RtpsSubmessageType::<PSM>::Gap(gap)),
                    None => destined_submessages_borrow.push((
                        *reader_locator.locator(),
                        vec![RtpsSubmessageType::<PSM>::Gap(gap)],
                    )),
                }
            },
        );
        destined_submessages.take()
    }
}

pub fn send_data<'a, Transport, PSM, Participant>(
    participant: &'a Participant,
    writer: &'a mut RtpsWriterImpl,
    transport: &'a mut Transport,
) where
    Transport: TransportWrite<'a>,
    Transport::Message: RtpsMessage<SubmessageType = RtpsSubmessageType<'a, PSM>>,
    PSM: RtpsSubmessagePIM<'a>,
    PSM::DataSubmessageType: DataSubmessage<'a>,
    PSM::GapSubmessageType: GapSubmessageTrait,
    Participant: RtpsParticipant + RtpsEntity,
{
    let header = RtpsMessageHeader {
        protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
        version: *participant.protocol_version(),
        vendor_id: *participant.vendor_id(),
        guid_prefix: *participant.guid().prefix(),
    };
    let destined_submessages = writer.create_submessages();
    for (dst_locator, submessages) in destined_submessages {
        let message = Transport::Message::new(&header, submessages);
        transport.write(&message, &dst_locator);
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::{
        messages::submessages::RtpsSubmessageType,
        structure::types::{self, LOCATOR_INVALID},
    };

    use super::*;

    #[derive(PartialEq, Debug)]
    struct MockPSM;

    impl<'a> RtpsSubmessagePIM<'a> for MockPSM {
        type AckNackSubmessageType = ();
        type DataSubmessageType = u8;
        type DataFragSubmessageType = ();
        type GapSubmessageType = ();
        type HeartbeatSubmessageType = ();
        type HeartbeatFragSubmessageType = ();
        type InfoDestinationSubmessageType = ();
        type InfoReplySubmessageType = ();
        type InfoSourceSubmessageType = ();
        type InfoTimestampSubmessageType = ();
        type NackFragSubmessageType = ();
        type PadSubmessageType = ();
    }

    struct MockReaderLocator(Locator);

    impl RtpsReaderLocator for MockReaderLocator {
        fn locator(&self) -> &types::Locator {
            &self.0
        }

        fn expects_inline_qos(&self) -> bool {
            todo!()
        }
    }

    #[test]
    fn submessage_send_empty() {
        struct MockBehavior;

        impl<'a> StatelessWriterBehavior<'a, u8, ()> for MockBehavior {
            type ReaderLocator = MockReaderLocator;

            fn send_unsent_data(
                &'a mut self,
                _send_data: impl FnMut(&Self::ReaderLocator, u8),
                _send_gap: impl FnMut(&Self::ReaderLocator, ()),
            ) {
            }
        }

        let mut writer = MockBehavior;
        let destined_submessages: Vec<(Locator, Vec<RtpsSubmessageType<'_, MockPSM>>)> =
            writer.create_submessages();

        assert!(destined_submessages.is_empty());
    }

    #[test]
    fn submessage_send_single_locator_send_only_data() {
        struct MockBehavior;

        impl<'a> StatelessWriterBehavior<'a, u8, ()> for MockBehavior {
            type ReaderLocator = MockReaderLocator;

            fn send_unsent_data(
                &'a mut self,
                mut send_data: impl FnMut(&Self::ReaderLocator, u8),
                _send_gap: impl FnMut(&Self::ReaderLocator, ()),
            ) {
                send_data(&MockReaderLocator(LOCATOR_INVALID), 0);
                send_data(&MockReaderLocator(LOCATOR_INVALID), 2);
            }
        }

        let mut writer = MockBehavior;
        let destined_submessages: Vec<(Locator, Vec<RtpsSubmessageType<'_, MockPSM>>)> =
            writer.create_submessages();
        let (dst_locator, submessages) = &destined_submessages[0];

        assert_eq!(dst_locator, &LOCATOR_INVALID);
        assert_eq!(
            submessages,
            &vec![RtpsSubmessageType::Data(0), RtpsSubmessageType::Data(2)]
        );
    }

    #[test]
    fn submessage_send_multiple_locator_send_data_and_gap() {
        struct MockBehavior;

        impl<'a> StatelessWriterBehavior<'a, u8, ()> for MockBehavior {
            type ReaderLocator = MockReaderLocator;

            fn send_unsent_data(
                &'a mut self,
                mut send_data: impl FnMut(&Self::ReaderLocator, u8),
                mut send_gap: impl FnMut(&Self::ReaderLocator, ()),
            ) {
                let locator1 = Locator::new(0, 1, [0; 16]);
                let locator2 = Locator::new(0, 2, [0; 16]);
                send_data(&MockReaderLocator(locator1), 0);
                send_data(&MockReaderLocator(locator1), 1);

                send_data(&MockReaderLocator(locator2), 2);
                send_gap(&MockReaderLocator(locator1), ());

                send_gap(&MockReaderLocator(locator2), ());
            }
        }

        let mut writer = MockBehavior;
        let destined_submessages: Vec<(Locator, Vec<RtpsSubmessageType<'_, MockPSM>>)> =
            writer.create_submessages();

        let locator1_submessages = &destined_submessages[0].1;
        let locator2_submessages = &destined_submessages[1].1;

        assert_eq!(destined_submessages.len(), 2);

        assert_eq!(locator1_submessages.len(), 3);
        assert_eq!(locator1_submessages[0], RtpsSubmessageType::Data(0));
        assert_eq!(locator1_submessages[1], RtpsSubmessageType::Data(1));
        assert_eq!(locator1_submessages[2], RtpsSubmessageType::Gap(()));

        assert_eq!(locator2_submessages.len(), 2);
        assert_eq!(locator2_submessages[0], RtpsSubmessageType::Data(2));
        assert_eq!(locator2_submessages[1], RtpsSubmessageType::Gap(()));
    }
}
