use crate::structure::types::{GuidPrefix, ProtocolVersion, VendorId};

use super::{
    submessages::{
        AckNackSubmessage, DataFragSubmessage, DataSubmessage, GapSubmessage,
        HeartbeatFragSubmessage, HeartbeatSubmessage, InfoDestinationSubmessage,
        InfoReplySubmessage, InfoSourceSubmessage, InfoTimestampSubmessage, NackFragSubmessage,
        PadSubmessage,
    },
    types::{ProtocolId, SubmessageFlag, SubmessageKind},
};

#[derive(Clone, Debug, PartialEq)]
pub struct RtpsMessageHeader {
    pub protocol: ProtocolId,
    pub version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}

#[derive(Debug, PartialEq)]
pub struct RtpsSubmessageHeader {
    pub submessage_id: SubmessageKind,
    pub flags: [SubmessageFlag; 8],
    pub submessage_length: u16,
}

#[derive(Debug, PartialEq)]
pub struct RtpsMessage<M> {
    pub header: RtpsMessageHeader,
    pub submessages: M,
}

#[derive(Debug, PartialEq)]
pub enum RtpsSubmessageType<S, P, D, L, F> {
    AckNack(AckNackSubmessage<S>),
    Data(DataSubmessage<P, D>),
    DataFrag(DataFragSubmessage<P, D>),
    Gap(GapSubmessage<S>),
    Heartbeat(HeartbeatSubmessage),
    HeartbeatFrag(HeartbeatFragSubmessage),
    InfoDestination(InfoDestinationSubmessage),
    InfoReply(InfoReplySubmessage<L>),
    InfoSource(InfoSourceSubmessage),
    InfoTimestamp(InfoTimestampSubmessage),
    NackFrag(NackFragSubmessage<F>),
    Pad(PadSubmessage),
}
