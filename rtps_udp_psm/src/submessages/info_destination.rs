use rust_rtps_pim::messages::types::SubmessageFlag;

use crate::{
    psm::RtpsUdpPsm, submessage_elements::GuidPrefix, submessage_header::SubmessageHeader,
};

#[derive(Debug, PartialEq)]
pub struct InfoDestination;

impl<'a> rust_rtps_pim::messages::submessages::InfoDestinationSubmessage<RtpsUdpPsm>
    for InfoDestination
{
    fn new(_endianness_flag: SubmessageFlag, _guid_prefix: GuidPrefix) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn guid_prefix(&self) -> &GuidPrefix {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage for InfoDestination {
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
