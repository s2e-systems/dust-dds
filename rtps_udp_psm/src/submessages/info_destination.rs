use crate::RtpsUdpPsm;

pub struct InfoDestination;

impl rust_rtps_pim::messages::submessages::InfoDestination<RtpsUdpPsm> for InfoDestination {
    fn endianness_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn guid_prefix(&self) -> rust_rtps_pim::messages::submessage_elements::GuidPrefix<RtpsUdpPsm> {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for InfoDestination {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
