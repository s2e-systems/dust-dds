use super::SubmessageHeader;
use crate::messages::submessage_elements;
use rust_rtps_pim::messages::{submessages::Submessage, types::SubmessageFlag};

pub struct InfoReply {
    endianness_flag: SubmessageFlag,
    multicast_flag: SubmessageFlag,
    unicast_locator_list: <Self as rust_rtps_pim::messages::submessages::info_reply_submessage::InfoReply>::LocatorList,
    multicast_locator_list: <Self as rust_rtps_pim::messages::submessages::info_reply_submessage::InfoReply>::LocatorList,
}

impl Submessage for InfoReply {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }

    fn is_valid(&self) -> bool {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::info_reply_submessage::InfoReply for InfoReply {
    type LocatorList = submessage_elements::LocatorList;

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn multicast_flag(&self) -> SubmessageFlag {
        self.multicast_flag
    }

    fn unicast_locator_list(&self) -> &Self::LocatorList {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &Self::LocatorList {
        &self.multicast_locator_list
    }
}
