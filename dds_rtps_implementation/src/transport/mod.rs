use rust_rtps_pim::{
    messages::RTPSMessagePIM,
    messages::{
        submessage_elements::{
            EntityIdSubmessageElementPIM, ParameterListSubmessageElementPIM,
            SequenceNumberSubmessageElementPIM, SerializedDataSubmessageElementPIM,
        },
        submessages::{
            AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessagePIM,
            HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM, InfoDestinationSubmessagePIM,
            InfoReplySubmessagePIM, InfoSourceSubmessagePIM, InfoTimestampSubmessagePIM,
            NackFragSubmessagePIM, PadSubmessagePIM, RtpsSubmessageType,
        },
        RtpsSubmessageHeaderPIM,
    },
    structure::types::Locator,
};

pub trait TransportWrite {
    fn write<T: (&mut self, data: &serde::Serialize, destination_locator: &Locator);
}

pub trait TransportRead {
    fn read(&'a self) -> Option<(PSM::RTPSMessageType, Locator)>;
}

pub trait TransportLocator {
    fn unicast_locator_list(&self) -> &[Locator];

    fn multicast_locator_list(&self) -> &[Locator];
}
