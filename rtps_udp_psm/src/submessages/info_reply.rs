use crate::{LocatorList, RtpsUdpPsm, SubmessageFlag};

use super::SubmessageHeader;

pub struct InfoReply;

impl<'a> rust_rtps_pim::messages::submessages::InfoReplySubmessage<'a, RtpsUdpPsm> for InfoReply {
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

impl<'a> rust_rtps_pim::messages::Submessage<'a, RtpsUdpPsm> for InfoReply {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }

    fn submessage_elements(
        &self,
    ) -> &[rust_rtps_pim::messages::submessage_elements::SubmessageElements<'a, RtpsUdpPsm>] {
        todo!()
    }
}
