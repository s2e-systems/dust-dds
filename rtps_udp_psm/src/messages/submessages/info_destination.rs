use rust_rtps_pim::messages::submessages::{submessage_elements, SubmessageHeader};

use crate::{
    messages::types::{SubmessageFlag, SubmessageKind},
    types::GuidPrefix,
};

pub struct InfoDestination {
    endianness_flag: SubmessageFlag,
    guid_prefix: submessage_elements::GuidPrefix<GuidPrefix>,
}

impl rust_rtps_pim::messages::submessages::info_destination_submessage::InfoDestination
    for InfoDestination
{
    type GuidPrefix = GuidPrefix;

    fn new(
        endianness_flag: SubmessageFlag,
        guid_prefix: submessage_elements::GuidPrefix<Self::GuidPrefix>,
    ) -> Self {
        Self {
            endianness_flag,
            guid_prefix,
        }
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn guid_prefix(&self) -> &submessage_elements::GuidPrefix<Self::GuidPrefix> {
        &self.guid_prefix
    }
}

impl rust_rtps_pim::messages::submessages::Submessage for InfoDestination {
    type SubmessageKind = SubmessageKind;
    type SubmessageFlag = SubmessageFlag;

    fn submessage_header(&self) -> SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
        todo!()
    }
}
