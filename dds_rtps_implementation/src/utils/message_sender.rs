use std::cell::RefCell;

use rust_rtps_pim::{behavior::{
        stateless_writer_behavior::StatelessWriterBehavior,
        writer::reader_locator::RTPSReaderLocator,
    }, messages::{RTPSMessage, RtpsMessageHeader, submessages::{DataSubmessage, GapSubmessage, RtpsSubmessagePIM, RtpsSubmessageType}}, structure::types::{GuidPrefix, Locator, ProtocolVersion, VendorId, LOCATOR_INVALID}};

use crate::rtps_impl::rtps_writer_impl::RTPSWriterImpl;

use super::transport::TransportWrite;

pub fn create_submessages<'a, PSM, StatelessWriter>(
    writer: StatelessWriter,
) -> Vec<(Locator, Vec<RtpsSubmessageType<'a, PSM>>)>
where
    PSM: RtpsSubmessagePIM<'a>,
    StatelessWriter: StatelessWriterBehavior<PSM::DataSubmessageType, PSM::GapSubmessageType>,
    StatelessWriter::ReaderLocator: RTPSReaderLocator,
{
    let mut dst_locator = LOCATOR_INVALID;
    let submessages = RefCell::new(Vec::new());
    writer.send_unsent_data(
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

pub fn send_data<'a, Transport, PSM>(
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    guid_prefix: GuidPrefix,
    writer: &'a mut RTPSWriterImpl,
    transport: &'a mut Transport,
) where
    Transport: TransportWrite<'a>,
    Transport::Message: RTPSMessage<SubmessageType=RtpsSubmessageType<'a, PSM>>,
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
    let destined_submessages = create_submessages::<PSM, _>(&mut *writer);
    for (dst_locator, submessages) in destined_submessages {
        let message = Transport::Message::new(&header, submessages);
        transport.write(&message, &dst_locator);
    }
}

#[cfg(test)]
mod tests {
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
    fn message_send_test() {
        struct MockBehavior;

        impl StatelessWriterBehavior<u8, ()> for MockBehavior {
            type ReaderLocator = MockReaderLocator;

            fn send_unsent_data(
                self,
                mut send_data: impl FnMut(&Self::ReaderLocator, u8),
                _send_gap: impl FnMut(&Self::ReaderLocator, ()),
            ) {
                send_data(&MockReaderLocator, 0);
                send_data(&MockReaderLocator, 2);
            }
        }

        let writer = MockBehavior;
        let destined_submessages = create_submessages::<MockPSM, _>(writer);
        let (dst_locator, submessages) = &destined_submessages[0];

        assert_eq!(dst_locator, &LOCATOR_INVALID);
        assert_eq!(
            submessages,
            &vec![RtpsSubmessageType::Data(0), RtpsSubmessageType::Data(2)]
        );
    }
}
