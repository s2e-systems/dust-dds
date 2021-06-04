use crate::{LocatorList, RtpsUdpPsm, SubmessageFlag};

use super::SubmessageHeader;

pub struct InfoReply;

impl rust_rtps_pim::messages::submessages::InfoReplySubmessage<RtpsUdpPsm> for InfoReply {
    type LocatorList = LocatorList;

    fn new(
        _endianness_flag: SubmessageFlag,
        _multicast_flag: SubmessageFlag,
        _unicast_locator_list: Self::LocatorList,
        _multicast_locator_list: Self::LocatorList,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn multicast_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn unicast_locator_list(&self) -> &Self::LocatorList {
        todo!()
    }

    fn multicast_locator_list(&self) -> &Self::LocatorList {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for InfoReply {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
