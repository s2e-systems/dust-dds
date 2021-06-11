use crate::{GuidPrefix, RtpsUdpPsm, SubmessageFlag};

use super::SubmessageHeader;

pub struct InfoDestination;

impl<'a> rust_rtps_pim::messages::submessages::InfoDestinationSubmessage<'a, RtpsUdpPsm>
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

impl<'a> rust_rtps_pim::messages::Submessage<'a, RtpsUdpPsm> for InfoDestination {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }

    fn submessage_elements(
        &self,
    ) -> &[rust_rtps_pim::messages::submessage_elements::SubmessageElements<'a, RtpsUdpPsm>] {
        todo!()
    }
}
