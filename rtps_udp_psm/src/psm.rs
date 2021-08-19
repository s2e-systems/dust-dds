use rust_rtps_pim::messages::{
    submessage_elements::Parameter,
    submessages::{DataSubmessage, RtpsSubmessagePIM},
};

use crate::submessages::{
    ack_nack::AckNackUdp, data::DataSubmesageUdp, data_frag::DataFragUdp, gap::GapSubmessageUdp,
    heartbeat::HeartbeatSubmessageUdp, heartbeat_frag::HeartbeatFragUdp,
    info_destination::InfoDestinationUdp, info_reply::InfoReplyUdp, info_source::InfoSourceUdp,
    info_timestamp::InfoTimestampUdp, nack_frag::NackFragUdp, pad::PadUdp,
};

#[derive(Debug, PartialEq)]
pub struct RtpsUdpPsm;

impl<'a> RtpsSubmessagePIM<'a> for RtpsUdpPsm {
    type AckNackSubmessageType = AckNackUdp;
    type DataSubmessageType = DataSubmessage<'a, &'a [Parameter<'a>]>;
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
