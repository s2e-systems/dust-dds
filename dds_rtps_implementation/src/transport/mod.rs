use rust_rtps_pim::{messages::{RTPSMessage, RTPSMessagePIM, RtpsMessageHeaderPIM, RtpsMessageHeaderType, submessages::{
            AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessagePIM,
            HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM,
            InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM,
            NackFragSubmessagePIM, PadSubmessagePIM,
        }}, structure::types::Locator};

pub trait TransportWrite {
    type RtpsMessageHeaderType: RtpsMessageHeaderType;
    type RTPSMessageType: for<'a> RTPSMessage<'a, RtpsMessageHeaderType=Self::RtpsMessageHeaderType>;
    fn write<'a>(&mut self, message: &<Self::RTPSMessageType as RTPSMessage<'a>>::Constructed, destination_locator: &Locator);
}

// pub trait TransportRead<PSM> {
//     fn read<'a>(&self) -> Option<(PSM::RTPSMessageType, Locator)>
//     where
//         PSM: RTPSMessagePIM<'a>;
// }

pub trait TransportLocator {
    fn unicast_locator_list(&self) -> &[Locator];

    fn multicast_locator_list(&self) -> &[Locator];
}
