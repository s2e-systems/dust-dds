use crate::RtpsUdpPsm;

pub struct Pad;

impl rust_rtps_pim::messages::submessages::Pad<RtpsUdpPsm> for Pad {}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for Pad {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
