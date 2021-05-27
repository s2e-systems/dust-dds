use crate::RtpsUdpPsm;

use super::SubmessageHeader;

pub struct Pad;

impl rust_rtps_pim::messages::submessages::Pad<RtpsUdpPsm> for Pad {}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for Pad {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }
}
