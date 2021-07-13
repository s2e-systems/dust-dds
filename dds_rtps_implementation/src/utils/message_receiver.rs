use rust_rtps_pim::{
    behavior::stateless_reader_behavior::StatelessReaderBehavior,
    messages::{
        submessages::{DataSubmessage, RtpsSubmessagePIM, RtpsSubmessageType},
        types::TIME_INVALID,
        RTPSMessage,
    },
    structure::{
        types::{
            Locator, GUIDPREFIX_UNKNOWN, LOCATOR_ADDRESS_INVALID, LOCATOR_PORT_INVALID,
            PROTOCOLVERSION_2_4, VENDOR_ID_UNKNOWN,
        },
        RTPSEntity,
    },
};

use crate::rtps_impl::rtps_participant_impl::RTPSParticipantImpl;

pub fn message_receiver<'a, PSM>(
    participant: &RTPSParticipantImpl,
    message: &impl RTPSMessage<SubmessageType = RtpsSubmessageType<'a, PSM>>,
    source_locator: Locator,
) where
    PSM: RtpsSubmessagePIM<'a>,
    PSM::DataSubmessageType: DataSubmessage<'a>,
{
    let mut source_version = PROTOCOLVERSION_2_4;
    let mut source_vendor_id = VENDOR_ID_UNKNOWN;
    let mut source_guid_prefix = GUIDPREFIX_UNKNOWN;
    let dest_guid_prefix = *participant.guid().prefix();
    let unicast_reply_locator_list = vec![Locator::new(
        *source_locator.kind(),
        LOCATOR_PORT_INVALID,
        *source_locator.address(),
    )];
    let multicast_reply_locator_list = vec![Locator::new(
        *source_locator.kind(),
        LOCATOR_PORT_INVALID,
        LOCATOR_ADDRESS_INVALID,
    )];
    let have_timestamp = false;
    let timestamp = TIME_INVALID;

    source_version = message.header().version;
    source_vendor_id = message.header().vendor_id;
    source_guid_prefix = message.header().guid_prefix;

    for submessage in message.submessages() {
        match submessage {
            RtpsSubmessageType::AckNack(_) => todo!(),
            RtpsSubmessageType::Data(data_submessage) => {
                let reader_group = participant.builtin_reader_group.lock();
                for reader in reader_group.reader_list() {
                    let mut reader = reader.lock();
                    reader.receive_data(source_guid_prefix, data_submessage);
                }
            }
            RtpsSubmessageType::DataFrag(_) => todo!(),
            RtpsSubmessageType::Gap(_) => todo!(),
            RtpsSubmessageType::Heartbeat(_) => todo!(),
            RtpsSubmessageType::HeartbeatFrag(_) => todo!(),
            RtpsSubmessageType::InfoDestination(_) => todo!(),
            RtpsSubmessageType::InfoReply(_) => todo!(),
            RtpsSubmessageType::InfoSource(_) => todo!(),
            RtpsSubmessageType::InfoTimestamp(_) => todo!(),
            RtpsSubmessageType::NackFrag(_) => todo!(),
            RtpsSubmessageType::Pad(_) => todo!(),
        }
    }
}
