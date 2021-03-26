use rust_rtps_pim::messages::submessages::{submessage_elements, SubmessageHeader};

use crate::{
    messages::types::{SubmessageFlag, SubmessageKind},
    types::Locator,
};

pub struct InfoReply {
    endianness_flag: SubmessageFlag,
    multicast_flag: SubmessageFlag,
    unicast_locator_list: submessage_elements::LocatorList<Locator, Vec<Locator>>,
    multicast_locator_list: submessage_elements::LocatorList<Locator, Vec<Locator>>,
}

impl rust_rtps_pim::messages::submessages::info_reply_submessage::InfoReply for InfoReply {
    type Locator = Locator;
    type LocatorList = Vec<Locator>;

    fn new(
        endianness_flag: SubmessageFlag,
        multicast_flag: SubmessageFlag,
        unicast_locator_list: submessage_elements::LocatorList<Self::Locator, Self::LocatorList>,
        multicast_locator_list: submessage_elements::LocatorList<Self::Locator, Self::LocatorList>,
    ) -> Self {
        Self {
            endianness_flag,
            multicast_flag,
            unicast_locator_list,
            multicast_locator_list,
        }
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn multicast_flag(&self) -> SubmessageFlag {
        self.multicast_flag
    }

    fn unicast_locator_list(
        &self,
    ) -> &submessage_elements::LocatorList<Self::Locator, Self::LocatorList> {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(
        &self,
    ) -> &submessage_elements::LocatorList<Self::Locator, Self::LocatorList> {
        &self.multicast_locator_list
    }
}

impl rust_rtps_pim::messages::submessages::Submessage for InfoReply {
    type SubmessageKind = SubmessageKind;
    type SubmessageFlag = SubmessageFlag;

    fn submessage_header(&self) -> SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
        todo!()
    }
}
