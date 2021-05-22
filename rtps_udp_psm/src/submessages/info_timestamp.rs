use crate::RtpsUdpPsm;

pub struct InfoTimestamp;

impl rust_rtps_pim::messages::submessages::InfoTimestamp<RtpsUdpPsm> for InfoTimestamp {
    fn endianness_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn invalidate_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn timestamp(&self) -> rust_rtps_pim::messages::submessage_elements::Timestamp<RtpsUdpPsm> {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for InfoTimestamp {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
