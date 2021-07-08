use rust_rtps_pim::{
    behavior::types::ParticipantMessageDataPIM,
    messages::{
        submessages::RtpsSubmessagePIM,
        types::{CountPIM, GroupDigestPIM, SubmessageKindPIM, TimePIM},
    },
};

use crate::{
    submessage_elements::{CountUdp, GroupDigestUdp, TimeUdp},
    submessages::{
        ack_nack::AckNackUdp, data::DataSubmesageUdp, data_frag::DataFragUdp,
        gap::GapSubmessageUdp, heartbeat::HeartbeatSubmessageUdp, heartbeat_frag::HeartbeatFragUdp,
        info_destination::InfoDestinationUdp, info_reply::InfoReplyUdp, info_source::InfoSourceUdp,
        info_timestamp::InfoTimestampUdp, nack_frag::NackFragUdp, pad::PadUdp,
    },
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

pub(crate) type SubmessageKind = u8;

impl<'a> SubmessageKindPIM for RtpsUdpPsm<'a> {
    type SubmessageKindType = SubmessageKind;
    const DATA: Self::SubmessageKindType = 0x15;
    const GAP: Self::SubmessageKindType = 0x08;
    const HEARTBEAT: Self::SubmessageKindType = 0x07;
    const ACKNACK: Self::SubmessageKindType = 0x06;
    const PAD: Self::SubmessageKindType = 0x01;
    const INFO_TS: Self::SubmessageKindType = 0x09;
    const INFO_REPLY: Self::SubmessageKindType = 0x0f;
    const INFO_DST: Self::SubmessageKindType = 0x0e;
    const INFO_SRC: Self::SubmessageKindType = 0x0c;
    const DATA_FRAG: Self::SubmessageKindType = 0x16;
    const NACK_FRAG: Self::SubmessageKindType = 0x12;
    const HEARTBEAT_FRAG: Self::SubmessageKindType = 0x13;
}

impl<'a> TimePIM for RtpsUdpPsm<'a> {
    type TimeType = TimeUdp;
    const TIME_ZERO: Self::TimeType = TimeUdp {
        seconds: 0,
        fraction: 0,
    };
    const TIME_INVALID: Self::TimeType = TimeUdp {
        seconds: 0xffffffff,
        fraction: 0xffffffff,
    };
    const TIME_INFINITE: Self::TimeType = TimeUdp {
        seconds: 0xffffffff,
        fraction: 0xfffffffe,
    };
}

impl<'a> CountPIM for RtpsUdpPsm<'a> {
    type CountType = CountUdp;
}

impl<'a> GroupDigestPIM for RtpsUdpPsm<'a> {
    type GroupDigestType = GroupDigestUdp;
}

impl<'a> ParticipantMessageDataPIM for RtpsUdpPsm<'a> {
    type ParticipantMessageDataType = ();
}
