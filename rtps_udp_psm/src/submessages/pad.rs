use crate::RtpsUdpPsm;

use super::SubmessageHeader;

pub struct Pad;

impl rust_rtps_pim::messages::submessages::PadSubmessage<RtpsUdpPsm> for Pad {}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for Pad {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }

    fn submessage_elements(
        &self,
    ) -> &[rust_rtps_pim::messages::submessage_elements::SubmessageElements<RtpsUdpPsm>] {
        &[]
    }
}
