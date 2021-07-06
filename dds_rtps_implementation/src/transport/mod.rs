use rust_rtps_pim::{
    messages::{
        submessages::{
            AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessagePIM,
            HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM,
            InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM,
            NackFragSubmessagePIM, PadSubmessagePIM,
        },
        types::ProtocolIdPIM,
        RTPSMessagePIM, RtpsMessageHeaderPIM,
    },
    structure::types::Locator,
};

pub trait TransportWrite<PSM> {
    fn write<'a>(&mut self, message: &PSM::RTPSMessageType, destination_locator: &Locator)
    where
        PSM: RTPSMessagePIM<'a, PSM>
            + ProtocolIdPIM
            + RtpsMessageHeaderPIM
            + AckNackSubmessagePIM
            + DataSubmessagePIM<'a>
            + DataFragSubmessagePIM<'a>
            + GapSubmessagePIM
            + HeartbeatSubmessagePIM
            + HeartbeatFragSubmessagePIM
            + InfoDestinationSubmessagePIM
            + InfoReplySubmessagePIM
            + InfoSourceSubmessagePIM
            + InfoTimestampSubmessagePIM
            + NackFragSubmessagePIM
            + PadSubmessagePIM;
}

pub trait TransportRead<PSM> {
    fn read<'a>(&self) -> Option<(PSM::RTPSMessageType, Locator)>
    where
        PSM: RTPSMessagePIM<'a, PSM>
            + ProtocolIdPIM
            + RtpsMessageHeaderPIM
            + AckNackSubmessagePIM
            + DataSubmessagePIM<'a>
            + DataFragSubmessagePIM<'a>
            + GapSubmessagePIM
            + HeartbeatSubmessagePIM
            + HeartbeatFragSubmessagePIM
            + InfoDestinationSubmessagePIM
            + InfoReplySubmessagePIM
            + InfoSourceSubmessagePIM
            + InfoTimestampSubmessagePIM
            + NackFragSubmessagePIM
            + PadSubmessagePIM;
}

pub trait TransportLocator {
    fn unicast_locator_list(&self) -> &[Locator];

    fn multicast_locator_list(&self) -> &[Locator];
}
