use rtps_pim::{
    messages::{
        overall_structure::RtpsMessageHeader,
        submessage_elements::Parameter,
        submessages::{
            AckNackSubmessage, DataFragSubmessage, DataSubmessage, GapSubmessage,
            HeartbeatFragSubmessage, HeartbeatSubmessage, InfoDestinationSubmessage,
            InfoReplySubmessage, InfoSourceSubmessage, InfoTimestampSubmessage, NackFragSubmessage,
            PadSubmessage,
        },
        types::FragmentNumber,
    },
    structure::types::{Locator, SequenceNumber},
};

#[derive(Debug, PartialEq)]
pub struct RtpsMessage<'a> {
    pub header: RtpsMessageHeader,
    pub submessages: Vec<RtpsSubmessageType<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum RtpsSubmessageType<'a> {
    AckNack(AckNackSubmessage<Vec<SequenceNumber>>),
    Data(DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>),
    DataFrag(DataFragSubmessage<Vec<Parameter<'a>>, &'a [u8]>),
    Gap(GapSubmessage<Vec<SequenceNumber>>),
    Heartbeat(HeartbeatSubmessage),
    HeartbeatFrag(HeartbeatFragSubmessage),
    InfoDestination(InfoDestinationSubmessage),
    InfoReply(InfoReplySubmessage<Vec<Locator>>),
    InfoSource(InfoSourceSubmessage),
    InfoTimestamp(InfoTimestampSubmessage),
    NackFrag(NackFragSubmessage<Vec<FragmentNumber>>),
    Pad(PadSubmessage),
}

pub trait TransportWrite {
    fn write(&mut self, message: &RtpsMessage<'_>, destination_locator: Locator);
}

pub trait TransportRead<'a> {
    fn read(&'a mut self) -> Option<(Locator, RtpsMessage<'a>)>;
}
