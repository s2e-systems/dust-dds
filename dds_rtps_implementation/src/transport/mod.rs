use rust_rtps_pim::{messages::RTPSMessagePIM, messages::{RtpsSubmessageHeaderPIM, submessage_elements::{EntityIdSubmessageElementPIM, ParameterListSubmessageElementPIM, SequenceNumberSubmessageElementPIM, SerializedDataSubmessageElementPIM}, submessages::{AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessagePIM, HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM, InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM, NackFragSubmessagePIM, PadSubmessagePIM, RtpsSubmessageType}}, structure::types::Locator};

pub trait Transport<PSM>
{
    fn write<'a>(
        &mut self,
        message: &[RtpsSubmessageType<'a, PSM>],
        destination_locator: &Locator,
    ) where
    PSM: AckNackSubmessagePIM
    + DataSubmessagePIM<'a, PSM>
    + DataFragSubmessagePIM<'a>
    + GapSubmessagePIM
    + HeartbeatSubmessagePIM
    + HeartbeatFragSubmessagePIM
    + InfoDestinationSubmessagePIM
    + InfoReplySubmessagePIM
    + InfoSourceSubmessagePIM
    + InfoTimestampSubmessagePIM
    + NackFragSubmessagePIM
    + PadSubmessagePIM
    + RtpsSubmessageHeaderPIM
    + EntityIdSubmessageElementPIM
    + SequenceNumberSubmessageElementPIM
    + ParameterListSubmessageElementPIM
    + SerializedDataSubmessageElementPIM<'a>;

    fn read<'a>(&'a self) -> Option<(PSM::RTPSMessageType, Locator)>
    where
        PSM: RTPSMessagePIM<'a, PSM>;

    fn unicast_locator_list(&self) -> &[Locator];

    fn multicast_locator_list(&self) -> &[Locator];
}
