use crate::{GuidPrefix, RtpsUdpPsm, SubmessageFlag};

use super::SubmessageHeader;

pub struct InfoDestination;

impl rust_rtps_pim::messages::submessages::InfoDestinationSubmessage<RtpsUdpPsm>
    for InfoDestination
{
    type GuidPrefix = GuidPrefix;

    fn new(_endianness_flag: SubmessageFlag, _guid_prefix: Self::GuidPrefix) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn guid_prefix(&self) -> &Self::GuidPrefix {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for InfoDestination {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
