use rust_rtps_pim::messages::overall_structure::{RtpsMessage, RtpsMessageHeader};

use super::submessages::{
    AckNackSubmessageRead, AckNackSubmessageWrite, DataFragSubmessageRead, DataFragSubmessageWrite,
    DataSubmessageRead, DataSubmessageWrite, GapSubmessageRead, GapSubmessageWrite,
    HeartbeatFragSubmessageRead, HeartbeatFragSubmessageWrite, HeartbeatSubmessageRead,
    HeartbeatSubmessageWrite, InfoDestinationSubmessageRead, InfoDestinationSubmessageWrite,
    InfoReplySubmessageRead, InfoReplySubmessageWrite, InfoSourceSubmessageRead,
    InfoSourceSubmessageWrite, InfoTimestampSubmessageRead, InfoTimestampSubmessageWrite,
    NackFragSubmessageRead, NackFragSubmessageWrite, PadSubmessageRead, PadSubmessageWrite,
};

#[derive(Debug, PartialEq)]
pub enum RtpsSubmessageTypeWrite {
    AckNack(AckNackSubmessageWrite),
    Data(DataSubmessageWrite),
    DataFrag(DataFragSubmessageWrite),
    Gap(GapSubmessageWrite),
    Heartbeat(HeartbeatSubmessageWrite),
    HeartbeatFrag(HeartbeatFragSubmessageWrite),
    InfoDestination(InfoDestinationSubmessageWrite),
    InfoReply(InfoReplySubmessageWrite),
    InfoSource(InfoSourceSubmessageWrite),
    InfoTimestamp(InfoTimestampSubmessageWrite),
    NackFrag(NackFragSubmessageWrite),
    Pad(PadSubmessageWrite),
}

#[derive(Debug, PartialEq)]
pub enum RtpsSubmessageTypeRead<'a> {
    AckNack(AckNackSubmessageRead),
    Data(DataSubmessageRead<'a>),
    DataFrag(DataFragSubmessageRead),
    Gap(GapSubmessageRead),
    Heartbeat(HeartbeatSubmessageRead),
    HeartbeatFrag(HeartbeatFragSubmessageRead),
    InfoDestination(InfoDestinationSubmessageRead),
    InfoReply(InfoReplySubmessageRead),
    InfoSource(InfoSourceSubmessageRead),
    InfoTimestamp(InfoTimestampSubmessageRead),
    NackFrag(NackFragSubmessageRead),
    Pad(PadSubmessageRead),
}

#[derive(Debug, PartialEq)]
pub struct RtpsMessageWrite(RtpsMessage<Vec<RtpsSubmessageTypeWrite>>);

impl RtpsMessageWrite {
    pub fn new(header: RtpsMessageHeader, submessages: Vec<RtpsSubmessageTypeWrite>) -> Self {
        Self(RtpsMessage {
            header,
            submessages,
        })
    }
}

impl std::ops::Deref for RtpsMessageWrite {
    type Target = RtpsMessage<Vec<RtpsSubmessageTypeWrite>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct RtpsMessageRead<'a>(RtpsMessage<Vec<RtpsSubmessageTypeRead<'a>>>);

impl<'a> RtpsMessageRead<'a> {
    pub fn new(header: RtpsMessageHeader, submessages: Vec<RtpsSubmessageTypeRead<'a>>) -> Self {
        Self(RtpsMessage {
            header,
            submessages,
        })
    }
}

impl<'a> std::ops::Deref for RtpsMessageRead<'a> {
    type Target = RtpsMessage<Vec<RtpsSubmessageTypeRead<'a>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
