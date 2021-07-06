use rust_rtps_pim::messages::types::SubmessageFlag;

use crate::{
    psm::RtpsUdpPsm, submessage_elements::LocatorList, submessage_header::SubmessageHeader,
};

#[derive(Debug, PartialEq)]
pub struct InfoReply;

impl<'a> rust_rtps_pim::messages::submessages::InfoReplySubmessage<RtpsUdpPsm> for InfoReply {
    fn new(
        _endianness_flag: SubmessageFlag,
        _multicast_flag: SubmessageFlag,
        _unicast_locator_list: LocatorList,
        _multicast_locator_list: LocatorList,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn multicast_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn unicast_locator_list(&self) -> &LocatorList {
        todo!()
    }

    fn multicast_locator_list(&self) -> &LocatorList {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage for InfoReply {
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
