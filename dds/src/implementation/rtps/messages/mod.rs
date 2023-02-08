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
    header: RtpsMessageHeader,
    submessages: Vec<RtpsSubmessageKind<'a>>,
}

impl<'a> RtpsMessage<'a> {
    pub fn new(header: RtpsMessageHeader, submessages: Vec<RtpsSubmessageKind<'a>>) -> Self {
        Self {
            header,
            submessages,
        }
    }

    pub fn header(&self) -> RtpsMessageHeader {
        self.header
    }

    pub fn submessages(&self) -> &[RtpsSubmessageKind] {
        self.submessages.as_ref()
    }
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
