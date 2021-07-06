use rust_rtps_pim::{messages::{RTPSMessage, RTPSMessagePIM, RtpsMessageHeaderPIM, submessages::{
            AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessagePIM,
            HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM,
            InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM,
            NackFragSubmessagePIM, PadSubmessagePIM,
        }}, structure::types::Locator};

pub trait TransportWrite<'a> {
    type RTPSMessageType: RTPSMessage<'a>;
    fn write(&mut self, message: &Self::RTPSMessageType, destination_locator: &Locator);
}

pub trait TransportRead<PSM> {
    fn read<'a>(&self) -> Option<(PSM::RTPSMessageType, Locator)>
    where
        PSM: RTPSMessagePIM<'a>;
}

pub trait TransportLocator {
    fn unicast_locator_list(&self) -> &[Locator];

    fn multicast_locator_list(&self) -> &[Locator];
}
