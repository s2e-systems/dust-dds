use crate::RtpsUdpPsm;

use super::SubmessageHeader;

pub struct Pad;

impl<'a> rust_rtps_pim::messages::submessages::PadSubmessage<'a, RtpsUdpPsm> for Pad {}

impl<'a> rust_rtps_pim::messages::Submessage<'a, RtpsUdpPsm> for Pad {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
