use self::{
    overall_structure::RtpsMessageHeader,
    submessages::{
        AckNackSubmessage, DataFragSubmessage, DataSubmessage, GapSubmessage,
        HeartbeatFragSubmessage, HeartbeatSubmessage, InfoDestinationSubmessage,
        InfoReplySubmessage, InfoSourceSubmessage, InfoTimestampSubmessage, NackFragSubmessage,
        PadSubmessage,
    },
};

pub mod overall_structure;
pub mod submessage_elements;
pub mod submessages;
pub mod types;

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsMessage<'a> {
    pub header: RtpsMessageHeader,
    pub submessages: Vec<RtpsSubmessageKind<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RtpsSubmessageKind<'a> {
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
