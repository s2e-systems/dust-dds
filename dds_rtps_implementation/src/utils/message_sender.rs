use std::cell::RefCell;

use rust_rtps_pim::{
    behavior::{
        stateless_writer_behavior::StatelessWriterBehavior,
        writer::reader_locator::RTPSReaderLocator,
    },
    messages::{
        submessages::{DataSubmessage, GapSubmessage, RtpsSubmessagePIM, RtpsSubmessageType},
        RTPSMessage, RtpsMessageHeader,
    },
    structure::types::{GuidPrefix, Locator, ProtocolVersion, VendorId, LOCATOR_INVALID},
};

use crate::rtps_impl::rtps_writer_impl::RTPSWriterImpl;

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
    T::ReaderLocator: RTPSReaderLocator,
    PSM: RtpsSubmessagePIM<'a>,
{
    fn create_submessages(&'a mut self) -> Vec<(Locator, Vec<RtpsSubmessageType<'a, PSM>>)> {
        let mut dst_locator = LOCATOR_INVALID;
        let submessages = RefCell::new(Vec::new());
        self.send_unsent_data(
            |reader_locator, data| {
                dst_locator = *reader_locator.locator();
                submessages
                    .borrow_mut()
                    .push(RtpsSubmessageType::<PSM>::Data(data));
            },
            |_reader_locator, gap| {
                submessages.borrow_mut().push(
                    // *reader_locator.locator(),
                    RtpsSubmessageType::<PSM>::Gap(gap),
                );
            },
        );
        vec![(dst_locator, submessages.take())]
    }
}

pub fn send_data<'a, Transport, PSM>(
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    guid_prefix: GuidPrefix,
    writer: &'a mut RTPSWriterImpl,
    transport: &'a mut Transport,
) where
    Transport: TransportWrite<'a>,
    Transport::Message: RTPSMessage<SubmessageType = RtpsSubmessageType<'a, PSM>>,
    PSM: RtpsSubmessagePIM<'a>,
    PSM::DataSubmessageType: DataSubmessage<'a>,
    PSM::GapSubmessageType: GapSubmessage,
{
    let header = RtpsMessageHeader {
        protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
        version: protocol_version,
        vendor_id,
        guid_prefix,
    };
    let destined_submessages = writer.create_submessages();
    for (dst_locator, submessages) in destined_submessages {
        let message = Transport::Message::new(&header, submessages);
        transport.write(&message, &dst_locator);
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::messages::submessages::RtpsSubmessageType;

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

    struct MockReaderLocator;

    impl RTPSReaderLocator for MockReaderLocator {
        fn locator(&self) -> &rust_rtps_pim::structure::types::Locator {
            &LOCATOR_INVALID
        }

        fn expects_inline_qos(&self) -> bool {
            todo!()
        }
    }

    #[test]
    fn message_send_no_data() {
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
    fn message_send_test() {
        struct MockBehavior;

        impl<'a> StatelessWriterBehavior<'a, u8, ()> for MockBehavior {
            type ReaderLocator = MockReaderLocator;

            fn send_unsent_data(
                &'a mut self,
                mut send_data: impl FnMut(&Self::ReaderLocator, u8),
                _send_gap: impl FnMut(&Self::ReaderLocator, ()),
            ) {
                send_data(&MockReaderLocator, 0);
                send_data(&MockReaderLocator, 2);
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
}
