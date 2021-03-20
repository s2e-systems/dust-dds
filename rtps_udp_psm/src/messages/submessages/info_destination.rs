use super::SubmessageHeader;
use crate::messages::submessage_elements;
use rust_rtps_pim::messages::{submessages::Submessage, types::SubmessageFlag};

pub struct InfoDestination {
    endianness_flag: SubmessageFlag,
    guid_prefix: <Self as rust_rtps_pim::messages::submessages::info_destination_submessage::InfoDestination>::GuidPrefix,
}

impl Submessage for InfoDestination {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }

    fn is_valid(&self) -> bool {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::info_destination_submessage::InfoDestination
    for InfoDestination
{
    type GuidPrefix = submessage_elements::GuidPrefix;

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn guid_prefix(&self) -> &Self::GuidPrefix {
        &self.guid_prefix
    }
}
