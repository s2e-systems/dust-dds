use rtps_pim::{
    messages::{
        overall_structure::RtpsMessageHeader,
        submessages::{
            AckNackSubmessage, DataFragSubmessage, DataSubmessage, GapSubmessage,
            HeartbeatFragSubmessage, HeartbeatSubmessage, InfoDestinationSubmessage,
            InfoReplySubmessage, InfoSourceSubmessage, InfoTimestampSubmessage, NackFragSubmessage,
            PadSubmessage,
        },
    },
    structure::types::Locator,
};

#[derive(Debug, PartialEq)]
pub struct RtpsMessage<'a> {
    pub header: RtpsMessageHeader,
    pub submessages: Vec<RtpsSubmessageType<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum RtpsSubmessageType<'a> {
    AckNack(AckNackSubmessage),
    Data(DataSubmessage<'a>),
    DataFrag(DataFragSubmessage<'a>),
    Gap(GapSubmessage),
    Heartbeat(HeartbeatSubmessage),
    HeartbeatFrag(HeartbeatFragSubmessage),
    InfoDestination(InfoDestinationSubmessage),
    InfoReply(InfoReplySubmessage),
    InfoSource(InfoSourceSubmessage),
    InfoTimestamp(InfoTimestampSubmessage),
    NackFrag(NackFragSubmessage),
    Pad(PadSubmessage),
}

pub trait TransportWrite {
    fn write(&mut self, message: &RtpsMessage<'_>, destination_locator: Locator);
}

pub trait TransportRead<'a> {
    fn read(&'a mut self) -> Option<(Locator, RtpsMessage<'a>)>;
}
