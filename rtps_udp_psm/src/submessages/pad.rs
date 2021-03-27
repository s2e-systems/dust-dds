use rust_rtps_pim::messages::Submessage;

use crate::RtpsUdpPsm;

pub struct Pad {}

impl Submessage for Pad {
    type PSM = RtpsUdpPsm;

    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<Self::PSM> {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::Pad for Pad {}
