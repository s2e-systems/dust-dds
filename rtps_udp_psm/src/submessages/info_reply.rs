use rust_rtps_pim::messages::types::SubmessageFlag;

use crate::{
    psm::RtpsUdpPsm, submessage_elements::LocatorListUdp, submessage_header::SubmessageHeader,
};

#[derive(Debug, PartialEq)]
pub struct InfoReplyUdp;

impl<'a> rust_rtps_pim::messages::submessages::InfoReplySubmessage<RtpsUdpPsm<'a>> for InfoReplyUdp {
    fn new(
        _endianness_flag: SubmessageFlag,
        _multicast_flag: SubmessageFlag,
        _unicast_locator_list: LocatorListUdp,
        _multicast_locator_list: LocatorListUdp,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn multicast_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn unicast_locator_list(&self) -> &LocatorListUdp {
        todo!()
    }

    fn multicast_locator_list(&self) -> &LocatorListUdp {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage for InfoReplyUdp {
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
