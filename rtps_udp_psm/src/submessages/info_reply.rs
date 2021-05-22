use crate::RtpsUdpPsm;

pub struct InfoReply;

impl rust_rtps_pim::messages::submessages::InfoReply<RtpsUdpPsm> for InfoReply {
    fn endianness_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn multicast_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn unicast_locator_list(
        &self,
    ) -> rust_rtps_pim::messages::submessage_elements::LocatorList<RtpsUdpPsm> {
        todo!()
    }

    fn multicast_locator_list(
        &self,
    ) -> rust_rtps_pim::messages::submessage_elements::LocatorList<RtpsUdpPsm> {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for InfoReply {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
