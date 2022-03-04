use rust_rtps_pim::messages::overall_structure::RtpsMessageHeader;

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
pub enum RtpsSubmessageTypeWrite<'a> {
    AckNack(AckNackSubmessageWrite),
    Data(DataSubmessageWrite<'a>),
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
pub struct RtpsMessageWrite<'a> {
    pub header: RtpsMessageHeader,
    pub submessages: Vec<RtpsSubmessageTypeWrite<'a>>,
}

impl<'a> RtpsMessageWrite<'a> {
    pub fn new(header: RtpsMessageHeader, submessages: Vec<RtpsSubmessageTypeWrite<'a>>) -> Self {
        Self {
            header,
            submessages,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RtpsMessageRead<'a> {
    pub header: RtpsMessageHeader,
    pub submessages: Vec<RtpsSubmessageTypeRead<'a>>,
}

impl<'a> RtpsMessageRead<'a> {
    pub fn new(header: RtpsMessageHeader, submessages: Vec<RtpsSubmessageTypeRead<'a>>) -> Self {
        Self {
            header,
            submessages,
        }
    }
}
