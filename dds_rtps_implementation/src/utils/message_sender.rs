use rust_rtps_pim::{
    behavior::{
        stateless_writer::{best_effort_send_unsent_data, RTPSReaderLocator, RTPSStatelessWriter},
        types::DurationPIM,
        RTPSWriter,
    },
    messages::{
        submessage_elements::{
            EntityIdSubmessageElementPIM, ParameterListSubmessageElementPIM,
            SequenceNumberSetSubmessageElementPIM, SequenceNumberSubmessageElementPIM,
            SerializedDataSubmessageElementPIM,
        },
        submessages::{DataSubmessagePIM, GapSubmessagePIM},
        types::{ParameterIdPIM, ProtocolIdPIM, SubmessageKindPIM},
        RTPSMessageConstructor, RTPSMessagePIM, RtpsMessageHeaderPIM, RtpsSubmessageHeaderPIM,
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
        + ParameterListSubmessageElementPIM<PSM>
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + SequenceNumberSetSubmessageElementPIM<PSM>
        + for<'a> SerializedDataSubmessageElementPIM<'a>
        + for<'a> DataSubmessagePIM<'a, PSM>
        + for<'a> GapSubmessagePIM<'a, PSM>
        + for<'a> RTPSMessagePIM<'a, PSM>
        + for<'a> RtpsMessageHeaderPIM<'a, PSM>
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
        let mut data_submessage_list: Vec<<PSM as DataSubmessagePIM<PSM>>::DataSubmessageType> = vec![];
        let mut gap_submessage_list: Vec<<PSM as GapSubmessagePIM<PSM>>::GapSubmessageType>  = vec![];
        best_effort_send_unsent_data(
            reader_locator,
            &last_change_sequence_number,
            writer_cache,
            |data_submessage| data_submessage_list.push(data_submessage),
            |gap_submessage| gap_submessage_list.push(gap_submessage),
        );

        let protocol = PSM::PROTOCOL_RTPS;
        // let version = rtps_participant_impl.protocol_version();
        // let vendor_id = rtps_participant_impl.vendor_id();
        // let guid_prefix = rtps_participant_impl.guid().prefix();

        let mut submessages: Vec<&dyn rust_rtps_pim::messages::Submessage<PSM>> = vec![];
        for data_submessage in &data_submessage_list {
            submessages.push(data_submessage)
        }
        for gap_submessage in &gap_submessage_list {
            submessages.push(gap_submessage);
        }

        let message = PSM::RTPSMessageType::new(
            protocol,
            PSM::PROTOCOLVERSION_2_4,
            PSM::VENDOR_ID_UNKNOWN,
            PSM::GUIDPREFIX_UNKNOWN,
            submessages.as_slice(),
        );
        transport.write(&message, reader_locator.locator());
    }
}
