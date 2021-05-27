use crate::{GuidPrefix, RtpsUdpPsm, SubmessageFlag};

use super::SubmessageHeader;

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
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }
}
