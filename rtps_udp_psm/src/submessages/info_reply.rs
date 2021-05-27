use crate::{LocatorList, RtpsUdpPsm, SubmessageFlag};

use super::SubmessageHeader;

pub struct InfoReply;

impl rust_rtps_pim::messages::submessages::InfoReply<RtpsUdpPsm> for InfoReply {
    type LocatorList = LocatorList;

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
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }
}
