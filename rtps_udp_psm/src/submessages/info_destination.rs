use crate::{GuidPrefix, RtpsUdpPsm, SubmessageFlag};

pub struct InfoDestination;

impl rust_rtps_pim::messages::submessages::InfoDestination<RtpsUdpPsm> for InfoDestination {
    type GuidPrefix = GuidPrefix;

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn guid_prefix(&self) -> &Self::GuidPrefix {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for InfoDestination {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
