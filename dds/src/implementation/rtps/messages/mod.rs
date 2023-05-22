use self::{
    overall_structure::RtpsMessageHeader,
    submessages::{
        AckNackSubmessage, DataFragSubmessage,  GapSubmessage,
        HeartbeatFragSubmessage, HeartbeatSubmessage, InfoDestinationSubmessage,
        InfoReplySubmessage, InfoSourceSubmessage, InfoTimestampSubmessage, NackFragSubmessage,
        PadSubmessage, DataSubmessageRead, DataSubmessageWrite,
    },
};

pub mod overall_structure;
pub mod submessage_elements;
pub mod submessages;
pub mod types;

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsMessageRead<'a> {
    header: RtpsMessageHeader,
    submessages: Vec<RtpsSubmessageReadKind<'a>>,
}

impl<'a> RtpsMessageRead<'a> {
    pub fn new(header: RtpsMessageHeader, submessages: Vec<RtpsSubmessageReadKind<'a>>) -> Self {
        Self {
            header,
            submessages,
        }
    }

    pub fn header(&self) -> RtpsMessageHeader {
        self.header
    }

    pub fn submessages(&self) -> &[RtpsSubmessageReadKind] {
        self.submessages.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsMessageWrite<'a> {
    header: RtpsMessageHeader,
    submessages: Vec<RtpsSubmessageWriteKind<'a>>,
}

impl<'a> RtpsMessageWrite<'a> {
    pub fn new(header: RtpsMessageHeader, submessages: Vec<RtpsSubmessageWriteKind<'a>>) -> Self {
        Self {
            header,
            submessages,
        }
    }

    pub fn header(&self) -> RtpsMessageHeader {
        self.header
    }

    pub fn submessages(&self) -> &[RtpsSubmessageWriteKind] {
        self.submessages.as_ref()
    }
}


#[derive(Debug, PartialEq, Eq)]
pub enum RtpsSubmessageReadKind<'a> {
    AckNack(AckNackSubmessage),
    Data(DataSubmessageRead<'a>),
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

#[derive(Debug, PartialEq, Eq)]
pub enum RtpsSubmessageWriteKind<'a> {
    AckNack(AckNackSubmessage),
    Data(DataSubmessageWrite<'a>),
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
