use rust_rtps_pim::messages::types::SubmessageFlag;

use crate::{submessage_elements::GuidPrefixUdp, submessage_header::SubmessageHeader,
};

#[derive(Debug, PartialEq)]
pub struct InfoDestinationUdp;

impl<'a> rust_rtps_pim::messages::submessages::InfoDestinationSubmessage for InfoDestinationUdp {
    type GuidPrefixSubmessageElementType = GuidPrefixUdp;
    fn new(_endianness_flag: SubmessageFlag, _guid_prefix: GuidPrefixUdp) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn guid_prefix(&self) -> &GuidPrefixUdp {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage for InfoDestinationUdp {
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
