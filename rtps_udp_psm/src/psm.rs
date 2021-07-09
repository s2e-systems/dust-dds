use rust_rtps_pim::{
    behavior::types::ParticipantMessageDataPIM, messages::submessages::RtpsSubmessagePIM,
};

use crate::submessages::{
    ack_nack::AckNackUdp, data::DataSubmesageUdp, data_frag::DataFragUdp, gap::GapSubmessageUdp,
    heartbeat::HeartbeatSubmessageUdp, heartbeat_frag::HeartbeatFragUdp,
    info_destination::InfoDestinationUdp, info_reply::InfoReplyUdp, info_source::InfoSourceUdp,
    info_timestamp::InfoTimestampUdp, nack_frag::NackFragUdp, pad::PadUdp,
};

#[derive(Debug, PartialEq)]
pub struct RtpsUdpPsm<'a>(&'a ());

impl<'a> RtpsSubmessagePIM<'a> for RtpsUdpPsm<'a> {
    type AckNackSubmessageType = AckNackUdp;
    type DataSubmessageType = DataSubmesageUdp<'a>;
    type DataFragSubmessageType = DataFragUdp<'a>;
    type GapSubmessageType = GapSubmessageUdp;
    type HeartbeatSubmessageType = HeartbeatSubmessageUdp;
    type HeartbeatFragSubmessageType = HeartbeatFragUdp;
    type InfoDestinationSubmessageType = InfoDestinationUdp;
    type InfoReplySubmessageType = InfoReplyUdp;
    type InfoSourceSubmessageType = InfoSourceUdp;
    type InfoTimestampSubmessageType = InfoTimestampUdp;
    type NackFragSubmessageType = NackFragUdp;
    type PadSubmessageType = PadUdp;
}

// impl<'a> SubmessageKindPIM for RtpsUdpPsm<'a> {
//     type SubmessageKindType = SubmessageKind;
//     const DATA: Self::SubmessageKindType = 0x15;
//     const GAP: Self::SubmessageKindType = 0x08;
//     const HEARTBEAT: Self::SubmessageKindType = 0x07;
//     const ACKNACK: Self::SubmessageKindType = 0x06;
//     const PAD: Self::SubmessageKindType = 0x01;
//     const INFO_TS: Self::SubmessageKindType = 0x09;
//     const INFO_REPLY: Self::SubmessageKindType = 0x0f;
//     const INFO_DST: Self::SubmessageKindType = 0x0e;
//     const INFO_SRC: Self::SubmessageKindType = 0x0c;
//     const DATA_FRAG: Self::SubmessageKindType = 0x16;
//     const NACK_FRAG: Self::SubmessageKindType = 0x12;
//     const HEARTBEAT_FRAG: Self::SubmessageKindType = 0x13;
// }

impl<'a> ParticipantMessageDataPIM for RtpsUdpPsm<'a> {
    type ParticipantMessageDataType = ();
}
