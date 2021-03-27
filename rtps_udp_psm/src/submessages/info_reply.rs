use rust_rtps_pim::messages::Submessage;

use crate::RtpsUdpPsm;

pub struct InfoReply {
    endianness_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    multicast_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    unicast_locator_list:
        rust_rtps_pim::messages::submessage_elements::LocatorList<<Self as Submessage>::PSM>,
    multicast_locator_list:
        rust_rtps_pim::messages::submessage_elements::LocatorList<<Self as Submessage>::PSM>,
}

impl Submessage for InfoReply {
    type PSM = RtpsUdpPsm;

    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<Self::PSM> {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::InfoReply for InfoReply {
    fn new(
        endianness_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        multicast_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        unicast_locator_list: rust_rtps_pim::messages::submessage_elements::LocatorList<Self::PSM>,
        multicast_locator_list: rust_rtps_pim::messages::submessage_elements::LocatorList<
            Self::PSM,
        >,
    ) -> Self {
        Self {
            endianness_flag,
            multicast_flag,
            unicast_locator_list,
            multicast_locator_list,
        }
    }

    fn endianness_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.endianness_flag
    }

    fn multicast_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.multicast_flag
    }

    fn unicast_locator_list(
        &self,
    ) -> &rust_rtps_pim::messages::submessage_elements::LocatorList<Self::PSM> {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(
        &self,
    ) -> &rust_rtps_pim::messages::submessage_elements::LocatorList<Self::PSM> {
        &self.multicast_locator_list
    }
}
