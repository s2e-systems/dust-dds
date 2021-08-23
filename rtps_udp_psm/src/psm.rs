use rust_rtps_pim::{
    messages::{
        submessage_elements::Parameter,
        submessages::{DataSubmessage, GapSubmessage, RtpsSubmessagePIM},
    },
    structure::types::SequenceNumber,
};

use crate::submessages::{
    ack_nack::AckNackUdp, data_frag::DataFragUdp, heartbeat::HeartbeatSubmessageUdp,
    heartbeat_frag::HeartbeatFragUdp, info_destination::InfoDestinationUdp,
    info_reply::InfoReplyUdp, info_source::InfoSourceUdp, info_timestamp::InfoTimestampUdp,
    nack_frag::NackFragUdp, pad::PadUdp,
};

#[derive(Debug, PartialEq)]
pub struct RtpsUdpPsm;

impl<'a> RtpsSubmessagePIM<'a> for RtpsUdpPsm {
    type AckNackSubmessageType = AckNackUdp;
    type DataSubmessageType = DataSubmessage<'a, &'a [Parameter<'a>]>;
    type DataFragSubmessageType = DataFragUdp<'a>;
    type GapSubmessageType = GapSubmessage<Vec<SequenceNumber>>;
    type HeartbeatSubmessageType = HeartbeatSubmessageUdp;
    type HeartbeatFragSubmessageType = HeartbeatFragUdp;
    type InfoDestinationSubmessageType = InfoDestinationUdp;
    type InfoReplySubmessageType = InfoReplyUdp;
    type InfoSourceSubmessageType = InfoSourceUdp;
    type InfoTimestampSubmessageType = InfoTimestampUdp;
    type NackFragSubmessageType = NackFragUdp;
    type PadSubmessageType = PadUdp;
}
