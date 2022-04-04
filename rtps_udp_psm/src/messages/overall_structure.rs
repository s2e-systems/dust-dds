use rtps_pim::{
    messages::{
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
pub enum RtpsSubmessageType<'a> {
    AckNack(AckNackSubmessage<Vec<SequenceNumber>>),
    Data(DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>),
    DataFrag(DataFragSubmessage<'a, Vec<Parameter<'a>>>),
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
