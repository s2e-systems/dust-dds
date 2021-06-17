use rust_rtps_pim::{
    behavior::{
        stateless_writer::{best_effort_send_unsent_data, RTPSReaderLocator, RTPSStatelessWriter},
        types::DurationPIM,
        RTPSWriter,
    },
    messages::{
        submessage_elements::{
            CountSubmessageElementPIM, EntityIdSubmessageElementPIM,
            FragmentNumberSetSubmessageElementPIM, FragmentNumberSubmessageElementPIM,
            GuidPrefixSubmessageElementPIM, LocatorListSubmessageElementPIM,
            ParameterListSubmessageElementPIM, ProtocolVersionSubmessageElementPIM,
            SequenceNumberSetSubmessageElementPIM, SequenceNumberSubmessageElementPIM,
            SerializedDataFragmentSubmessageElementPIM, SerializedDataSubmessageElementPIM,
            TimestampSubmessageElementPIM, ULongSubmessageElementPIM, UShortSubmessageElementPIM,
            VendorIdSubmessageElementPIM,
        },
        submessages::{
            AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessagePIM,
            HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM,
            InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM,
            NackFragSubmessagePIM, PadSubmessagePIM, RtpsSubmessageType,
        },
        types::{
            CountPIM, FragmentNumberPIM, ParameterIdPIM, ProtocolIdPIM, SubmessageKindPIM, TimePIM,
        },
        RTPSMessage, RTPSMessagePIM, RtpsMessageHeaderPIM, RtpsSubmessageHeaderPIM,
    },
    structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM, ProtocolVersionPIM,
        SequenceNumberPIM, VendorIdPIM, GUIDPIM,
    },
};

use crate::{rtps_impl::rtps_writer_impl::RTPSWriterImpl, transport::Transport};

pub fn send_data<
    PSM: GuidPrefixPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + LocatorPIM
        + VendorIdPIM
        + DurationPIM
        + InstanceHandlePIM
        + DataPIM
        + ProtocolVersionPIM
        + ParameterIdPIM
        + GUIDPIM<PSM>
        + SubmessageKindPIM
        + ProtocolIdPIM
        + CountPIM
        + FragmentNumberPIM
        + UShortSubmessageElementPIM
        + ULongSubmessageElementPIM
        + TimePIM
        + ParameterListSubmessageElementPIM<PSM>
        + RtpsSubmessageHeaderPIM<PSM>
        + CountSubmessageElementPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + SequenceNumberSetSubmessageElementPIM<PSM>
        + FragmentNumberSubmessageElementPIM<PSM>
        + GuidPrefixSubmessageElementPIM<PSM>
        + LocatorListSubmessageElementPIM<PSM>
        + ProtocolVersionSubmessageElementPIM<PSM>
        + VendorIdSubmessageElementPIM<PSM>
        + TimestampSubmessageElementPIM<PSM>
        + FragmentNumberSubmessageElementPIM<PSM>
        + FragmentNumberSetSubmessageElementPIM<PSM>
        + AckNackSubmessagePIM<PSM>
        + HeartbeatSubmessagePIM<PSM>
        + HeartbeatFragSubmessagePIM<PSM>
        + InfoDestinationSubmessagePIM<PSM>
        + InfoReplySubmessagePIM<PSM>
        + InfoSourceSubmessagePIM<PSM>
        + InfoTimestampSubmessagePIM<PSM>
        + NackFragSubmessagePIM<PSM>
        + PadSubmessagePIM<PSM>
        + for<'a> SerializedDataSubmessageElementPIM<'a>
        + for<'a> SerializedDataFragmentSubmessageElementPIM<'a>
        + for<'a> DataFragSubmessagePIM<'a, PSM>
        + for<'a> DataSubmessagePIM<'a, PSM>
        + GapSubmessagePIM<PSM>
        + for<'a> RTPSMessagePIM<'a, PSM>
        + RtpsMessageHeaderPIM<PSM>
        + Sized
        + 'static,
>(
    writer: &mut RTPSWriterImpl<PSM>,
    transport: &mut dyn Transport<PSM>,
) where
    PSM::SequenceNumberType: Clone + Copy + Ord,
    PSM::GuidPrefixType: Clone,
    PSM::LocatorType: Clone + PartialEq,
    PSM::GUIDType: Copy,
    PSM::ParameterListSubmessageElementType: Clone,
{
    // for writer_group in &rtps_participant_impl.rtps_writer_groups {
    // let writer_group_lock = writer_group.lock();
    // let writer_list = rtps_writer_group_impl.writer_list();
    // for writer in writer_list {
    let last_change_sequence_number = *writer.last_change_sequence_number();
    let (reader_locators, writer_cache) = writer.reader_locators();
    for reader_locator in reader_locators {
        let mut data_submessage_list: Vec<RtpsSubmessageType<'_, PSM>> = vec![];
        let mut gap_submessage_list: Vec<RtpsSubmessageType<'_, PSM>> = vec![];

        // let mut submessages = vec![];
        best_effort_send_unsent_data(
            reader_locator,
            &last_change_sequence_number,
            writer_cache,
            |data_submessage| data_submessage_list.push(RtpsSubmessageType::Data(data_submessage)),
            |gap_submessage| gap_submessage_list.push(RtpsSubmessageType::Gap(gap_submessage)),
        );

        let protocol = PSM::PROTOCOL_RTPS;
        // let version = rtps_participant_impl.protocol_version();
        // let vendor_id = rtps_participant_impl.vendor_id();
        // let guid_prefix = rtps_participant_impl.guid().prefix();
        // for data_submessage in &data_submessage_list {
        //     submessages.push(RtpsSubmessageType::Data(data_submessage))
        // }
        // for gap_submessage in &gap_submessage_list {
        //     submessages.push(gap_submessage);
        // }

        let message = PSM::RTPSMessageType::new(
            protocol,
            PSM::PROTOCOLVERSION_2_4,
            PSM::VENDOR_ID_UNKNOWN,
            PSM::GUIDPREFIX_UNKNOWN,
            data_submessage_list,
        );
        transport.write(&message, reader_locator.locator());
    }
}
