use rust_rtps_pim::{messages::{RTPSMessagePIM, RtpsMessageHeaderPIM, RtpsSubmessageHeaderPIM, submessage_elements::{
            CountSubmessageElementPIM, EntityIdSubmessageElementPIM,
            FragmentNumberSetSubmessageElementPIM, FragmentNumberSubmessageElementPIM,
            GuidPrefixSubmessageElementPIM, LocatorListSubmessageElementPIM,
            ParameterListSubmessageElementPIM, ProtocolVersionSubmessageElementPIM,
            SequenceNumberSetSubmessageElementPIM, SequenceNumberSubmessageElementPIM,
            SerializedDataFragmentSubmessageElementPIM, SerializedDataSubmessageElementPIM,
            TimestampSubmessageElementPIM, ULongSubmessageElementPIM, UShortSubmessageElementPIM,
            VendorIdSubmessageElementPIM,
        }, submessages::{AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessage, GapSubmessagePIM, HeartbeatFragSubmessagePIM, HeartbeatSubmessage, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM, InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM, NackFragSubmessagePIM, PadSubmessagePIM}, types::{
            CountPIM, FragmentNumberPIM, ParameterIdPIM, ProtocolIdPIM, SubmessageKindPIM, TimePIM,
        }}, structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, LocatorPIM, ProtocolVersionPIM, SequenceNumberPIM,
        VendorIdPIM,
    }};

pub trait Transport<
    PSM: LocatorPIM
        + ProtocolIdPIM
        + ProtocolVersionPIM
        + VendorIdPIM
        + GuidPrefixPIM
        + EntityIdPIM
        + SequenceNumberPIM
        + CountPIM
        + ParameterIdPIM
        + DataPIM
        + FragmentNumberPIM
        + UShortSubmessageElementPIM
        + ULongSubmessageElementPIM
        + TimePIM
        + ParameterListSubmessageElementPIM<PSM>
        + RtpsSubmessageHeaderPIM<PSM>
        + EntityIdSubmessageElementPIM<PSM>
        + SequenceNumberSubmessageElementPIM<PSM>
        + SequenceNumberSetSubmessageElementPIM<PSM>
        + CountSubmessageElementPIM<PSM>
        + for<'a> SerializedDataSubmessageElementPIM<'a>
        + for<'a> SerializedDataFragmentSubmessageElementPIM<'a>
        + FragmentNumberSubmessageElementPIM<PSM>
        + GuidPrefixSubmessageElementPIM<PSM>
        + LocatorListSubmessageElementPIM<PSM>
        + ProtocolVersionSubmessageElementPIM<PSM>
        + VendorIdSubmessageElementPIM<PSM>
        + TimestampSubmessageElementPIM<PSM>
        + FragmentNumberSetSubmessageElementPIM<PSM>
        + AckNackSubmessagePIM<PSM>
        + for<'a> DataSubmessagePIM<'a, PSM>
        + for<'a> DataFragSubmessagePIM<'a, PSM>
        + GapSubmessagePIM<PSM>
        + HeartbeatSubmessagePIM<PSM>
        + HeartbeatFragSubmessagePIM<PSM>
        + InfoDestinationSubmessagePIM<PSM>
        + InfoReplySubmessagePIM<PSM>
        + InfoSourceSubmessagePIM<PSM>
        + InfoTimestampSubmessagePIM<PSM>
        + NackFragSubmessagePIM<PSM>
        + PadSubmessagePIM<PSM>
        + RtpsMessageHeaderPIM<PSM>
        + SubmessageKindPIM,
>: Send + Sync
{
    fn write<'a>(
        &mut self,
        message: &<PSM as RTPSMessagePIM<'a, PSM>>::RTPSMessageType,
        destination_locator: &PSM::LocatorType,
    ) where
        PSM: RTPSMessagePIM<'a, PSM>;

    fn read<'a>(&'a self) -> Option<(PSM::RTPSMessageType, PSM::LocatorType)>
    where
        PSM: RTPSMessagePIM<'a, PSM>;

    fn unicast_locator_list(&self) -> &[PSM::LocatorType];

    fn multicast_locator_list(&self) -> &[PSM::LocatorType];
}
